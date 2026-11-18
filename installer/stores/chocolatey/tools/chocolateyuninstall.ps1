$ErrorActionPreference = 'Stop'

[array]$keys = Get-UninstallRegistryKey -SoftwareName 'OpenECraft*'
foreach ($key in $keys) {
  # UninstallString is the quoted path of unins000.exe written by Inno Setup.
  Uninstall-ChocolateyPackage -PackageName $env:ChocolateyPackageName -FileType 'exe' `
    -SilentArgs '/VERYSILENT /SUPPRESSMSGBOXES /NORESTART' -File ($key.UninstallString -replace '"', '')
}
