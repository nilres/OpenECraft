cask "openecraft" do
  version "0.1.2"
  sha256 "bf58be78d2cf6e46fec4283cccd8e872c414957307fd26b9a8802d82688861b5"

  url "https://github.com/nilres/OpenECraft/releases/download/v#{version}/OpenECraft-#{version}-macos.dmg",
      verified: "github.com/nilres/OpenECraft/"
  name "OpenECraft"
  desc "Electrical CAE editor for schematics, reports and panel layouts"
  homepage "https://openecraft.org/"

  livecheck do
    url :url
    strategy :github_latest
  end

  # The editor replaces OpenECraft.app from the disk image of a newer release itself.
  auto_updates true

  app "OpenECraft.app"
  app "OpenECraft Parts Manager.app"
  binary "#{appdir}/OpenECraft.app/Contents/MacOS/openecraft-cli"
  binary "#{appdir}/OpenECraft.app/Contents/MacOS/openecraft-parts-server"
  binary "#{appdir}/OpenECraft.app/Contents/MacOS/openecraft-project-server"

  zap trash: [
    "~/Library/Application Support/OpenECraft",
    "~/Library/Application Support/OpenECraft-parts-manager",
    "~/Library/Saved Application State/io.github.nilres.OpenECraft.PartsManager.savedState",
    "~/Library/Saved Application State/io.github.nilres.OpenECraft.savedState",
  ]

  caveats <<~EOS
    The apps are not notarized yet: Gatekeeper asks for confirmation on the first start
    (System Settings > Privacy & Security > Open Anyway).
  EOS
end
