@echo off
setlocal

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0generate-latest.ps1"

pause