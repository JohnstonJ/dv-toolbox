# Check status of a downloaded file given the stored ETag header.

param (
    [Parameter(Mandatory)]
    [string]$Uri,

    [Parameter(Mandatory)]
    [string]$OutFile
)

Set-StrictMode -Version 3.0
$ErrorActionPreference = "Stop"

if (! (Test-Path "$OutFile.etag" -PathType Leaf) `
        -or ! (Test-Path $OutFile -PathType Leaf)) {
    exit 1
}
$Resp = Invoke-WebRequest -Uri $Uri -Method Head
$ETag = $Resp.Headers.ETag
if ($ETag -ne (Get-Content "$OutFile.etag")) {
    exit 1
}
