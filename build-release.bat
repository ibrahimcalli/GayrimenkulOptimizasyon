@echo off
chcp 65001 >nul
title Gayrimenkul360 Derleme
cd /d "%~dp0"
echo === Gayrimenkul360 derleniyor (release) ===
cargo build --release
echo.
echo Derleme tamamlandi. Olusan dosyalar:
echo   target\release\gayrimenkul-server.exe
echo   target\release\gayrimenkul-client.exe
pause
