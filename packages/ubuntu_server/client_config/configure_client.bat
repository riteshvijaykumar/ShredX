@echo off
REM HDD Tool Client Configuration Script
REM Run this on your Windows machine to configure client for server connection

echo ========================================
echo HDD Tool Client Configuration
echo ========================================
echo.

REM Get server IP from user if not provided
set /p SERVER_IP="Enter your Ubuntu server IP address: "

if "%SERVER_IP%"=="" (
    echo ERROR: Server IP address is required
    pause
    exit /b 1
)

echo.
echo Configuring client to connect to: %SERVER_IP%
echo.

REM Backup existing config
if exist "config.json" (
    echo Backing up existing config.json...
    copy "config.json" "config.json.backup.%date:~-4,4%%date:~-10,2%%date:~-7,2%"
)

REM Create new config with server IP
echo Creating new configuration...
(
echo {
echo   "server_config": {
echo     "is_server_enabled": true,
echo     "server_url": "http://%SERVER_IP%",
echo     "api_base_url": "http://%SERVER_IP%/api",
echo     "enable_server_sync": true,
echo     "connection_timeout": 30,
echo     "retry_attempts": 3,
echo     "auto_upload_certificates": true,
echo     "local_storage_only": false
echo   },
echo   "client_config": {
echo     "app_name": "HDD Tool Client",
echo     "version": "1.0.0",
echo     "auto_connect": true,
echo     "cache_server_data": true,
echo     "offline_mode_fallback": true
echo   },
echo   "authentication": {
echo     "method": "server",
echo     "auto_login": false,
echo     "remember_credentials": false,
echo     "session_timeout": 1440
echo   },
echo   "network": {
echo     "ping_interval": 30,
echo     "health_check_url": "/api/health",
echo     "connection_retry_delay": 5
echo   },
echo   "ui": {
echo     "show_server_status": true,
echo     "enable_real_time_sync": true,
echo     "theme": "default"
echo   }
echo }
) > config.json

echo.
echo ========================================
echo Configuration completed successfully!
echo ========================================
echo.
echo Server Configuration:
echo   IP Address: %SERVER_IP%
echo   Web Interface: http://%SERVER_IP%
echo   API Endpoint: http://%SERVER_IP%/api
echo.
echo Next Steps:
echo 1. Start your HDD Tool desktop application
echo 2. It will now connect to the server at %SERVER_IP%
echo 3. Use server credentials to login:
echo    - Admin: admin / admin123
echo    - User: user / user123
echo.
echo Troubleshooting:
echo - If connection fails, check firewall settings
echo - Ensure server is running: http://%SERVER_IP%/api/health
echo - Check network connectivity to %SERVER_IP%
echo.
pause