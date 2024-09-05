# Download a file, as well as associated ETag header.

param (
    [Parameter(Mandatory)]
    [string]$Uri,

    [Parameter(Mandatory)]
    [string]$OutFile
)

Set-StrictMode -Version 3.0
$ErrorActionPreference = "Stop"

$Resp = Invoke-WebRequest -Uri $Uri -Method Head
$ETag = $Resp.Headers.ETag
Invoke-WebRequest -Uri $Uri -OutFile "$OutFile.tmp"
Move-Item "$OutFile.tmp" $OutFile -Force
$ETag | Out-File "$OutFile.etag"
