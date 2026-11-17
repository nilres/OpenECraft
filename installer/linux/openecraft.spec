# RPM package of OpenECraft, built by installer/linux/build.sh from the binaries it
# installed below %{files_dir} (bin/, share/ as for the prefix /usr).

%global debug_package %{nil}
%global app_id io.github.nilres.OpenECraft

Name:           openecraft
Version:        %{pkg_version}
Release:        1
Summary:        Electrical CAE editor: schematics, panel layouts and reports
License:        (MIT OR Apache-2.0) AND OFL-1.1 AND Ubuntu-font-1.0 AND Bitstream-Vera AND MPL-2.0
URL:            https://openecraft.org/
# The window system and OpenGL libraries are loaded at run time, rpmbuild does not
# see them.
Requires:       libxkbcommon.so.0()(64bit)
Requires:       libEGL.so.1()(64bit)
Requires:       libGL.so.1()(64bit)
Recommends:     libwayland-client.so.0()(64bit)
Recommends:     libwayland-egl.so.1()(64bit)
Recommends:     libX11.so.6()(64bit)
Recommends:     libXcursor.so.1()(64bit)
Recommends:     libXi.so.6()(64bit)
Recommends:     libXrandr.so.2()(64bit)
Recommends:     libxkbcommon-x11.so.0()(64bit)
Recommends:     xdg-desktop-portal

%description
OpenECraft is an open electrical CAE editor in the spirit of EPLAN Electric P8:
schematic pages with device tags, cross references and a netlist, panel layout
in 2D and 3D, checks, reports and exports.

The package contains the editor, the parts manager, the command line and MCP
server for AI agents and the parts and project servers.

%install
mkdir -p %{buildroot}%{_prefix}
cp -a %{files_dir}/. %{buildroot}%{_prefix}/

%files
%{_bindir}/openecraft
%{_bindir}/openecraft-parts-manager
%{_bindir}/openecraft-cli
%{_bindir}/openecraft-parts-server
%{_bindir}/openecraft-project-server
%{_datadir}/applications/%{app_id}.desktop
%{_datadir}/applications/%{app_id}.PartsManager.desktop
%{_datadir}/metainfo/%{app_id}.metainfo.xml
%{_datadir}/mime/packages/%{app_id}.xml
%{_datadir}/icons/hicolor/256x256/apps/%{app_id}.png
%{_datadir}/icons/hicolor/scalable/apps/%{app_id}.svg
%{_docdir}/openecraft
