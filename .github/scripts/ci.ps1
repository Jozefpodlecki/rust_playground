$WorkspacePath = "$env:GITHUB_WORKSPACE/projects"
$Projects = Get-ChildItem -Path $WorkspacePath -Directory | Where-Object { Test-Path "$($_.FullName)/Cargo.toml" }

foreach ($project in $projects) {
    Write-Host "Building $($project.Name)..."
    Push-Location $project.FullName
    cargo build --release --quiet
    Pop-Location
}