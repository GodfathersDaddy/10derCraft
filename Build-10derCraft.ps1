$ErrorActionPreference = 'Stop'
$studioSource = Join-Path $PSScriptRoot 'solidcraft'
if (Test-Path -LiteralPath (Join-Path $PSScriptRoot 'Cargo.toml')) { $studioSource = $PSScriptRoot }
Push-Location -LiteralPath $studioSource
try {
    & cargo --config net.offline=false --config 'profile.ci.opt-level=2' --config 'profile.ci.debug=0' --config 'profile.ci.package.solidcraft-ui-egui.opt-level=0' build -p solidcraft -p solidcraft-cli --profile ci --locked -j 4 --target-dir target/native-ui
    if ($LASTEXITCODE -ne 0) { throw 'Der 10derCraft-Build ist fehlgeschlagen.' }
    $launchTarget = Join-Path $studioSource 'target/native-ui/ci/launch-target.txt'
    if (Test-Path -LiteralPath $launchTarget) { Remove-Item -LiteralPath $launchTarget -Force }
    Write-Host 'Fertig. Start-10derCraft.cmd startet 10derCraft. Modelle liegen unter Beispiele.'
}
finally { Pop-Location }
