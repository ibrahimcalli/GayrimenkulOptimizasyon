@echo off
chcp 65001 >nul
title Gayrimenkul360 - Ofis Sunucusu
cd /d "%~dp0"

if exist "target\release\gayrimenkul-server.exe" (
  echo Sunucu baslatiliyor... Bu pencereyi acik tutun.
  "target\release\gayrimenkul-server.exe"
) else if exist "target\debug\gayrimenkul-server.exe" (
  echo Sunucu baslatiliyor... Bu pencereyi acik tutun.
  "target\debug\gayrimenkul-server.exe"
) else (
  echo Sunucu bulunamadi. Once build-release.bat calistirin.
)
pause
