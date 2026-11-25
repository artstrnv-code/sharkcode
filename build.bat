@echo off
setlocal EnableExtensions
cd /d "%~dp0"
chcp 65001 >nul

echo [SHARKCODE] Build 0.2.36
where cargo >nul 2>nul
if errorlevel 1 (
  echo ERROR: Cargo not found. Install Rust toolchain and reopen terminal.
  exit /b 1
)
REM CPython is bundled by package_windows.bat for player releases.
REM Debug builds may use system Python if embedded runtime is absent.

REM Use the same project folder between releases. Cargo will reuse ./target.
REM The archive must NOT contain target or sharkcode_data.
where git >nul 2>nul
if errorlevel 1 goto git_missing
if not exist .git\ (git init || goto git_warning)
REM Record source changes before build; never auto-push or overwrite branches.
git add -A
if errorlevel 1 goto git_warning
git diff --cached --quiet
if not errorlevel 1 goto build
REM A missing Git identity should not prevent compiling the game.
git commit -m "SHARKCODE source checkpoint before build" || echo WARNING: Git commit skipped.
goto build
:git_missing
echo WARNING: Git not installed. Building without source checkpoint.
goto build
:git_warning
echo WARNING: Git checkpoint failed. Building anyway.
:build
if /I "%~1"=="release" (
  cargo build --release
  if errorlevel 1 exit /b 1
  echo OK: target\release\sharkcode.exe
) else (
  cargo build
  if errorlevel 1 exit /b 1
  echo OK: target\debug\sharkcode.exe
)
echo The same target directory is reused on subsequent builds.
exit /b 0
