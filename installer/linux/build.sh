#!/usr/bin/env bash
# Builds the Linux packages of OpenECraft into target/installer/:
#
#   bash installer/linux/build.sh [tar.gz] [deb] [rpm] [appimage] [flatpak] [snap]
#
# Without arguments it builds tar.gz, deb, rpm and appimage. The packages contain the
# editor, the parts manager, the command line / MCP server and the parts and project
# servers for the architecture of the build machine. Each format needs its tool:
#
#   deb       dpkg-deb, dpkg-shlibdeps (apt install dpkg-dev)
#   rpm       rpmbuild (dnf install rpm-build, apt install rpm)
#   appimage  Debian or Ubuntu (dpkg-query), patchelf, appimagetool (APPIMAGETOOL=<path>,
#             otherwise release 1.9.1 is downloaded) and the AppImage runtime (release
#             20251108, downloaded; both checksums verified)
#   flatpak   flatpak, flatpak-builder (the runtime is installed from Flathub)
#   snap      snapcraft (SNAPCRAFT="sudo snapcraft" SNAPCRAFT_ARGS=--destructive-mode in CI)
#
# All formats need cargo-about for the third-party licenses
# (cargo install cargo-about@0.9.3 --locked --features cli).
# OPENECRAFT_BIN_DIR=<dir> packages binaries built before instead of running cargo.

set -euo pipefail
umask 022

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/../.." && pwd)"
out="$root/target/installer"
work="$out/linux"
id=io.github.nilres.OpenECraft
bins=(openecraft openecraft-parts-manager openecraft-cli openecraft-parts-server openecraft-project-server)
version="$(sed -n '/^\[workspace\.package\]/,/^\[/s/^version *= *"\([^"]*\)".*/\1/p' "$root/Cargo.toml" | head -n 1)"
notices="$out/THIRD-PARTY-LICENSES.html"
# deb and rpm sort `~` before a release ("0.2.0~rc1" < "0.2.0") and do not allow `-`.
pkgver="${version//-/\~}"
arch="$(uname -m)"
formats=("$@")
[ ${#formats[@]} -gt 0 ] || formats=(tar.gz deb rpm appimage)

die() { echo "error: $*" >&2; exit 1; }
need() { command -v "$1" >/dev/null || die "$1 not found, needed for $2"; }

[ -n "$version" ] || die "no version in [workspace.package] of Cargo.toml"
need cargo-about "the third-party licenses (cargo install cargo-about@0.9.3 --locked --features cli)"
grep -q "<release version=\"$version\"" "$here/$id.metainfo.xml" \
    || die "add <release version=\"$version\" date=\"…\"/> to installer/linux/$id.metainfo.xml"

if [ -n "${OPENECRAFT_BIN_DIR:-}" ]; then
    bin_dir="$OPENECRAFT_BIN_DIR"
else
    cargo build --release --locked --manifest-path "$root/Cargo.toml" \
        -p openecraft -p openecraft-parts-manager -p openecraft-cli -p openecraft-parts-server -p openecraft-project-server
    bin_dir="$root/target/release"
fi

# The installed files below <dir> as for the prefix /usr: bin/, share/.
stage() {
    local dir="$1" b
    rm -rf "$dir"
    install -d "$dir/bin"
    for b in "${bins[@]}"; do
        install -m 755 -s "$bin_dir/$b" "$dir/bin/$b"
    done
    install -D -m 644 "$here/$id.desktop" "$dir/share/applications/$id.desktop"
    install -D -m 644 "$here/$id.PartsManager.desktop" "$dir/share/applications/$id.PartsManager.desktop"
    install -D -m 644 "$here/$id.metainfo.xml" "$dir/share/metainfo/$id.metainfo.xml"
    install -D -m 644 "$here/$id.xml" "$dir/share/mime/packages/$id.xml"
    install -D -m 644 "$root/assets/icon.png" "$dir/share/icons/hicolor/256x256/apps/$id.png"
    install -D -m 644 "$root/assets/icon.svg" "$dir/share/icons/hicolor/scalable/apps/$id.svg"
    install -D -m 644 "$root/README.md" "$dir/share/doc/openecraft/README.md"
    install -D -m 644 "$root/LICENSE-MIT" "$dir/share/doc/openecraft/LICENSE-MIT"
    install -D -m 644 "$root/LICENSE-APACHE" "$dir/share/doc/openecraft/LICENSE-APACHE"
    install -D -m 644 "$root/crates/openecraft-fonts/fonts/OFL.txt" "$dir/share/doc/openecraft/OFL-NotoSans.txt"
    install -D -m 644 "$notices" "$dir/share/doc/openecraft/THIRD-PARTY-LICENSES.html"
}

build_tar() {
    local name="openecraft-$version-linux-$arch"
    stage "$work/$name"
    install -m 755 "$here/install.sh" "$work/$name/install.sh"
    tar -C "$work" --owner=0 --group=0 -czf "$out/OpenECraft-$version-linux-$arch.tar.gz" "$name"
}

build_deb() {
    need dpkg-deb deb
    need dpkg-shlibdeps deb
    local pkg="$work/deb" darch shlibs
    darch="$(dpkg --print-architecture)"
    stage "$pkg/usr"
    cat > "$pkg/usr/share/doc/openecraft/copyright" <<EOF
Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
Upstream-Name: OpenECraft
Source: https://github.com/nilres/OpenECraft

Files: *
Copyright: OpenECraft contributors
License: MIT or Apache-2.0
Comment: The programs contain libraries and fonts under their own licenses, listed in
 THIRD-PARTY-LICENSES.html; the license texts are LICENSE-MIT and LICENSE-APACHE.

Files: usr/bin/*
Copyright: OpenECraft contributors and the authors listed in THIRD-PARTY-LICENSES.html
License: (MIT or Apache-2.0) and OFL-1.1 and Ubuntu-font-1.0 and Bitstream-Vera and MPL-2.0
Comment: Rust libraries (MIT, Apache-2.0, BSD, ISC, Zlib, BSL-1.0, Unicode-3.0,
 CDLA-Permissive-2.0, MPL-2.0) and fonts linked into the programs: Noto Sans SC / JP /
 KR subsets (OFL-1.1, OFL-NotoSans.txt), Ubuntu Light (Ubuntu Font Licence), Noto Emoji
 (OFL-1.1), Hack (MIT and Bitstream Vera License). All license texts are in
 THIRD-PARTY-LICENSES.html.
EOF
    # dpkg-shlibdeps wants a debian/control in the current directory.
    mkdir -p "$work/shlibs/debian"
    printf 'Source: openecraft\n\nPackage: openecraft\nArchitecture: any\n' > "$work/shlibs/debian/control"
    shlibs="$(cd "$work/shlibs" && dpkg-shlibdeps -O "$pkg"/usr/bin/* | sed -n 's/^shlibs:Depends=//p')"
    install -d "$pkg/DEBIAN"
    # The window system and OpenGL libraries are loaded at run time, dpkg-shlibdeps
    # does not see them.
    cat > "$pkg/DEBIAN/control" <<EOF
Package: openecraft
Version: $pkgver
Architecture: $darch
Maintainer: ${MAINTAINER:-OpenECraft <https://github.com/nilres/OpenECraft/issues>}
Installed-Size: $(du -sk "$pkg/usr" | cut -f 1)
Depends: $shlibs, libxkbcommon0, libgl1, libegl1
Recommends: libwayland-client0, libwayland-egl1, libx11-6, libxcursor1, libxi6, libxrandr2, libxkbcommon-x11-0, xdg-desktop-portal
Section: electronics
Priority: optional
Homepage: https://github.com/nilres/OpenECraft
Description: Electrical CAE editor: schematics, panel layouts and reports
 OpenECraft is an open electrical CAE editor in the spirit of EPLAN Electric P8:
 schematic pages with device tags, cross references and a netlist, panel layout
 in 2D and 3D, checks, reports and exports.
 .
 The package contains the editor, the parts manager, the command line and MCP
 server for AI agents and the parts and project servers.
EOF
    dpkg-deb --root-owner-group --build "$pkg" "$out/openecraft_${pkgver}_$darch.deb"
}

build_rpm() {
    need rpmbuild rpm
    stage "$work/rpm/files"
    rpmbuild -bb --target "$arch" \
        --define "_topdir $work/rpm" \
        --define "pkg_version $pkgver" \
        --define "files_dir $work/rpm/files" \
        "$here/openecraft.spec"
    cp "$work/rpm/RPMS/$arch/openecraft-$pkgver-1.$arch.rpm" "$out/"
}

# appimagetool and the AppImage runtime it puts in front of the image, pinned with their
# SHA-256 (RUNTIME-LICENSES.txt lists the runtime's components; update both together).
appimagetool_version=1.9.1
runtime_version=20251108
declare -A appimagetool_sha256=(
    [x86_64]=ed4ce84f0d9caff66f50bcca6ff6f35aae54ce8135408b3fa33abfc3cb384eb0
    [aarch64]=f0837e7448a0c1e4e650a93bb3e85802546e60654ef287576f46c71c126a9158
)
declare -A runtime_sha256=(
    [x86_64]=2fca8b443c92510f1483a883f60061ad09b46b978b2631c807cd873a47ec260d
    [aarch64]=00cbdfcf917cc6c0ff6d3347d59e0ca1f7f45a6df1a428a0d6d8a78664d87444
)

# download <url> <file> <sha256>: downloads once, refuses a file with another checksum.
download() {
    if [ ! -f "$2" ] || ! echo "$3  $2" | sha256sum -c --status -; then
        need curl appimage
        curl -fL -o "$2.part" "$1"
        echo "$3  $2.part" | sha256sum -c --status - || die "checksum of $1 does not match"
        mv "$2.part" "$2"
    fi
}

# The window libraries winit loads at run time that are not on every system, with what
# they need. The AppImage carries them; the libraries of the AppImage excludelist
# (libX11, libxcb, libwayland-client, libGL, libEGL, …) and the Wayland libraries tied to
# libwayland-client come from the system.
appimage_libs=(libxkbcommon.so.0 libxkbcommon-x11.so.0 libxcb-xkb.so.1 libXcursor.so.1 libXrender.so.1
    libXfixes.so.3 libXi.so.6 libXext.so.6)
appimage_system_libs=(libc.so.6 libm.so.6 libdl.so.2 libpthread.so.0 librt.so.1 libutil.so.1 libgcc_s.so.1
    ld-linux-x86-64.so.2 ld-linux-aarch64.so.1 libX11.so.6 libX11-xcb.so.1 libxcb.so.1 libz.so.1)

# bundle_libs <AppDir>: copies appimage_libs to usr/lib and their packages' copyright
# files to usr/share/doc, makes the editor and the parts manager look there first and refuses a library the AppImage neither has nor may expect.
# $ORIGIN is for patchelf, not the shell.
# shellcheck disable=SC2016
bundle_libs() {
    local dir="$1" lib path pkg file need
    need patchelf appimage
    need readelf appimage
    need ldconfig appimage
    need dpkg-query "the licenses of the libraries in the AppImage (build it on Debian or Ubuntu)"
    install -d "$dir/usr/lib"
    for lib in "${appimage_libs[@]}"; do
        path="$(ldconfig -p | awk -v lib="$lib" '$1 == lib && !found { print $NF; found = 1 }')"
        [ -n "$path" ] || die "$lib not found, needed for appimage"
        path="$(readlink -f "$path")"
        install -m 644 "$path" "$dir/usr/lib/$lib"
        patchelf --set-rpath '$ORIGIN' "$dir/usr/lib/$lib"
        pkg="$(dpkg-query -S "*/${path##*/}" | awk -F : 'NR == 1 { print $1 }')" || die "no package contains $path"
        install -D -m 644 "/usr/share/doc/$pkg/copyright" "$dir/usr/share/doc/$pkg/copyright"
    done
    patchelf --set-rpath '$ORIGIN/../lib' "$dir/usr/bin/openecraft" "$dir/usr/bin/openecraft-parts-manager"
    for file in "$dir"/usr/bin/* "$dir"/usr/lib/*; do
        for need in $(readelf -d "$file" | sed -n 's/.*(NEEDED).*\[\(.*\)\]$/\1/p'); do
            [ -e "$dir/usr/lib/$need" ] || [[ " ${appimage_system_libs[*]} " == *" $need "* ]] \
                || die "$need, needed by ${file##*/}, is neither in the AppImage nor on every system"
        done
    done
    local bins_glibc libs_glibc
    bins_glibc="$(glibc_needed "$dir"/usr/bin/*)"
    libs_glibc="$(glibc_needed "$dir"/usr/lib/*)"
    [ "$(printf '%s\n' "$bins_glibc" "$libs_glibc" | sort -V | tail -n 1)" = "$bins_glibc" ] \
        || die "the libraries for the AppImage need $libs_glibc, the programs only $bins_glibc: build on an older system"
}

# glibc_needed <ELF file>...: the newest glibc version the files need, e.g. GLIBC_2.35.
glibc_needed() {
    readelf -V "$@" | grep -o 'GLIBC_[0-9.]*' | sort -uV | tail -n 1
}

build_appimage() {
    local dir="$work/OpenECraft.AppDir" tool="${APPIMAGETOOL:-}" runtime
    [ -n "${runtime_sha256[$arch]:-}" ] || die "no AppImage runtime for $arch"
    stage "$dir/usr"
    bundle_libs "$dir"
    # The AppImage catalog and appimagetool look for AppStream data by its older name.
    mv "$dir/usr/share/metainfo/$id.metainfo.xml" "$dir/usr/share/metainfo/$id.appdata.xml"
    install -m 644 "$here/appimage/RUNTIME-LICENSES.txt" "$dir/usr/share/doc/openecraft/APPIMAGE-RUNTIME-LICENSES.txt"
    install -m 755 "$here/appimage/AppRun" "$dir/AppRun"
    cp "$here/$id.desktop" "$dir/$id.desktop"
    cp "$root/assets/icon.png" "$dir/$id.png"
    ln -sf "$id.png" "$dir/.DirIcon"
    if [ -z "$tool" ]; then
        tool="$work/appimagetool-$appimagetool_version-$arch.AppImage"
        download "https://github.com/AppImage/appimagetool/releases/download/$appimagetool_version/appimagetool-$arch.AppImage" \
            "$tool" "${appimagetool_sha256[$arch]}"
        chmod +x "$tool"
    fi
    runtime="$work/runtime-$runtime_version-$arch"
    download "https://github.com/AppImage/type2-runtime/releases/download/$runtime_version/runtime-$arch" \
        "$runtime" "${runtime_sha256[$arch]}"
    # Extract-and-run works without FUSE (containers, CI).
    ARCH="$arch" APPIMAGE_EXTRACT_AND_RUN=1 "$tool" --runtime-file "$runtime" "$dir" "$out/OpenECraft-$version-$arch.AppImage"
}

build_flatpak() {
    need flatpak flatpak
    need flatpak-builder flatpak
    local dir="$work/flatpak"
    install -d "$dir"
    stage "$dir/files"
    cp "$here/flatpak/$id.yml" "$dir/$id.yml"
    flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
    flatpak-builder --user --force-clean --disable-rofiles-fuse --install-deps-from=flathub \
        --state-dir="$dir/.flatpak-builder" --repo="$dir/repo" "$dir/build" "$dir/$id.yml"
    flatpak build-bundle --runtime-repo=https://dl.flathub.org/repo/flathub.flatpakrepo \
        "$dir/repo" "$out/OpenECraft-$version-$arch.flatpak" "$id"
}

build_snap() {
    local dir="$work/snap" snapcraft="${SNAPCRAFT:-snapcraft}"
    need "${snapcraft##* }" snap
    rm -rf "$dir"
    stage "$dir/files/usr"
    # snapd does not look up theme icons of the snap, the entries name the file.
    sed -i "s|^Icon=.*|Icon=\${SNAP}/usr/share/icons/hicolor/256x256/apps/$id.png|" "$dir/files/usr/share/applications/"*.desktop
    install -d "$dir/snap/gui"
    cp "$root/assets/icon.png" "$dir/snap/gui/openecraft.png"
    sed "s/@VERSION@/$version/" "$here/snap/snapcraft.yaml" > "$dir/snap/snapcraft.yaml"
    # SNAPCRAFT and SNAPCRAFT_ARGS are split into words on purpose.
    # shellcheck disable=SC2086
    (cd "$dir" && $snapcraft pack ${SNAPCRAFT_ARGS:-})
    mv "$dir"/openecraft_*.snap "$out/"
}

install -d "$work"
cargo about generate --fail --manifest-path "$root/Cargo.toml" -c "$root/installer/about.toml" \
    -o "$notices" "$root/installer/about.hbs"
for format in "${formats[@]}"; do
    case "$format" in
        tar.gz) build_tar ;;
        deb) build_deb ;;
        rpm) build_rpm ;;
        appimage) build_appimage ;;
        flatpak) build_flatpak ;;
        snap) build_snap ;;
        *) die "unknown format $format (tar.gz, deb, rpm, appimage, flatpak, snap)" ;;
    esac
done
ls -l "$out"
