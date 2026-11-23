$ErrorActionPreference = 'Stop'

$packageArgs = @{
  packageName    = $env:ChocolateyPackageName
  fileType       = 'exe'
  url64bit       = 'https://github.com/nilres/OpenECraft/releases/download/v0.1.3/OpenECraft-0.1.3-setup.exe'
  checksum64     = 'b1165d6ee1ce08d1c58fba11c14460a2b503ae1462237f5579cbc9a89ef6817d'
  checksumType64 = 'sha256'
  # Inno Setup: all users in Program Files; --install-arguments adds /COMPONENTS and /TASKS.
  silentArgs     = '/VERYSILENT /SUPPRESSMSGBOXES /NORESTART /SP- /ALLUSERS'
  validExitCodes = @(0)
  softwareName   = 'OpenECraft*'
}

Install-ChocolateyPackage @packageArgs
