# SHARKCODE Windows packaging. Run on the maintainer's PC, NEVER on the player's PC.
# Downloads the official embeddable CPython *during packaging*, not at game startup.
# Produced package runs without Python, Docker, WSL2 or Rust on the player's PC.
# SECURITY: isolated execution is not finished; release inspector is fail-closed.
[CmdletBinding()]
param(
    [string]$PythonVersion = '3.13.17',
    [string]$OutputZip = 'SHARKCODE_0_2_41_windows.zip'
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$root = $PSScriptRoot
Set-Location $root

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw 'Cargo is required to package SHARKCODE. Players do NOT need Cargo.'
}
Write-Host '[SHARKCODE] Compiling Rust release...'
& cargo build --release
if ($LASTEXITCODE -ne 0) { throw 'Rust compilation failed' }

$buildRoot = Join-Path $root 'package_tmp'
$packageRoot = Join-Path $buildRoot 'SHARKCODE'
$pythonFolder = Join-Path $packageRoot 'runtime\python'
if (Test-Path $buildRoot) { Remove-Item -Recurse -Force $buildRoot }
New-Item -ItemType Directory -Force -Path $pythonFolder | Out-Null

$pythonZip = Join-Path $buildRoot 'python-embed.zip'
$download = "https://www.python.org/ftp/python/$PythonVersion/python-$PythonVersion-embed-amd64.zip"
Write-Host "[SHARKCODE] Downloading official embedded CPython $PythonVersion for packaging..."
Invoke-WebRequest -Uri $download -OutFile $pythonZip -UseBasicParsing
Expand-Archive -LiteralPath $pythonZip -DestinationPath $pythonFolder -Force
Remove-Item $pythonZip

# Protect isolated Python stdlib lookup: the Python-supplied ._pth must be present.
$pth = @(Get-ChildItem -LiteralPath $pythonFolder -Filter 'python*._pth' -File)
if ($pth.Count -ne 1) { throw 'Unexpected embedded Python layout: missing/duplicate ._pth file' }
$pthText = Get-Content -LiteralPath $pth[0].FullName -Raw
if ($pthText -notmatch 'python313\.zip') { throw 'Unexpected CPython build: expected python313.zip' }
if ($pthText -match '(?m)^\s*import site\s*$') { throw 'Embedded runtime must NOT enable site-packages' }

$pythonExe = Join-Path $pythonFolder 'python.exe'
if (-not (Test-Path $pythonExe)) { throw 'python.exe missing in embedded ZIP' }
& $pythonExe -I -c 'import sys; print(sys.version); print(sys.flags.isolated)'
if ($LASTEXITCODE -ne 0) { throw 'Embedded interpreter smoke test failed' }

Copy-Item -LiteralPath (Join-Path $root 'target\release\sharkcode.exe') -Destination (Join-Path $packageRoot 'SHARKCODE.exe')
Copy-Item -LiteralPath (Join-Path $root 'content') -Destination (Join-Path $packageRoot 'content') -Recurse
Copy-Item -LiteralPath (Join-Path $root 'runtime\inspector.py') -Destination (Join-Path $packageRoot 'runtime\inspector.py')
Copy-Item -LiteralPath (Join-Path $root 'runtime\quality.py') -Destination (Join-Path $packageRoot 'runtime\quality.py')
Copy-Item -LiteralPath (Join-Path $root 'runtime\project_validator.py') -Destination (Join-Path $packageRoot 'runtime\project_validator.py')

# Full-process smoke test: -I plus bundled inspector and sibling quality.py.
# This catches missing/invisible modules that ordinary Python unit tests miss.
$inspectorPath = Join-Path $packageRoot 'runtime\inspector.py'
$smokeRequest = '{"task":"shyguy","source":"print(\"Hello world!\")\n","tests":[{"inputs":[]}]}'
$smokeOutput = $smokeRequest | & $pythonExe -X utf8 -I $inspectorPath
if ($LASTEXITCODE -ne 0) { throw 'Bundled inspector failed in -I mode' }
$smokeResult = $smokeOutput | ConvertFrom-Json
if (-not $smokeResult.evaluation.passed) { throw 'Bundled inspector smoke test did not pass' }
Write-Host '[SHARKCODE] Isolated-path inspector import test passed.'


$manifest = foreach ($file in (Get-ChildItem -LiteralPath $packageRoot -File -Recurse)) {
    $relative = $file.FullName.Substring($packageRoot.Length + 1).Replace('\','/')
    [pscustomobject]@{ file=$relative; sha256=(Get-FileHash $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant() }
}
$manifest | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $packageRoot 'SHA256_MANIFEST.json') -Encoding UTF8
$destination = Join-Path $root $OutputZip
if (Test-Path $destination) { Remove-Item -Force $destination }
Compress-Archive -LiteralPath $packageRoot -DestinationPath $destination -CompressionLevel Optimal
Write-Host "[SHARKCODE] Packaged: $destination"
Write-Host 'Warning: current Windows native worker is NOT security isolated. This package is for development validation only.'
