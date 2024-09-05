# This script downloads Task to a project-specific .tools/ directory, and then runs it.

param (
    [string]$Task
)

# Make this script more robust:

$MinimumPowerShellVersion = "7.4.5"

if ($PSVersionTable.PSVersion -lt [Version]$MinimumPowerShellVersion) {
    Write-Error ("This script requires PowerShell $MinimumPowerShellVersion or higher. " + `
            "Current version: $($PSVersionTable.PSVersion)")
    exit 1
}

Set-StrictMode -Version 3.0

$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true
$PSNativeCommandArgumentPassing = 'Standard'

# For now, we only run on Windows.
if (!$IsWindows) {
    Write-Error "This script only runs on Windows."
    exit 1
}

# Create tools directory
$Repo = Split-Path $PSScriptRoot -Parent -Resolve
$ToolsDir = New-Item -Path $Repo -Name ".tools" -ItemType Directory -Force

# Download and set up Task

$TaskVersion = "3.38.0"

$TaskURI = "https://github.com/go-task/task/releases/download/v$TaskVersion/task_windows_amd64.zip"
$TaskZIP = Join-Path $ToolsDir "task_v$($TaskVersion)_windows_amd64.zip"
$TaskDir = Join-Path $ToolsDir "task"

Write-Host "Downloading $TaskURI..."
Invoke-WebRequest -URI $TaskURI -OutFile "$TaskZIP.tmp"
Move-Item "$TaskZIP.tmp" "$TaskZIP" -Force

if (Test-Path $TaskDir) {
    Write-Host "Deleting $TaskDir..."
    Remove-Item $TaskDir -Recurse
}
Expand-Archive $TaskZIP $TaskDir

# Run default or specified task
Set-Location $Repo
if ($Task) {
	& (Join-Path ".tools" "task" "task") $Task
} else {
	& (Join-Path ".tools" "task" "task")
}
