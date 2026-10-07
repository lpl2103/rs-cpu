@echo off
setlocal
echo ==> Compilando RS-CPU (release)...
cargo build --release
if %ERRORLEVEL% EQU 0 (
    copy /Y target\release\rs-cpu.exe rs-cpu.exe >nul
    echo ==> [SUCESSO] Binario atualizado na raiz: rs-cpu.exe
) else (
    echo ==> [ERRO] Falha na compilacao.
    exit /b %ERRORLEVEL%
)
endlocal
