param(
    [Parameter(Mandatory = $true)][string]$Installer,
    [Parameter(Mandatory = $true)][string]$Version
)
$ErrorActionPreference = 'Stop'
if ($env:GITHUB_ACTIONS -ne 'true') { throw 'Run this install/uninstall test only on a disposable GitHub Windows runner' }
if ($Version -notmatch '^\d+\.\d+\.\d+(-beta\.\d+)?$') { throw 'Invalid test version' }
$installPath = [System.IO.Path]::GetFullPath((Join-Path $env:LOCALAPPDATA "Programs/TimestampCleaner/$Version"))
$registryPath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\TimestampCleaner-$Version"
$shortcut = Join-Path ([Environment]::GetFolderPath('Programs')) "Timestamp Cleaner $Version.lnk"
if ((Test-Path -LiteralPath $installPath) -or (Test-Path -LiteralPath $registryPath) -or (Test-Path -LiteralPath $shortcut)) {
    throw 'Test refuses to modify a pre-existing installation, registry entry or shortcut'
}
$installerPath = (Resolve-Path -LiteralPath $Installer).Path
$process = Start-Process -FilePath $installerPath -ArgumentList '/S' -PassThru -Wait -WindowStyle Hidden
if ($process.ExitCode -ne 0) { throw "Silent setup failed: $($process.ExitCode)" }
$manifest = Get-Content -LiteralPath (Join-Path $installPath 'file-manifest.json') -Raw | ConvertFrom-Json
foreach ($property in $manifest.PSObject.Properties) {
    $filePath = Join-Path $installPath $property.Name
    if ((Get-FileHash -LiteralPath $filePath -Algorithm SHA256).Hash.ToLowerInvariant() -ne $property.Value) {
        throw "Installed payload hash mismatch: $($property.Name)"
    }
}
$entry = Get-ItemProperty -LiteralPath $registryPath
if ($entry.DisplayVersion -ne $Version -or $entry.InstallLocation -ne $installPath) { throw 'Uninstall registry data mismatch' }
if (-not (Test-Path -LiteralPath $shortcut -PathType Leaf)) { throw 'Start shortcut missing' }
foreach ($name in @('NSIS-COPYING.txt', 'SETUP.md', 'build-info.json')) {
    if (-not (Test-Path -LiteralPath (Join-Path $installPath "installer/$name"))) { throw "Missing installer notice: $name" }
}
# A same-version reinstall must refuse the existing directory without changing it.
$sentinel = Join-Path $installPath 'personal-file.keep'
$nestedSentinel = Join-Path $installPath 'installer/personal-file.keep'
Set-Content -LiteralPath $sentinel -Value 'preserve user files'
Set-Content -LiteralPath $nestedSentinel -Value 'preserve nested user files'
$collision = Start-Process -FilePath $installerPath -ArgumentList '/S' -PassThru -Wait -WindowStyle Hidden
if ($collision.ExitCode -eq 0) { throw 'Same-version collision was accepted' }
$uninstaller = Join-Path $installPath 'uninstall.exe'
Start-Process -FilePath $uninstaller -ArgumentList '/S' -Wait -WindowStyle Hidden
$deadline = (Get-Date).AddSeconds(30)
while ((Test-Path -LiteralPath $uninstaller) -and (Get-Date) -lt $deadline) { Start-Sleep -Milliseconds 200 }
if ((Test-Path -LiteralPath $uninstaller) -or (Test-Path -LiteralPath $registryPath) -or (Test-Path -LiteralPath $shortcut)) {
    throw 'Uninstall did not remove its executable, registration and shortcut'
}
foreach ($property in $manifest.PSObject.Properties) {
    if (Test-Path -LiteralPath (Join-Path $installPath $property.Name)) { throw "Packaged file survived uninstall: $($property.Name)" }
}
if ((Get-Content -LiteralPath $sentinel) -ne 'preserve user files' -or
    (Get-Content -LiteralPath $nestedSentinel) -ne 'preserve nested user files') { throw 'Uninstall removed or changed user files' }
# Remove only the exact sentinels created above and their now-empty test directories.
Remove-Item -LiteralPath $sentinel, $nestedSentinel
[System.IO.Directory]::Delete((Join-Path $installPath 'installer'), $false)
[System.IO.Directory]::Delete($installPath, $false)
Write-Host 'Silent install, payload hashes, collision rejection, uninstall and user-file preservation passed.'
