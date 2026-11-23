@echo off
rem Mo Flowsup trong cua so rieng (Edge che do app) khi khong dung Flowsup.exe. Khong tu luu ra file.
rem Chay "Flowsup.cmd shortcut" de tao loi tat co icon tren Desktop (tro toi Flowsup.exe).
setlocal
set "PAGE=%~dp0flowsup.html"
set "EDGE=%ProgramFiles(x86)%\Microsoft\Edge\Application\msedge.exe"
if not exist "%EDGE%" set "EDGE=%ProgramFiles%\Microsoft\Edge\Application\msedge.exe"
if not exist "%EDGE%" (echo Khong tim thay Microsoft Edge. & pause & exit /b 1)
if /i "%~1"=="shortcut" (
  powershell -NoProfile -Command "$s=(New-Object -ComObject WScript.Shell).CreateShortcut([Environment]::GetFolderPath('Desktop')+'\Flowsup.lnk');$s.TargetPath='%~dp0Flowsup.exe';$s.IconLocation='%~dp0Flowsup.exe,0';$s.WorkingDirectory='%~dp0';$s.Save()"
  echo Da tao loi tat Flowsup tren Desktop.
  exit /b 0
)
start "" "%EDGE%" --app="%PAGE%" --user-data-dir="%LOCALAPPDATA%\Flowsup\window" --no-first-run --no-default-browser-check --window-size=1440,920 --lang=vi