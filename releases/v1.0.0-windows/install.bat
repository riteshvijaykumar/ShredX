@echo off
setlocal enabledelayedexpansion

:: ShredX Windows Installer
:: Installs the ShredX disk sanitization tool

echo ========================================
echo       ShredX Windows Installer
echo ========================================
echo.

:: Check for Administrator privileges
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo ERROR: Administrator privileges required!
    echo Please right-click and select "Run as administrator"
    echo.
    pause
    exit /b 1
)

echo Installing ShredX...

:: Create installation directory
set INSTALL_DIR=%ProgramFiles%\ShredX
if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"

:: Copy files
echo Copying application files...
copy /Y "hdd-tool.exe" "%INSTALL_DIR%\ShredX.exe" >nul
copy /Y "config.json" "%INSTALL_DIR%\" >nul
copy /Y "LICENSE" "%INSTALL_DIR%\" >nul
copy /Y "README.txt" "%INSTALL_DIR%\" >nul

:: Copy documentation
if exist "docs" (
    if not exist "%INSTALL_DIR%\docs" mkdir "%INSTALL_DIR%\docs"
    xcopy /Y /S "docs\*" "%INSTALL_DIR%\docs\" >nul
)

:: Create desktop shortcut
echo Creating desktop shortcut...
set DESKTOP=%USERPROFILE%\Desktop
echo Set oWS = WScript.CreateObject("WScript.Shell") > "%TEMP%\CreateShortcut.vbs"
echo sLinkFile = "%DESKTOP%\ShredX.lnk" >> "%TEMP%\CreateShortcut.vbs"
echo Set oLink = oWS.CreateShortcut(sLinkFile) >> "%TEMP%\CreateShortcut.vbs"
echo oLink.TargetPath = "%INSTALL_DIR%\ShredX.exe" >> "%TEMP%\CreateShortcut.vbs"
echo oLink.WorkingDirectory = "%INSTALL_DIR%" >> "%TEMP%\CreateShortcut.vbs"
echo oLink.Description = "ShredX - NIST SP 800-88 Disk Sanitization Tool" >> "%TEMP%\CreateShortcut.vbs"
echo oLink.Save >> "%TEMP%\CreateShortcut.vbs"
cscript /nologo "%TEMP%\CreateShortcut.vbs"
del "%TEMP%\CreateShortcut.vbs"

:: Create Start Menu shortcut
echo Creating Start Menu shortcut...
set STARTMENU=%ProgramData%\Microsoft\Windows\Start Menu\Programs
if not exist "%STARTMENU%\ShredX" mkdir "%STARTMENU%\ShredX"
echo Set oWS = WScript.CreateObject("WScript.Shell") > "%TEMP%\CreateStartMenu.vbs"
echo sLinkFile = "%STARTMENU%\ShredX\ShredX.lnk" >> "%TEMP%\CreateStartMenu.vbs"
echo Set oLink = oWS.CreateShortcut(sLinkFile) >> "%TEMP%\CreateStartMenu.vbs"
echo oLink.TargetPath = "%INSTALL_DIR%\ShredX.exe" >> "%TEMP%\CreateStartMenu.vbs"
echo oLink.WorkingDirectory = "%INSTALL_DIR%" >> "%TEMP%\CreateStartMenu.vbs"
echo oLink.Description = "ShredX - NIST SP 800-88 Disk Sanitization Tool" >> "%TEMP%\CreateStartMenu.vbs"
echo oLink.Save >> "%TEMP%\CreateStartMenu.vbs"
cscript /nologo "%TEMP%\CreateStartMenu.vbs"
del "%TEMP%\CreateStartMenu.vbs"

:: Add to PATH (optional)
echo Adding ShredX to system PATH...
for /f "tokens=2*" %%a in ('reg query "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment" /v PATH 2^>nul') do set CURRENT_PATH=%%b
echo !CURRENT_PATH! | findstr /i "%INSTALL_DIR%" >nul
if !errorlevel! neq 0 (
    reg add "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment" /v PATH /t REG_EXPAND_SZ /d "!CURRENT_PATH!;%INSTALL_DIR%" /f >nul
)

:: Create uninstaller
echo Creating uninstaller...
(
echo @echo off
echo setlocal
echo.
echo echo ========================================
echo echo       ShredX Uninstaller
echo echo ========================================
echo echo.
echo.
echo net session ^>nul 2^>^&1
echo if %%errorLevel%% neq 0 ^(
echo     echo ERROR: Administrator privileges required!
echo     echo Please right-click and select "Run as administrator"
echo     echo.
echo     pause
echo     exit /b 1
echo ^)
echo.
echo echo Removing ShredX...
echo.
echo del /Q "%DESKTOP%\ShredX.lnk" 2^>nul
echo rmdir /S /Q "%STARTMENU%\ShredX" 2^>nul
echo rmdir /S /Q "%INSTALL_DIR%" 2^>nul
echo.
echo echo ShredX has been successfully uninstalled.
echo echo.
echo pause
) > "%INSTALL_DIR%\uninstall.bat"

echo.
echo ========================================
echo      Installation Complete!
echo ========================================
echo.
echo ShredX has been successfully installed to:
echo %INSTALL_DIR%
echo.
echo You can now:
echo 1. Use desktop shortcut to launch ShredX
echo 2. Find ShredX in Start Menu
echo 3. Run 'ShredX' from command prompt
echo.
echo IMPORTANT: Always run ShredX as Administrator
echo for full disk sanitization capabilities.
echo.
echo To uninstall, run: %INSTALL_DIR%\uninstall.bat
echo.
pause