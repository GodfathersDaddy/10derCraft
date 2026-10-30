@echo off
setlocal
set "STUDIO_ROOT=%~dp0"
set "STUDIO_SOURCE=%STUDIO_ROOT%solidcraft"
if exist "%STUDIO_ROOT%Cargo.toml" set "STUDIO_SOURCE=%STUDIO_ROOT%"
if not defined SOLIDCRAFT_SIMULATION_PYTHON if exist "%STUDIO_ROOT%simulation\.venv\Scripts\python.exe" set "SOLIDCRAFT_SIMULATION_PYTHON=%STUDIO_ROOT%simulation\.venv\Scripts\python.exe"
set "STUDIO_IMAGE=solidcraft.exe"
if exist "%STUDIO_SOURCE%\target\native-ui\ci\launch-target.txt" (
    set /p "STUDIO_IMAGE="<"%STUDIO_SOURCE%\target\native-ui\ci\launch-target.txt"
)
set "STUDIO_EXE=%STUDIO_SOURCE%\target\native-ui\ci\%STUDIO_IMAGE%"
if exist "%STUDIO_EXE%" (
    start "10derCraft" "%STUDIO_EXE%" %*
    exit /b 0
)
echo 10derCraft ist noch nicht gebaut. Bitte Build-10derCraft.ps1 ausfuehren.
pause
exit /b 1
