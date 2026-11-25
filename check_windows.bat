@echo off
cd /d "%~dp0"
python -m unittest discover -s tests -v
if errorlevel 1 exit /b 1
cargo check
if errorlevel 1 exit /b 1
pause
