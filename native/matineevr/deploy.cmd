@echo off
py -3 "%~dp0deploy.py" %*
set "result=%errorlevel%"
pause
exit /b %result%
