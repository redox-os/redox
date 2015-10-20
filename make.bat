@echo off

if "%~1"=="" (
  windows\make.exe
) else (
  windows\make.exe %*
)
