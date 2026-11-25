//! SHARKCODE standalone CPython worker (Windows-first).
//!
//! Windows release bundles the official CPython embeddable distribution.
//! Resource control: Windows Job Object, bounded I/O, timeout, no automatic
//! system-Python fallback. IMPORTANT: this is NOT an OS security sandbox.
//! Job Objects cannot prevent filesystem/network/registry access.
//! Production release must not be advertised safe for untrusted os/sys code.

use std::{io::{Read, Write}, process::{Command, Stdio}, thread,
          time::{Duration, Instant}};

const MAX_REQUEST_BYTES: usize = 1024 * 1024;
const MAX_STREAM_BYTES: usize = 2 * 1024 * 1024;
const RUN_TIMEOUT: Duration = Duration::from_secs(3);

/// Process execution report: only trusted engine code should interpret stdout as JSON.
pub struct ExecutionOutput {
    pub stdout: Vec<u8>,
}

/// Bound capture memory while continuing to drain pipes to prevent pipe deadlock.
fn drain_limited<R: Read>(mut stream: R) -> std::io::Result<(Vec<u8>, bool)> {
    let mut output = Vec::new();
    let mut chunk = [0u8; 8192];
    let mut exceeded = false;
    loop {
        let count = stream.read(&mut chunk)?;
        if count == 0 { break; }
        let room = MAX_STREAM_BYTES.saturating_sub(output.len());
        output.extend_from_slice(&chunk[..count.min(room)]);
        if count > room { exceeded = true; }
    }
    Ok((output, exceeded))
}

/// Choose Python from the installation, not the user's PATH.
/// `SHARKCODE_PYTHON` is permitted only in debug builds for developer tests.
/// Release mode fails closed if the packaged interpreter is missing.
pub fn locate_python() -> Result<std::path::PathBuf, String> {
    #[cfg(debug_assertions)]
    if let Some(override_path) = std::env::var_os("SHARKCODE_PYTHON") {
        if !override_path.is_empty() {
            return Ok(std::path::PathBuf::from(override_path));
        }
    }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe_dir = exe.parent().ok_or("Не найдена папка исполняемого файла")?;
    #[cfg(windows)]
    let candidates = [exe_dir.join("runtime/python/python.exe"),
                      std::env::current_dir().map_err(|e| e.to_string())?.join("runtime/python/python.exe")];
    #[cfg(not(windows))]
    let candidates = [exe_dir.join("runtime/python/bin/python3"),
                      std::env::current_dir().map_err(|e| e.to_string())?.join("runtime/python/bin/python3")];
    for candidate in candidates {
        if candidate.is_file() {
            return candidate.canonicalize().map_err(|e| e.to_string());
        }
    }
    #[cfg(debug_assertions)]
    {
        // Source-tree convenience only. Never compiled into a release binary.
        eprintln!("SHARKCODE DEVELOPMENT: embedded Python absent; using system interpreter. NOT SANDBOXED.");
        return Ok(std::path::PathBuf::from(if cfg!(windows) { "python.exe" } else { "python3" }));
    }
    #[cfg(not(debug_assertions))]
    Err("Не найден встроенный CPython в runtime/python. Переустановите полный дистрибутив SHARKCODE.".into())
}

/// Launch the bundled Python interpreter and receive the inspector's JSON bytes.
/// For safety, a production build refuses unsafe direct execution by default.
/// This guard will be replaced by a validated OS-isolated backend in a future release.
pub fn execute(python: &std::path::Path, inspector_path: &str, payload: &[u8]) -> Result<ExecutionOutput, String> {
    if payload.len() > MAX_REQUEST_BYTES {
        return Err("Слишком большой запрос к Python (максимум 1 МиБ)".into());
    }
    if inspector_path != "runtime/inspector.py" {
        return Err("Запрещён неизвестный скрипт инспектора".into());
    }
    #[cfg(not(debug_assertions))]
    if std::env::var("SHARKCODE_ALLOW_UNSAFE_NATIVE").as_deref() != Ok("1") {
        return Err("Защищённый исполнитель Python ещё не реализован. "
            .to_string() + "В этой технической сборке запуск кода отключён, "
            + "чтобы не выдавать нативный CPython за безопасную песочницу.");
    }
    static WARNING: std::sync::Once = std::sync::Once::new();
    WARNING.call_once(|| eprintln!(
        "SHARKCODE SECURITY: native CPython is NOT isolated from files/network/registry. Use only trusted code while developing."
    ));
    let mut cmd = Command::new(python);
    let inspector = std::env::current_dir().map_err(|e| e.to_string())?
        .join("runtime/inspector.py").canonicalize().map_err(|e| e.to_string())?;
    cmd.args(["-X", "utf8"]);
    cmd.arg(inspector);
    // Avoid inheriting an unrelated Python installation's module paths.
    cmd.env_remove("PYTHONHOME").env_remove("PYTHONPATH").env_remove("PYTHONSTARTUP");
    cmd.env("PYTHONDONTWRITEBYTECODE", "1");
    // Running in a fresh temp working directory avoids accidental relative
    // writes into the game tree. It is NOT a filesystem security boundary.
    let work = EphemeralWorkdir::new()?;
    cmd.current_dir(&work.path);
    run_command(cmd, payload, true)
}

struct EphemeralWorkdir { path: std::path::PathBuf }
impl EphemeralWorkdir {
    fn new() -> Result<Self, String> {
        use std::sync::atomic::{AtomicU64, Ordering};
        static SEQUENCE: AtomicU64 = AtomicU64::new(0);
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?.as_nanos();
        let path = std::env::temp_dir().join(format!("sharkcode-{}-{}-{}",
            std::process::id(), now, SEQUENCE.fetch_add(1, Ordering::Relaxed)));
        std::fs::create_dir(&path).map_err(|e| e.to_string())?;
        Ok(Self { path })
    }
}
impl Drop for EphemeralWorkdir {
    fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.path); }
}

/// Close all pipes and terminate the process tree even on timeout or error.
fn run_command(mut cmd: Command, payload: &[u8], native_job: bool) -> Result<ExecutionOutput, String> {
    let mut child = cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped())
        .spawn().map_err(|e| format!("Не удалось запустить исполнитель: {e}"))?;

    // Native inspector waits for stdin; Job limits attach before user code runs.
    #[cfg(windows)]
    let job = if native_job {
        match windows_job::Job::attach(&child) {
            Ok(job) => Some(job),
            Err(e) => {
                let _ = child.kill(); let _ = child.wait();
                return Err(format!("Windows Job Object: {e}"));
            }
        }
    } else { None };
    #[cfg(not(windows))]
    let _ = native_job;

    let stdout = child.stdout.take().ok_or("Нет канала stdout")?;
    let stderr = child.stderr.take().ok_or("Нет канала stderr")?;
    let stdout_reader = thread::spawn(move || drain_limited(stdout));
    let stderr_reader = thread::spawn(move || drain_limited(stderr));

    let mut stdin = child.stdin.take().ok_or("Нет канала stdin")?;
    let bytes = payload.to_vec();
    // Do not block the Bevy thread on a full pipe if Docker/CPython stalls.
    let writer = thread::spawn(move || stdin.write_all(&bytes));

    let started = Instant::now();
    let wall_timeout = RUN_TIMEOUT;
    let mut timed_out = false;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() >= wall_timeout => {
                timed_out = true;
                #[cfg(windows)]
                if let Some(ref job) = job { job.terminate(); }
                stop_process(&mut child);
                break child.wait().map_err(|e| e.to_string())?;
            }
            Ok(None) => thread::sleep(Duration::from_millis(10)),
            Err(e) => {
                #[cfg(windows)]
                if let Some(ref job) = job { job.terminate(); }
                stop_process(&mut child);
                let _ = stdout_reader.join(); let _ = stderr_reader.join();
                let _ = writer.join();
                return Err(e.to_string());
            }
        }
    };
    let write_result = writer.join().map_err(|_| "Ошибка потока отправки запроса")?;
    let (stdout, stdout_overflow) = stdout_reader.join()
        .map_err(|_| "Ошибка потока чтения stdout")?
        .map_err(|e| e.to_string())?;
    let (stderr, stderr_overflow) = stderr_reader.join()
        .map_err(|_| "Ошибка потока чтения stderr")?
        .map_err(|e| e.to_string())?;
    if timed_out { return Err(format!("Превышено время выполнения ({} секунд)", wall_timeout.as_secs())); }
    if let Err(e) = write_result {
        return Err(format!("Не удалось передать запрос в CPython: {e}"));
    }
    if stdout_overflow || stderr_overflow {
        return Err("Превышен лимит вывода CPython (2 МиБ на канал)".into());
    }
    if !status.success() {
        return Err(format!("CPython runner завершился с ошибкой: {}", String::from_utf8_lossy(&stderr)));
    }
    Ok(ExecutionOutput { stdout })
}

fn stop_process(child: &mut std::process::Child) {
    let _ = child.kill();
    let _ = child.wait();
}

#[cfg(windows)]
mod windows_job {
    use std::{ffi::c_void, io, mem, os::windows::io::AsRawHandle, process::Child};
    type Handle = *mut c_void;

    #[repr(C)]
    struct BasicLimit {
        per_process_user_time_limit: i64,
        per_job_user_time_limit: i64,
        limit_flags: u32,
        minimum_working_set_size: usize,
        maximum_working_set_size: usize,
        active_process_limit: u32,
        affinity: usize,
        priority_class: u32,
        scheduling_class: u32,
    }
    #[repr(C)]
    struct IoCounters {
        read_operation_count: u64,
        write_operation_count: u64,
        other_operation_count: u64,
        read_transfer_count: u64,
        write_transfer_count: u64,
        other_transfer_count: u64,
    }
    #[repr(C)]
    struct ExtendedLimit {
        basic: BasicLimit,
        io: IoCounters,
        process_memory_limit: usize,
        job_memory_limit: usize,
        peak_process_memory_used: usize,
        peak_job_memory_used: usize,
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn CreateJobObjectW(attributes: *const c_void, name: *const u16) -> Handle;
        fn SetInformationJobObject(job: Handle, class: i32, information: *const c_void, length: u32) -> i32;
        fn AssignProcessToJobObject(job: Handle, process: Handle) -> i32;
        fn TerminateJobObject(job: Handle, exit_code: u32) -> i32;
        fn CloseHandle(handle: Handle) -> i32;
    }
    const JOB_OBJECT_EXTENDED_LIMIT_INFORMATION: i32 = 9;
    const KILL_ON_JOB_CLOSE: u32 = 0x2000;
    const ACTIVE_PROCESS_LIMIT: u32 = 0x0008;
    const PROCESS_MEMORY_LIMIT: u32 = 0x0100;
    const JOB_MEMORY_LIMIT: u32 = 0x0200;
    const MEMORY_CAP: usize = 384 * 1024 * 1024;

    /// Closing this handle kills all processes in its job, including descendants.
    pub struct Job(Handle);
    impl Drop for Job {
        fn drop(&mut self) { unsafe { CloseHandle(self.0); } }
    }
    impl Job {
        pub fn attach(child: &Child) -> Result<Self, String> {
            let handle = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
            if handle.is_null() { return Err(io::Error::last_os_error().to_string()); }
            let job = Self(handle);
            let mut limits: ExtendedLimit = unsafe { mem::zeroed() };
            limits.basic.limit_flags = KILL_ON_JOB_CLOSE | ACTIVE_PROCESS_LIMIT
                | PROCESS_MEMORY_LIMIT | JOB_MEMORY_LIMIT;
            limits.basic.active_process_limit = 1;
            limits.process_memory_limit = MEMORY_CAP;
            limits.job_memory_limit = MEMORY_CAP;
            let applied = unsafe { SetInformationJobObject(job.0,
                JOB_OBJECT_EXTENDED_LIMIT_INFORMATION, &limits as *const _ as *const c_void,
                mem::size_of::<ExtendedLimit>() as u32) };
            if applied == 0 { return Err(io::Error::last_os_error().to_string()); }
            let assigned = unsafe { AssignProcessToJobObject(job.0, child.as_raw_handle() as Handle) };
            if assigned == 0 { return Err(io::Error::last_os_error().to_string()); }
            Ok(job)
        }
        pub fn terminate(&self) { unsafe { TerminateJobObject(self.0, 1); } }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_request_is_strict() {
        let payload = vec![b'x'; MAX_REQUEST_BYTES + 1];
        assert!(execute("nonexistent-python", "none", &payload).is_err());
    }
    #[test]
    fn capped_reader_drains() {
        let data = vec![b'x'; MAX_STREAM_BYTES + 100];
        let (out, truncated) = drain_limited(data.as_slice()).unwrap();
        assert_eq!(out.len(), MAX_STREAM_BYTES);
        assert!(truncated);
    }
}
