"""Static invariants of the standalone runtime, not proof of OS isolation."""
import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
ENGINE = (ROOT / 'src' / 'sandbox.rs').read_text(encoding='utf-8')
MAIN = (ROOT / 'src' / 'main.rs').read_text(encoding='utf-8')
PACKAGE = (ROOT / 'package_windows.ps1').read_text(encoding='utf-8')


class StandaloneRuntimeContracts(unittest.TestCase):
    def test_inspector_launch_is_centralized(self):
        self.assertIn('sandbox::execute(&python, "runtime/inspector.py"', MAIN)
        self.assertIn('sandbox::locate_python()', MAIN)

    def test_no_player_docker_dependency(self):
        self.assertNotIn('Command::new("docker")', ENGINE)
        self.assertNotIn('docker pull', PACKAGE)

    def test_embedded_python_prioritized(self):
        self.assertIn('runtime/python/python.exe', ENGINE)
        self.assertIn('SHARKCODE_PYTHON', ENGINE)
        self.assertIn('#[cfg(debug_assertions)]', ENGINE)
        self.assertIn('#[cfg(not(debug_assertions))]', ENGINE)

    def test_release_does_not_silently_execute_untrusted_code(self):
        self.assertIn('SHARKCODE_ALLOW_UNSAFE_NATIVE', ENGINE)
        self.assertIn('Защищённый исполнитель Python ещё не реализован', ENGINE)
        self.assertIn('Job Objects cannot prevent filesystem/network/registry access', ENGINE)

    def test_windows_limits_and_timeout(self):
        self.assertIn('windows_job::Job::attach(&child)', ENGINE)
        self.assertIn('MAX_STREAM_BYTES', ENGINE)
        self.assertIn('RUN_TIMEOUT', ENGINE)
        self.assertIn('EphemeralWorkdir', ENGINE)

    def test_package_is_truly_offline_for_player(self):
        self.assertIn('python-$PythonVersion-embed-amd64.zip', PACKAGE)
        self.assertIn('target\\release\\sharkcode.exe', PACKAGE)
        self.assertIn('Get-FileHash', PACKAGE)
        self.assertIn('Expand-Archive', PACKAGE)
        self.assertIn('runtime\\inspector.py', PACKAGE)
        self.assertIn('SHA256_MANIFEST.json', PACKAGE)

    def test_portable_resource_base(self):
        self.assertIn('std::env::set_current_dir(directory)', MAIN)


if __name__ == '__main__':
    unittest.main()
