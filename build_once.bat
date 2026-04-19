@echo off
call "C:\Program Files\Microsoft Visual Studio\18\Community\VC\Auxiliary\Build\vcvars64.bat"
if errorlevel 1 (
  echo vcvars64.bat failed
  exit /b 1
)
rem Prepend MSVC x64 bin so link.exe resolves to MSVC, not Git's /usr/bin/link
set "PATH=C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.50.35717\bin\Hostx64\x64;C:\Program Files\nodejs;%USERPROFILE%\.cargo\bin;%PATH%"
echo === Using link.exe:
where link.exe
echo ===
npx tauri build
