# Build script para RS-CPU
# Compila em modo release e copia o executavel diretamente para a raiz do projeto.

param(
    [switch]$Debug
)

$target = if ($Debug) { "debug" } else { "release" }
$args = if ($Debug) { @("build") } else { @("build", "--release") }

Write-Host "==> Compilando RS-CPU ($target)..." -ForegroundColor Cyan
& cargo @args

if ($LASTEXITCODE -eq 0) {
    $srcPath = "target\$target\rs-cpu.exe"
    if (Test-Path $srcPath) {
        Copy-Item $srcPath .\rs-cpu.exe -Force
        Write-Host "==> [SUCESSO] Binario atualizado na raiz: .\rs-cpu.exe" -ForegroundColor Green
    }
} else {
    Write-Host "==> [ERRO] Falha na compilacao." -ForegroundColor Red
    exit $LASTEXITCODE
}
