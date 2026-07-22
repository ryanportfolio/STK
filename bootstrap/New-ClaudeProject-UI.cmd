@echo off
REM Double-clickable launcher for the New Claude Project GUI (WPF front end).
REM
REM -STA is MANDATORY: WPF requires a single-threaded apartment. Without it the
REM XamlReader/Window plumbing throws and no window ever appears.
REM -NoProfile keeps the user's PowerShell profile out of the way; the launcher
REM uses the built-in Windows PowerShell 5.1 host ("powershell", not "pwsh").
powershell -NoProfile -ExecutionPolicy Bypass -STA -File "%~dp0new-claude-project-ui.ps1"
if %ERRORLEVEL% neq 0 (
    echo.
    echo The GUI exited with an error ^(code %ERRORLEVEL%^).
    pause
)
