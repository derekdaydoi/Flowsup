@echo off
rem Tao Flowsup.exe tu launcher\ (Rust, bo GNU). Can: rustup + toolchain stable-x86_64-pc-windows-gnu.
rem Flowsup.exe phai nam canh flowsup.html. Icon: node tools\make-icons.mjs (tao flowsup.ico).
cd /d "%~dp0"
set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
cargo build --release --manifest-path launcher\Cargo.toml
if errorlevel 1 (echo BUILD LOI & exit /b 1)
copy /y launcher\target\release\flowsup.exe Flowsup.exe >nul
launcher\target\release\seticon.exe Flowsup.exe flowsup.ico
if errorlevel 1 (echo GAN ICON LOI & exit /b 1)
echo Da tao Flowsup.exe
