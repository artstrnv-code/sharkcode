@echo off
setlocal
cd /d "%~dp0"
echo [SHARKCODE] Checking standalone CPython package...
if not exist "runtime\python\python.exe" (
    echo Missing runtime\python\python.exe. Run package_windows.bat on Windows to build a standalone package.
    exit /b 1
)
"runtime\python\python.exe" -I -c "import sys; print(sys.version); print('isolated:', sys.flags.isolated)"
if errorlevel 1 exit /b 1
echo Embedded CPython is available. WARNING: process is NOT filesystem/network sandboxed yet.
exit /b 0
