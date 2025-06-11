@echo off
setlocal enabledelayedexpansion

REM Build script for creating release binaries on Windows
REM Usage: scripts\build-release.bat [version]

set VERSION=%1
if "%VERSION%"=="" set VERSION=dev

set BUILD_DIR=dist
set TARGETS=x86_64-pc-windows-msvc aarch64-pc-windows-msvc x86_64-unknown-linux-gnu x86_64-unknown-linux-musl

echo 🚀 Building Hitman v%VERSION% for Windows platforms...

REM Clean previous builds
if exist %BUILD_DIR% rmdir /s /q %BUILD_DIR%
mkdir %BUILD_DIR%

REM Check if cross is installed
cross --version >nul 2>&1
if errorlevel 1 (
    echo 📦 Installing cross...
    cargo install cross
)

REM Build for each target
for %%t in (%TARGETS%) do (
    echo 🔨 Building for %%t...
    
    if "%%t"=="x86_64-pc-windows-msvc" (
        cargo build --release --target %%t
    ) else if "%%t"=="aarch64-pc-windows-msvc" (
        cargo build --release --target %%t
    ) else (
        cross build --release --target %%t
    )
    
    REM Package the binary
    if "%%t"=="x86_64-pc-windows-msvc" (
        set ARCHIVE_NAME=hitman-%VERSION%-%%t.zip
        cd target\%%t\release
        powershell Compress-Archive -Path hitman.exe -DestinationPath ..\..\..\%BUILD_DIR%\!ARCHIVE_NAME!
        cd ..\..\..
    ) else if "%%t"=="aarch64-pc-windows-msvc" (
        set ARCHIVE_NAME=hitman-%VERSION%-%%t.zip
        cd target\%%t\release
        powershell Compress-Archive -Path hitman.exe -DestinationPath ..\..\..\%BUILD_DIR%\!ARCHIVE_NAME!
        cd ..\..\..
    ) else (
        set ARCHIVE_NAME=hitman-%VERSION%-%%t.tar.gz
        cd target\%%t\release
        tar czf ..\..\..\%BUILD_DIR%\!ARCHIVE_NAME! hitman
        cd ..\..\..
    )
    
    echo ✅ Created !ARCHIVE_NAME!
)

REM Generate checksums
echo 🔐 Generating checksums...
cd %BUILD_DIR%
powershell "Get-ChildItem *.zip, *.tar.gz | ForEach-Object { (Get-FileHash $_.Name -Algorithm SHA256).Hash + '  ' + $_.Name } | Out-File -Encoding ASCII checksums.txt"
cd ..

echo 🎉 Build complete! Artifacts are in %BUILD_DIR%\
dir %BUILD_DIR%
