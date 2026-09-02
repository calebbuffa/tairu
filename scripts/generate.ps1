param(
    [string]$SchemaGenDir = "",
    [string]$SchemaFile = "",
    [string]$ConfigFile = "",
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
if (-not $SchemaFile) {
    $SchemaFile = Join-Path $Root "extern/3d-tiles/specification/schema/tileset.schema.json"
}
if (-not $ConfigFile) {
    $ConfigFile = Join-Path $Root "xtask/config.json"
}
if (-not $OutputDir) {
    $OutputDir = Join-Path $Root "src"
}

# Resolve paths. Output may not exist yet.
$SchemaGenDir = (Resolve-Path $SchemaGenDir -ErrorAction Stop).Path
$SchemaFile = (Resolve-Path $SchemaFile -ErrorAction Stop).Path
$ConfigFile = (Resolve-Path $ConfigFile -ErrorAction Stop).Path
$OutputDir = if ([System.IO.Path]::IsPathRooted($OutputDir)) {
    [System.IO.Path]::GetFullPath($OutputDir)
}
else {
    [System.IO.Path]::GetFullPath((Join-Path $Root $OutputDir))
}

$ManifestFile = Join-Path $SchemaGenDir "Cargo.toml"

if (-not (Test-Path $ManifestFile)) {
    Write-Error "xtask Cargo manifest not found: $ManifestFile"
}

if (-not (Test-Path $OutputDir)) {
    New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
}

Write-Host "Regenerating 3D Tiles types..." -ForegroundColor Cyan
Write-Host "  Schema: $SchemaFile"
Write-Host "  Config: $ConfigFile"
Write-Host "  Output: $OutputDir"
Write-Host ""

$arguments = @(
    "--schema", $SchemaFile,
    "--config", $ConfigFile,
    "--output", $OutputDir,
    "--manifest"
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
        Write-Host "  Output: $(Join-Path $OutputDir 'generated.rs')"
        Write-Host "  Manifest: $(Join-Path $OutputDir 'MANIFEST.md')"
    }
}
else {
    Write-Error "Schema generation failed with exit code $LASTEXITCODE"
}