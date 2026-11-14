<#
Run with PowerShell 7 and a Rust toolchain on PATH:
    ./scripts/test-cable-rename.ps1
Builds and starts a disposable loopback server on a free port with two editors
and a viewer, using random credentials and synthetic projects only. Runs the
ignored HTTP test explicitly: conflicts, cancellation while a lock is pending,
other users' changes, stale drafts, permissions, undo/redo and failed transport.
Stops its server afterwards; keeps logs/data in target/rename-multiuser.
No existing project server is contacted.
#>
param([string]$LogDirectory = (Join-Path $PSScriptRoot '../target/rename-multiuser'))
$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
Set-Location $repo
& cargo build -p openecraft-project-server --locked
if ($LASTEXITCODE -ne 0) {throw 'Project server build failed'}
$out = $LogDirectory
New-Item -ItemType Directory -Force $out | Out-Null
$runDir = Join-Path $out ('multiuser-' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $runDir | Out-Null
$listener = [Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback, 0)
$listener.Start()
$port = $listener.LocalEndpoint.Port
$listener.Stop()
$url = "http://127.0.0.1:$port"
$secret = [Guid]::NewGuid().ToString('N')
$start = [Diagnostics.ProcessStartInfo]::new()
$start.FileName = Join-Path $repo 'target\debug\openecraft-project-server.exe'
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
try {
    $headers = @{Authorization="Bearer $secret"}
    for ($try=0; $try -lt 100; $try++) {
        try { $null=Invoke-RestMethod "$url/api/v1/info" -Headers $headers; break }
        catch { if ($server.HasExited -or $try -eq 99) {throw}; Start-Sleep -Milliseconds 100 }
    }
    foreach ($user in @('alice','bob','viewer')) {
        $password = [Guid]::NewGuid().ToString('N')
        $role = if ($user -eq 'viewer') {'viewer'} else {'editor'}
        $body = @{password=$password;role=$role;projects=@()} | ConvertTo-Json
        $null = Invoke-RestMethod "$url/api/v1/users/$user" -Method Put -Headers $headers -ContentType 'application/json' -Body $body
        $login = Invoke-RestMethod "$url/api/v1/login" -Method Post -ContentType 'application/json' -Body (@{user=$user;password=$password}|ConvertTo-Json)
        [Environment]::SetEnvironmentVariable("OPENECRAFT_RENAME_TEST_$($user.ToUpper())",$login.token,'Process')
    }
    $env:OPENECRAFT_RENAME_TEST_URL = $url
    Set-Location $repo
    & cargo test -p openecraft live_http_multiuser_rename --locked -- --ignored --nocapture > (Join-Path $out 'multiuser-live.log') 2>&1
    $testExit = $LASTEXITCODE
    Get-Content (Join-Path $out 'multiuser-live.log') -Tail 55
    if ($testExit -ne 0) {throw 'Multiuser live test failed'}
    Write-Output 'PASS: live HTTP multiuser test'
} finally {
    if (-not $server.HasExited) {$server.Kill(); $server.WaitForExit()}
    [IO.File]::WriteAllText((Join-Path $runDir 'server.log'),$stdout.GetAwaiter().GetResult()+$stderr.GetAwaiter().GetResult())
    foreach ($name in @('ALICE','BOB','VIEWER','URL')) {[Environment]::SetEnvironmentVariable("OPENECRAFT_RENAME_TEST_$name",$null,'Process')}
}
