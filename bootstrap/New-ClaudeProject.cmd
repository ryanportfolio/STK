@echo off
REM Double-clickable wrapper for new-claude-project.ps1.
REM Prompts for a project name, creates the folder + private GitHub repo.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0new-claude-project.ps1" -Dest "%USERPROFILE%\CoreWise"
echo.
pause
