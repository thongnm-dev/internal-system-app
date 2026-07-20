# Usage: .\scripts\bump-version.ps1 <version>
# Example: .\scripts\bump-version.ps1 1.2.0
#
# Updates version in package.json, tauri.conf.json, Cargo.toml,
# commits the change, and creates a git tag.

param(
    [Parameter(Mandatory=$true, Position=0)]
    [string]$Version
)

$ErrorActionPreference = "Stop"

if ($Version -notmatch '^\d+\.\d+\.\d+$') {
    Write-Error "Version must be in semver format (e.g., 1.2.0)"
    exit 1
}

$RootDir = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)

# package.json
$pkgPath = Join-Path $RootDir "package.json"
$pkg = Get-Content $pkgPath -Raw -Encoding utf8
$pkg = $pkg -replace '"version":\s*"[^"]*"', "`"version`": `"$Version`""
[System.IO.File]::WriteAllText($pkgPath, $pkg, [System.Text.UTF8Encoding]::new($false))

# tauri.conf.json
$tauriPath = Join-Path $RootDir "src-tauri\tauri.conf.json"
$tauri = Get-Content $tauriPath -Raw -Encoding utf8
$tauri = $tauri -replace '"version":\s*"[^"]*"', "`"version`": `"$Version`""
[System.IO.File]::WriteAllText($tauriPath, $tauri, [System.Text.UTF8Encoding]::new($false))

# Cargo.toml — only the [package] version, not dependency versions
$cargoPath = Join-Path $RootDir "src-tauri\Cargo.toml"
$cargo = Get-Content $cargoPath -Raw -Encoding utf8
$regex = [regex]::new('(?m)^version = "[^"]*"')
$cargo = $regex.Replace($cargo, "version = `"$Version`"", 1)
[System.IO.File]::WriteAllText($cargoPath, $cargo, [System.Text.UTF8Encoding]::new($false))

Write-Host "Updated version to $Version in:"
Write-Host "  - package.json"
Write-Host "  - src-tauri\tauri.conf.json"
Write-Host "  - src-tauri\Cargo.toml"

# Commit and tag
git add $pkgPath $tauriPath $cargoPath
git commit -m "chore: bump version to $Version"
git tag "v$Version"

Write-Host ""
Write-Host "Committed and tagged v$Version"
Write-Host "Run 'git push; git push --tags' to publish."
