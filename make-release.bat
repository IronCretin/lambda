@echo off

rem Builds and packages a release folder on windows. Requires cargo.

if exist lambda-%1 del lambda-%1 /Q
cargo build --release
mkdir lambda-%1
copy README.md lambda-%1\readme.md
copy target\debug\lambda.exe lambda-%1\lambda.exe
xcopy samples lambda-%1\samples /Q /I /Y