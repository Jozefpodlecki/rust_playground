$WorkspacePath = "$env:GITHUB_WORKSPACE/crates"
$Projects = Get-ChildItem -Path $WorkspacePath -Directory | Where-Object { Test-Path "$($_.FullName)/Cargo.toml" }

foreach ($project in $projects) {
    Write-Host "Building $($project.Name)..."
    Push-Location $project.FullName
    cargo clean
    Pop-Location
}