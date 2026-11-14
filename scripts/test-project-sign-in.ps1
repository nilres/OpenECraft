<#
PowerShell 7, Rust on PATH. Tests remembered project-server sign-ins against a
disposable loopback server and unique synthetic Windows Credential Manager entries.
No existing server, project, credential or application settings are used.
Retains only synthetic server data/logs under target/project-sign-in; stops its
server and removes its credential entries (the Rust test's cleanup guard).
#>
param([string]$LogDirectory = (Join-Path $PSScriptRoot '../target/project-sign-in'))
$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
Set-Location $repo
& cargo build -p openecraft-project-server --locked
if ($LASTEXITCODE -ne 0) {throw 'Project server build failed'}
$runDir = Join-Path $LogDirectory ('sign-in-' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Force -Path $runDir | Out-Null
$runDir = (Resolve-Path $runDir).Path
$listener = [Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback, 0)
$listener.Start()
$port = $listener.LocalEndpoint.Port
$listener.Stop()
$url = "http://127.0.0.1:$port"
$secret = [Guid]::NewGuid().ToString('N')
$target = if ($env:CARGO_TARGET_DIR) {$env:CARGO_TARGET_DIR} else {Join-Path $repo 'target'}
$start = [Diagnostics.ProcessStartInfo]::new()
$start.FileName = Join-Path $target 'debug/openecraft-project-server.exe'
$start.UseShellExecute = $false
$start.CreateNoWindow = $true
$start.RedirectStandardOutput = $true
$start.RedirectStandardError = $true
$start.ArgumentList.Add('--bind'); $start.ArgumentList.Add("127.0.0.1:$port")
$start.ArgumentList.Add('--data'); $start.ArgumentList.Add((Join-Path $runDir 'data'))
$start.Environment['OPENECRAFT_PROJECT_TOKEN'] = $secret
$server = [Diagnostics.Process]::Start($start)
$stdout = $server.StandardOutput.ReadToEndAsync()
$stderr = $server.StandardError.ReadToEndAsync()
$envNames = @('URL','USER','PASSWORD','OTHER_USER','OTHER_PASSWORD','PROFILE_DIR')
try {
    $headers = @{Authorization="Bearer $secret"}
    for ($attempt=0; $attempt -lt 100; $attempt++) {
        try { $null=Invoke-RestMethod "$url/api/v1/info" -Headers $headers; break }
        catch { if ($server.HasExited -or $attempt -eq 99) {throw}; Start-Sleep -Milliseconds 100 }
    }
    foreach ($pair in @(@('USER','PASSWORD'),@('OTHER_USER','OTHER_PASSWORD'))) {
        $user = 'sign-in-' + [Guid]::NewGuid().ToString('N')
        $password = [Guid]::NewGuid().ToString('N')
        $body = @{password=$password;role='editor';projects=@()} | ConvertTo-Json
        $null = Invoke-RestMethod "$url/api/v1/users/$user" -Method Put -Headers $headers -ContentType 'application/json' -Body $body
        [Environment]::SetEnvironmentVariable("OPENECRAFT_AUTH_TEST_$($pair[0])",$user,'Process')
        [Environment]::SetEnvironmentVariable("OPENECRAFT_AUTH_TEST_$($pair[1])",$password,'Process')
    }
    $env:OPENECRAFT_AUTH_TEST_URL = $url
    $env:OPENECRAFT_AUTH_TEST_PROFILE_DIR = Join-Path $runDir 'profile'
    & cargo test -p openecraft live_auth_http_remembered_project_sign_in --locked -- --ignored --nocapture *> (Join-Path $runDir 'live.log')
    $code = $LASTEXITCODE
    Get-Content (Join-Path $runDir 'live.log') -Tail 35
    if ($code -ne 0) {throw 'Live project sign-in test failed'}
    Write-Output "PASS: disposable-server remembered sign-in test ($runDir)"
} finally {
    if (-not $server.HasExited) {$server.Kill(); $server.WaitForExit()}
    [IO.File]::WriteAllText((Join-Path $runDir 'server.log'),$stdout.GetAwaiter().GetResult()+$stderr.GetAwaiter().GetResult())
    foreach ($name in $envNames) {[Environment]::SetEnvironmentVariable("OPENECRAFT_AUTH_TEST_$name",$null,'Process')}
}
