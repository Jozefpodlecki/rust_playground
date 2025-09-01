param(
    [string]$TomlPath = "buildtest/Cargo.toml",
    [string]$DepName = "mscp",
    [string]$FeatureName = "some-feat"
)

$lines = Get-Content $TomlPath

$processedLines = $lines | ForEach-Object {
    # Remove dependency line
    if ($_ -match "^\s*$DepName\s*=" -or $_ -match "^\s*$FeatureName\s*=") {
        return
    }

    # Replace default feature line
    if ($_ -match "^\s*default\s*=\s*\[`"$FeatureName`"\]") {
        "default = []"
    } else {
        $_
    }
}

$processedLines | Set-Content $TomlPath