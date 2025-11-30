# Check for admin rights at start
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if (-not $isAdmin) {
    Write-Host "Administrator rights required. Restarting..." -ForegroundColor Yellow
    Start-Process powershell -Verb RunAs -ArgumentList "-ExecutionPolicy Bypass -File `"$($MyInvocation.MyCommand.Path)`""
    exit
}

# Create user binary directory
$binPath = "$env:USERPROFILE\.local\bin"
New-Item -ItemType Directory -Force -Path $binPath

# Download v2parser
$v2repo = "house-of-vanity/v2-uri-parser"
$v2release = Invoke-RestMethod "https://api.github.com/repos/$v2repo/releases/latest"
$v2asset = $v2release.assets | Where-Object { $_.name -eq "v2parser-x86_64-pc-windows-msvc.zip" }
$v2zip = "$env:TEMP\v2parser.zip"
Invoke-WebRequest -Uri $v2asset.browser_download_url -OutFile $v2zip
Expand-Archive -Path $v2zip -DestinationPath $binPath -Force
Remove-Item $v2zip

# Download Xray-core
$xrayRepo = "XTLS/Xray-core"
$xrayRelease = Invoke-RestMethod "https://api.github.com/repos/$xrayRepo/releases/latest"
$xrayAsset = $xrayRelease.assets | Where-Object { $_.name -eq "Xray-windows-64.zip" }
$xrayZip = "$env:TEMP\xray.zip"
Invoke-WebRequest -Uri $xrayAsset.browser_download_url -OutFile $xrayZip
Expand-Archive -Path $xrayZip -DestinationPath $binPath -Force
Remove-Item $xrayZip

# Request server location
$serverLocation = Read-Host "Enter server location (e.g., US-NY, DE-Berlin, JP-Tokyo)"
$serverLocation = $serverLocation -replace '[^a-zA-Z0-9-]', '-'

# Request proxy URI from user
do {
    $uri = Read-Host "Enter proxy URI (vless://, vmess://, shadowsocks://, trojan://, or socks://)"
    $validPrefix = $uri -match "^(vless|vmess|shadowsocks|trojan|socks)://"
    if (-not $validPrefix) {
        Write-Host "Invalid URI. Must start with vless://, vmess://, shadowsocks://, trojan://, or socks://" -ForegroundColor Red
    }
} while (-not $validPrefix)

# Find available port
$port = Read-Host "Enter HTTP port (default: 1080)"
if ([string]::IsNullOrWhiteSpace($port)) {
    $port = 1080
}

$port = [int]$port
while ($true) {
    $listener = $null
    try {
        $listener = New-Object System.Net.Sockets.TcpListener([System.Net.IPAddress]::Loopback, $port)
        $listener.Start()
        $listener.Stop()
        Write-Host "Port $port is available" -ForegroundColor Green
        break
    } catch {
        Write-Host "Port $port is in use, trying $($port + 1)" -ForegroundColor Yellow
        $port++
    } finally {
        if ($listener) { $listener.Stop() }
    }
}

# Create unique task name
$taskName = "V2ProxyService $serverLocation $port"
$v2parserPath = Join-Path $binPath "v2parser.exe"
$xrayPath = Join-Path $binPath "xray.exe"

# Create batch file wrapper
$batchPath = Join-Path $binPath "v2proxy-$serverLocation.bat"
$batchContent = "@echo off`ncd /d `"$binPath`"`n`"$v2parserPath`" `"$uri`" --httpport $port --run --xray-binary `"$xrayPath`""
Set-Content -Path $batchPath -Value $batchContent -Encoding ASCII

# Remove existing task if exists
Unregister-ScheduledTask -TaskName $taskName -Confirm:$false -ErrorAction SilentlyContinue

# Create scheduled task action
$action = New-ScheduledTaskAction -Execute $batchPath -WorkingDirectory $binPath

# Create trigger for system startup
$trigger = New-ScheduledTaskTrigger -AtStartup

# Create principal to run with highest privileges
$principal = New-ScheduledTaskPrincipal -UserId "SYSTEM" -LogonType ServiceAccount -RunLevel Highest

# Create settings
$settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -StartWhenAvailable -RestartCount 3 -RestartInterval (New-TimeSpan -Minutes 1)

# Register scheduled task
Register-ScheduledTask -TaskName $taskName -Action $action -Trigger $trigger -Principal $principal -Settings $settings -Description "V2Ray Proxy Service - $serverLocation"

# Start task immediately
Start-ScheduledTask -TaskName $taskName

Start-Sleep -Seconds 2

# Check if task is running
$task = Get-ScheduledTask -TaskName $taskName
$taskInfo = Get-ScheduledTaskInfo -TaskName $taskName

Write-Host "`nTask '$taskName' created!" -ForegroundColor Green
Write-Host "Status: $($task.State)" -ForegroundColor Cyan
Write-Host "Last Run: $($taskInfo.LastRunTime)" -ForegroundColor Cyan
Write-Host "Proxy is running on http://127.0.0.1:$port" -ForegroundColor Cyan
