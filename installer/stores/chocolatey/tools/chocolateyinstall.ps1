$ErrorActionPreference = 'Stop'

$packageArgs = @{
  packageName    = $env:ChocolateyPackageName
  fileType       = 'exe'
  url64bit       = 'https://github.com/nilres/OpenECraft/releases/download/v0.1.2/OpenECraft-0.1.2-setup.exe'
  checksum64     = 'f254bba47ff33af744b78003948f18e7c6ef8fec88fab3b02179efbb35e76346'
  checksumType64 = 'sha256'
  # Inno Setup: all users in Program Files; --install-arguments adds /COMPONENTS and /TASKS.
  silentArgs     = '/VERYSILENT /SUPPRESSMSGBOXES /NORESTART /SP- /ALLUSERS'
  validExitCodes = @(0)
  softwareName   = 'OpenECraft*'
}

Install-ChocolateyPackage @packageArgs
