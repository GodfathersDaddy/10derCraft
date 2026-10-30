$ErrorActionPreference = 'Stop'
$projectRoot = $PSScriptRoot
$env:DOTNET_CLI_HOME = Join-Path $projectRoot 'external/.dotnet-home'
$env:NUGET_PACKAGES = Join-Path $projectRoot 'external/.nuget'
$env:DOTNET_SKIP_FIRST_TIME_EXPERIENCE = '1'
$env:DOTNET_CLI_TELEMETRY_OPTOUT = '1'
$calcpadSource = Join-Path $projectRoot 'external/CalcpadCE'
if (-not (Test-Path -LiteralPath (Join-Path $calcpadSource 'Calcpad.Core/Calcpad.Core.csproj'))) {
    & git clone https://github.com/imartincei/CalcpadCE.git $calcpadSource
    if ($LASTEXITCODE -ne 0) { throw 'CalcpadCE download failed.' }
    & git -C $calcpadSource checkout --detach 0cf438f7d4a78a7ee50724eefb712c23f2ec09c5
    if ($LASTEXITCODE -ne 0) { throw 'Pinned CalcpadCE revision unavailable.' }
}
$calcpadRevision = & git -C $calcpadSource rev-parse HEAD
if ($LASTEXITCODE -ne 0 -or $calcpadRevision -ne '0cf438f7d4a78a7ee50724eefb712c23f2ec09c5') { throw 'Unexpected CalcpadCE revision; keep the pinned source in external/CalcpadCE.' }
$dotnet = Join-Path $projectRoot 'external/dotnet/dotnet.exe'
if (-not (Test-Path -LiteralPath $dotnet)) {
    $dotnet = (Get-Command dotnet -ErrorAction Stop).Source
}
$coreFolder = Join-Path $projectRoot 'external/CalcpadCE/Calcpad.Core'
Copy-Item -LiteralPath (Join-Path $projectRoot 'integrations/calcpad-local/SolidCraftExports.cs') -Destination (Join-Path $coreFolder 'SolidCraftExports.cs')
$parserFile = Join-Path $coreFolder 'Parsers/ExpressionParser/ExpressionParser.cs'
$parserText = Get-Content -LiteralPath $parserFile -Raw -Encoding UTF8
if (-not $parserText.Contains('CaptureSolidCraftScalars();')) {
    $parserText = $parserText.Replace('_parser.ClearCache();', "CaptureSolidCraftScalars();`n                _parser.ClearCache();")
    Set-Content -LiteralPath $parserFile -Value $parserText -Encoding UTF8
}
& $dotnet publish (Join-Path $projectRoot 'integrations/calcpad-local/CalcpadLocal.csproj') -c Release --self-contained true -r win-x64 -o (Join-Path $projectRoot 'output/calcpad-local')
if ($LASTEXITCODE -ne 0) { throw 'Local Calcpad build failed.' }
Copy-Item -LiteralPath (Join-Path $projectRoot 'external/CalcpadCE/LICENSE') -Destination (Join-Path $projectRoot 'output/calcpad-local/CalcpadCE-LICENSE.txt')
Copy-Item -LiteralPath (Join-Path $projectRoot 'integrations/calcpad-local/README.md') -Destination (Join-Path $projectRoot 'output/calcpad-local/INTEGRATION.md')
