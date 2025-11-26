@echo off
chcp 65001 >nul

echo ========================================
echo Building zml and mcp-any-rest executables
echo ========================================

REM Build zml executable
echo Building zml executable...
cargo build --release --bin zml
if %errorlevel% neq 0 (
    echo Error: Failed to build zml executable
    exit /b 1
)

REM Build mcp-any-rest executable
echo Building mcp-any-rest executable...
cargo build --release --bin mcp-any-rest
if %errorlevel% neq 0 (
    echo Error: Failed to build mcp-any-rest executable
    exit /b 1
)

REM Create package directory if it doesn't exist
if not exist "package" mkdir package
if not exist "package\config" mkdir package\config
if not exist "package\config\presets" mkdir package\config\presets
if not exist "package\config\zml" mkdir package\config\zml

REM Copy executables to package directory
echo Copying executables to package directory...
copy "target\release\zml.exe" "package\zml.exe" >nul
copy "target\release\mcp-any-rest.exe" "package\mcp-any-rest.exe" >nul

REM Copy configuration files
echo Copying configuration files...
copy "config\config.json" "package\config\config.json" >nul
copy "config\modules.json" "package\config\modules.json" >nul
copy "config\mcp-stdio-example.json" "package\config\mcp-stdio-example.json" >nul

REM Copy ZML files
echo Copying ZML files...
for %%f in (config\zml\*.zml) do (
    copy "%%f" "package\config\zml\" >nul
)

REM Copy preset files
echo Copying preset files...
for %%f in (config\presets\*.json) do (
    copy "%%f" "package\config\presets\" >nul
)

echo ========================================
echo Build and copy completed successfully!
echo ========================================
echo.
echo Executables copied to package directory:
echo - package\zml.exe
echo - package\mcp-any-rest.exe
echo.
echo Configuration files copied to package\config\
echo.
echo Ready to use!
pause