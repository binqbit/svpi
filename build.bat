@echo off
REM Build the project in release mode
cargo build --release

REM Copy the executable to the bin directory
xcopy /y .\target\release\svpi.exe .\bin\

REM Add the bin directory to the PATH environment variable
@REM setx PATH "%PATH%;%CD%\bin"

echo Build and setup completed.
