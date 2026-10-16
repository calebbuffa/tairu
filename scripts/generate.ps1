param(
    [string]$SchemaGenDir = "",
    [string]$SchemaDir = "",
    [string]$OutputDir = "",
    [switch]$Check
)

$ErrorActionPreference = "Stop"

# Get script directory and tairu root
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$Root = Split-Path -Parent $ScriptDir

# Defaults relative to tairu root
if (-not $SchemaGenDir) {
    $SchemaGenDir = Join-Path $Root "xtask"
}
if (-not $SchemaDir) {
    $SchemaDir = Join-Path $Root "extern/3d-tiles/specification/schema"
}
if (-not $OutputDir) {
    $OutputDir = Join-Path $Root "src"
}

# Resolve paths. Output may not exist yet.
$SchemaGenDir = (Resolve-Path $SchemaGenDir -ErrorAction Stop).Path
$SchemaDir = (Resolve-Path $SchemaDir -ErrorAction Stop).Path
$OutputDir = if ([System.IO.Path]::IsPathRooted($OutputDir)) {
    [System.IO.Path]::GetFullPath($OutputDir)
}
else {
    [System.IO.Path]::GetFullPath((Join-Path $Root $OutputDir))
}
$OutputFile = Join-Path $OutputDir "generated.rs"
$ManifestOutput = Join-Path $OutputDir "MANIFEST.md"

$ManifestFile = Join-Path $SchemaGenDir "Cargo.toml"

if (-not (Test-Path $ManifestFile)) {
    Write-Error "xtask Cargo manifest not found: $ManifestFile"
}

Write-Host "Regenerating 3D Tiles types..." -ForegroundColor Cyan
Write-Host "  Schemas: $SchemaDir"
Write-Host "  Output:  $OutputFile"
Write-Host ""

$arguments = @(
    "--schema", $SchemaDir,
    "--output", $OutputFile,
    "--manifest", $ManifestOutput,
    "--module-doc", "Generated 3D Tiles data model. Do not edit."
)

if ($Check) {
    $arguments += "--check"
}

Write-Verbose "Running: cargo run --manifest-path $ManifestFile -- $($arguments -join ' ')"

& cargo run --manifest-path $ManifestFile -- @arguments

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    if ($Check) {
        Write-Host "3D Tiles types are up to date!" -ForegroundColor Green
    }
    else {
        Write-Host "3D Tiles types regenerated successfully!" -ForegroundColor Green
        Write-Host "  Output: $OutputFile"
        Write-Host "  Manifest: $ManifestOutput"
    }
}
else {
    Write-Error "Schema generation failed with exit code $LASTEXITCODE"
}