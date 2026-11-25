@echo off
setlocal
cd /d "%~dp0"
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0package_windows.ps1"
exit /b %ERRORLEVEL%
