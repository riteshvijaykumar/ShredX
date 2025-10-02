@echo off
echo Starting ShredX...
echo.

:: Check if running as Administrator
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo [WARNING] ShredX requires Administrator privileges for disk operations.
    echo.
    echo To run as Administrator:
    echo 1. Right-click this shortcut
    echo 2. Select "Run as administrator"
    echo 3. Click "Yes" when prompted
    echo.
    echo Attempting to launch anyway...
    timeout /t 3 >nul
)

:: Get current directory
set "CURRENT_DIR=%~dp0"

:: Change to ShredX directory
cd /d "%CURRENT_DIR%"

:: Initialize user configuration if not exists
if not exist "users.json" (
    echo [INFO] Creating default user configuration...
    echo {"users": [], "version": "1.0.1", "created": "%date% %time%"} > users.json
    echo [INFO] Default configuration created.
    echo.
)

:: Create directories if they don't exist
if not exist "reports" mkdir "reports"
if not exist "logs" mkdir "logs"
if not exist "certificates" mkdir "certificates"

:: Display startup message
echo ================================================================
echo                         ShredX v1.0.1
echo              Professional Disk Sanitization Tool
echo ================================================================
echo.
echo IMPORTANT SECURITY NOTICES:
echo • Administrator privileges required for disk operations
echo • All sanitization operations are IRREVERSIBLE
echo • Verify target devices before proceeding
echo • Ensure stable power during operations
echo.
echo Starting application...
echo.

:: Launch ShredX
start "" "%CURRENT_DIR%ShredX.exe"

:: Wait a moment for application to start
timeout /t 2 >nul

:: Check if application started successfully
tasklist /FI "IMAGENAME eq ShredX.exe" 2>NUL | find /I /N "ShredX.exe">NUL
if "%ERRORLEVEL%"=="0" (
    echo [SUCCESS] ShredX launched successfully.
) else (
    echo [ERROR] Failed to launch ShredX.
    echo.
    echo Troubleshooting:
    echo 1. Check if ShredX.exe exists in the current directory
    echo 2. Verify Windows compatibility
    echo 3. Run as Administrator
    echo 4. Check antivirus quarantine
    echo.
    echo Press any key to exit...
    pause >nul
)