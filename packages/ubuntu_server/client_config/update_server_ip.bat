@echo off
REM Quick Server IP Configuration Update
REM Run this to quickly change server IP in your client config

setlocal enabledelayedexpansion

echo ========================================
echo HDD Tool - Quick Server IP Update
echo ========================================
echo.

REM Check if config.json exists
if not exist "..\..\config.json" (
    echo ERROR: config.json not found in the main directory
    echo Please run this script from ubuntu_server\client_config\ directory
    pause
    exit /b 1
)

REM Show current configuration
echo Current configuration:
type "..\..\config.json" | findstr "server_url"
echo.

REM Get new server IP
set /p NEW_SERVER_IP="Enter new server IP address: "

if "%NEW_SERVER_IP%"=="" (
    echo ERROR: Server IP address is required
    pause
    exit /b 1
)

echo.
echo Updating configuration with IP: %NEW_SERVER_IP%

REM Create temporary PowerShell script to update JSON
echo $config = Get-Content "..\..\config.json" ^| ConvertFrom-Json > temp_update.ps1
echo $config.server_config.server_url = "http://%NEW_SERVER_IP%" >> temp_update.ps1
echo $config.server_config.api_base_url = "http://%NEW_SERVER_IP%/api" >> temp_update.ps1
echo $config ^| ConvertTo-Json -Depth 10 ^| Set-Content "..\..\config.json" >> temp_update.ps1

REM Run the PowerShell script
powershell -ExecutionPolicy Bypass -File temp_update.ps1

REM Clean up
del temp_update.ps1

echo.
echo ========================================
echo Configuration updated successfully!
echo ========================================
echo.
echo New server settings:
echo   Server URL: http://%NEW_SERVER_IP%
echo   API URL: http://%NEW_SERVER_IP%/api
echo.
echo You can now start your HDD Tool application.
echo It will connect to the server at %NEW_SERVER_IP%
echo.
pause