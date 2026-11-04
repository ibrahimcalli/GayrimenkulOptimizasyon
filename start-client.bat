@echo off
chcp 65001 >nul
title Gayrimenkul360 - Danisman Programi
cd /d "%~dp0"

if exist "target\release\gayrimenkul-client.exe" (
  "target\release\gayrimenkul-client.exe"
) else if exist "target\debug\gayrimenkul-client.exe" (
  "target\debug\gayrimenkul-client.exe"
) else (
  echo Program bulunamadi. Once build-release.bat calistirin.
  pause
)
