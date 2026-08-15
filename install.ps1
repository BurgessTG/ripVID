#
# ripVID Windows Installer
# Usage: irm https://raw.githubusercontent.com/honeycomb-Technologies/ripVID/main/install.ps1 | iex
#

$ErrorActionPreference = "Stop"

$VERSION = "2.3.0"
$REPO = "honeycomb-Technologies/ripVID"
$INSTALLER_URL = "https://github.com/$REPO/releases/download/v$VERSION/ripVID_${VERSION}_x64-setup.exe"
$TEMP_FILE = "$env:TEMP\ripVID-setup.exe"

Write-Host ""
Write-Host "╔═══════════════════════════════════════════╗" -ForegroundColor Blue
Write-Host "║           ripVID Installer v$VERSION          ║" -ForegroundColor Blue
Write-Host "║     Video Downloader for YouTube & X      ║" -ForegroundColor Blue
Write-Host "╚═══════════════════════════════════════════╝" -ForegroundColor Blue
Write-Host ""

Write-Host "Downloading ripVID v$VERSION..." -ForegroundColor Green

try {
    # Download installer
    Invoke-WebRequest -Uri $INSTALLER_URL -OutFile $TEMP_FILE -UseBasicParsing

    Write-Host "Download complete. Starting installer..." -ForegroundColor Green
    Write-Host ""

    # Run installer
    Start-Process -FilePath $TEMP_FILE -Wait

    # Clean up
    Remove-Item $TEMP_FILE -ErrorAction SilentlyContinue

    Write-Host ""
    Write-Host "✓ ripVID installed successfully!" -ForegroundColor Green
    Write-Host ""
    Write-Host "You can find ripVID in your Start Menu." -ForegroundColor Cyan
}
catch {
    Write-Host "Error: $_" -ForegroundColor Red
    Write-Host ""
    Write-Host "Manual download: $INSTALLER_URL" -ForegroundColor Yellow
    exit 1
}
