@echo off
setlocal enabledelayedexpansion

:: ShredX Standalone Installer - No Dependencies Required
:: Self-contained Windows installer for easy deployment

echo.
echo ============================================================
echo                ShredX Standalone Installer
echo          Professional Disk Sanitization Tool
echo ============================================================
echo.
echo This installer will set up ShredX on your Windows system
echo with zero dependencies and minimal configuration required.
echo.

:: Check for Administrator privileges
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo [ERROR] Administrator privileges required!
    echo.
    echo Please:
    echo 1. Right-click this installer
    echo 2. Select "Run as administrator"
    echo 3. Click "Yes" when prompted
    echo.
    echo Press any key to exit...
    pause >nul
    exit /b 1
)

echo [INFO] Administrator privileges confirmed
echo.

:: Get installation directory preference
echo Choose installation location:
echo.
echo 1. Program Files (Recommended for all users)
echo 2. Current user directory (No admin needed later)
echo 3. Custom location
echo.
set /p choice="Enter choice (1-3): "

if "%choice%"=="1" (
    set "INSTALL_DIR=%ProgramFiles%\ShredX"
    set "SHORTCUT_TYPE=all"
) else if "%choice%"=="2" (
    set "INSTALL_DIR=%USERPROFILE%\ShredX"
    set "SHORTCUT_TYPE=user"
) else if "%choice%"=="3" (
    set /p "INSTALL_DIR=Enter full path: "
    set "SHORTCUT_TYPE=user"
) else (
    echo Invalid choice. Using default location...
    set "INSTALL_DIR=%ProgramFiles%\ShredX"
    set "SHORTCUT_TYPE=all"
)

echo.
echo [INFO] Installing to: !INSTALL_DIR!
echo.

:: Create installation directory
if not exist "!INSTALL_DIR!" (
    echo [INFO] Creating installation directory...
    mkdir "!INSTALL_DIR!" 2>nul
    if !errorLevel! neq 0 (
        echo [ERROR] Failed to create directory: !INSTALL_DIR!
        echo Please check permissions or choose a different location.
        pause
        exit /b 1
    )
)

:: Copy main executable
echo [INFO] Installing ShredX executable...
if exist "ShredX.exe" (
    copy /Y "ShredX.exe" "!INSTALL_DIR!\ShredX.exe" >nul
    if !errorLevel! neq 0 (
        echo [ERROR] Failed to copy executable
        pause
        exit /b 1
    )
) else (
    echo [ERROR] ShredX.exe not found in installer package
    pause
    exit /b 1
)

:: Copy configuration files
echo [INFO] Setting up configuration...
if exist "config.json" copy /Y "config.json" "!INSTALL_DIR!\" >nul
if exist "README.txt" copy /Y "README.txt" "!INSTALL_DIR!\" >nul
if exist "LICENSE" copy /Y "LICENSE" "!INSTALL_DIR!\" >nul

:: Create data directories
echo [INFO] Creating data directories...
if not exist "!INSTALL_DIR!\reports" mkdir "!INSTALL_DIR!\reports"
if not exist "!INSTALL_DIR!\logs" mkdir "!INSTALL_DIR!\logs"
if not exist "!INSTALL_DIR!\certificates" mkdir "!INSTALL_DIR!\certificates"

:: Create launcher script for easy access
echo [INFO] Creating launcher scripts...
(
echo @echo off
echo cd /d "!INSTALL_DIR!"
echo if not exist "users.json" ^(
echo     echo Creating default user configuration...
echo     echo {"users": [], "version": "1.0.0"} ^> users.json
echo ^)
echo echo Starting ShredX...
echo echo.
echo echo IMPORTANT: ShredX requires Administrator privileges
echo echo for disk-level operations. You may see a UAC prompt.
echo echo.
echo start "" "!INSTALL_DIR!\ShredX.exe"
) > "!INSTALL_DIR!\Launch ShredX.bat"

:: Create desktop shortcut
echo [INFO] Creating desktop shortcut...
set "DESKTOP=%PUBLIC%\Desktop"
if "%SHORTCUT_TYPE%"=="user" set "DESKTOP=%USERPROFILE%\Desktop"

echo Set oWS = WScript.CreateObject("WScript.Shell") > "%TEMP%\CreateShortcut.vbs"
echo sLinkFile = "%DESKTOP%\ShredX.lnk" >> "%TEMP%\CreateShortcut.vbs"
echo Set oLink = oWS.CreateShortcut(sLinkFile) >> "%TEMP%\CreateShortcut.vbs"
echo oLink.TargetPath = "!INSTALL_DIR!\Launch ShredX.bat" >> "%TEMP%\CreateShortcut.vbs"
echo oLink.WorkingDirectory = "!INSTALL_DIR!" >> "%TEMP%\CreateShortcut.vbs"
echo oLink.Description = "ShredX - Professional Disk Sanitization Tool" >> "%TEMP%\CreateShortcut.vbs"
echo oLink.IconLocation = "!INSTALL_DIR!\ShredX.exe,0" >> "%TEMP%\CreateShortcut.vbs"
echo oLink.Save >> "%TEMP%\CreateShortcut.vbs"
cscript /nologo "%TEMP%\CreateShortcut.vbs" 2>nul
del "%TEMP%\CreateShortcut.vbs" 2>nul

:: Create Start Menu entry
echo [INFO] Creating Start Menu entry...
set "STARTMENU=%ProgramData%\Microsoft\Windows\Start Menu\Programs"
if "%SHORTCUT_TYPE%"=="user" set "STARTMENU=%APPDATA%\Microsoft\Windows\Start Menu\Programs"

if not exist "%STARTMENU%\ShredX" mkdir "%STARTMENU%\ShredX" 2>nul

echo Set oWS = WScript.CreateObject("WScript.Shell") > "%TEMP%\CreateStartMenu.vbs"
echo sLinkFile = "%STARTMENU%\ShredX\ShredX.lnk" >> "%TEMP%\CreateStartMenu.vbs"
echo Set oLink = oWS.CreateShortcut(sLinkFile) >> "%TEMP%\CreateStartMenu.vbs"
echo oLink.TargetPath = "!INSTALL_DIR!\Launch ShredX.bat" >> "%TEMP%\CreateStartMenu.vbs"
echo oLink.WorkingDirectory = "!INSTALL_DIR!" >> "%TEMP%\CreateStartMenu.vbs"
echo oLink.Description = "ShredX - Professional Disk Sanitization Tool" >> "%TEMP%\CreateStartMenu.vbs"
echo oLink.IconLocation = "!INSTALL_DIR!\ShredX.exe,0" >> "%TEMP%\CreateStartMenu.vbs"
echo oLink.Save >> "%TEMP%\CreateStartMenu.vbs"
cscript /nologo "%TEMP%\CreateStartMenu.vbs" 2>nul
del "%TEMP%\CreateStartMenu.vbs" 2>nul

:: Add uninstaller
echo [INFO] Creating uninstaller...
(
echo @echo off
echo setlocal
echo.
echo echo ============================================================
echo echo                ShredX Uninstaller
echo echo ============================================================
echo echo.
echo.
echo set /p confirm="Are you sure you want to uninstall ShredX? (y/N): "
echo if /i not "%%confirm%%"=="y" ^(
echo     echo Uninstall cancelled.
echo     pause
echo     exit /b 0
echo ^)
echo.
echo echo Removing ShredX...
echo.
echo del /Q "%DESKTOP%\ShredX.lnk" 2^>nul
echo del /Q "%PUBLIC%\Desktop\ShredX.lnk" 2^>nul
echo rmdir /S /Q "%STARTMENU%\ShredX" 2^>nul
echo rmdir /S /Q "%APPDATA%\Microsoft\Windows\Start Menu\Programs\ShredX" 2^>nul
echo.
echo echo Removing installation directory...
echo echo Note: User data and reports will be preserved in a backup folder
echo.
echo if exist "!INSTALL_DIR!\reports" ^(
echo     if not exist "%USERPROFILE%\ShredX-Backup" mkdir "%USERPROFILE%\ShredX-Backup"
echo     xcopy /E /Y "!INSTALL_DIR!\reports" "%USERPROFILE%\ShredX-Backup\reports\" ^>nul
echo     xcopy /E /Y "!INSTALL_DIR!\certificates" "%USERPROFILE%\ShredX-Backup\certificates\" ^>nul
echo     if exist "!INSTALL_DIR!\users.json" copy "!INSTALL_DIR!\users.json" "%USERPROFILE%\ShredX-Backup\" ^>nul
echo     echo User data backed up to: %%USERPROFILE%%\ShredX-Backup
echo ^)
echo.
echo rmdir /S /Q "!INSTALL_DIR!" 2^>nul
echo.
echo echo ============================================================
echo echo              ShredX Uninstalled Successfully
echo echo ============================================================
echo echo.
echo echo Your reports and certificates have been backed up to:
echo echo %%USERPROFILE%%\ShredX-Backup
echo echo.
echo pause
) > "!INSTALL_DIR!\Uninstall ShredX.bat"

:: Set file permissions for security
echo [INFO] Setting file permissions...
icacls "!INSTALL_DIR!" /grant:r Users:(RX) >nul 2>&1
icacls "!INSTALL_DIR!\ShredX.exe" /grant:r Users:(RX) >nul 2>&1

:: Create quick access batch file in installation directory
(
echo @echo off
echo echo ShredX Quick Actions
echo echo.
echo echo 1. Launch ShredX
echo echo 2. Open Reports Folder
echo echo 3. Open Installation Directory
echo echo 4. View Documentation
echo echo 5. Uninstall ShredX
echo echo 6. Exit
echo echo.
echo set /p choice="Choose an option (1-6): "
echo.
echo if "%%choice%%"=="1" start "" "!INSTALL_DIR!\Launch ShredX.bat"
echo if "%%choice%%"=="2" start "" "!INSTALL_DIR!\reports"
echo if "%%choice%%"=="3" start "" "!INSTALL_DIR!"
echo if "%%choice%%"=="4" if exist "!INSTALL_DIR!\README.txt" start "" "!INSTALL_DIR!\README.txt"
echo if "%%choice%%"=="5" start "" "!INSTALL_DIR!\Uninstall ShredX.bat"
echo if "%%choice%%"=="6" exit /b 0
) > "!INSTALL_DIR!\ShredX Menu.bat"

:: Final verification
echo [INFO] Verifying installation...
if exist "!INSTALL_DIR!\ShredX.exe" (
    echo [SUCCESS] ShredX executable installed
) else (
    echo [ERROR] Installation verification failed
    pause
    exit /b 1
)

:: Display completion message
echo.
echo ============================================================
echo                Installation Complete!
echo ============================================================
echo.
echo ShredX has been successfully installed to:
echo !INSTALL_DIR!
echo.
echo You can now launch ShredX using:
echo   • Desktop shortcut: "ShredX"
echo   • Start Menu: All Programs ^> ShredX ^> ShredX
echo   • Quick launcher: "!INSTALL_DIR!\Launch ShredX.bat"
echo   • Menu system: "!INSTALL_DIR!\ShredX Menu.bat"
echo.
echo ============================================================
echo                   IMPORTANT NOTES
echo ============================================================
echo.
echo 1. ADMINISTRATOR PRIVILEGES: ShredX requires Administrator
echo    privileges for disk-level operations. You will see UAC
echo    prompts when performing sanitization operations.
echo.
echo 2. FIRST RUN: On first launch, ShredX will create a default
echo    user configuration. You can set up additional users
echo    through the application interface.
echo.
echo 3. DATA LOCATION: Reports and certificates are stored in:
echo    !INSTALL_DIR!\reports\
echo    !INSTALL_DIR!\certificates\
echo.
echo 4. UNINSTALL: To remove ShredX, run:
echo    !INSTALL_DIR!\Uninstall ShredX.bat
echo.
echo 5. DOCUMENTATION: Complete user guide available at:
echo    !INSTALL_DIR!\README.txt
echo.
echo ============================================================
echo.
echo Would you like to launch ShredX now? (y/N): 
set /p launch_now=""
if /i "%launch_now%"=="y" (
    echo.
    echo Launching ShredX...
    start "" "!INSTALL_DIR!\Launch ShredX.bat"
)
echo.
echo Installation completed successfully!
echo Thank you for choosing ShredX for your data sanitization needs.
echo.
pause