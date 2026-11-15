//! What search engines and link previews read of the page: the tags of its head, the
//! structured data, the gallery and the parts that need the public address of the site
//! (`installer/docker/website-site.sh`). The tests keep them together.

/// The variable of the Docker image with the public address of the site.
pub const SITE_URL: &str = "OPENECRAFT_SITE_URL";

/// What search results and link previews say about the page: the description of its
/// head, translated in the pages of the other languages (`openecraft_i18n::page`).
pub const DESCRIPTION: &str = openecraft_i18n::tr_const!(
    "OpenECraft is an open source electrical CAE editor: schematics, reports, panel layout in 3D, a command line and an MCP server. Try it live in the browser."
);

/// The pictures of the gallery: `images/<name>.webp`, made smaller from the screenshot
/// `docs/images/<name>.png` of the documentation.
pub const GALLERY: [&str; 6] = ["schematic", "panel-3d", "heat-map", "wiring-assistant", "pneumatics", "terminal-diagram"];

#[cfg(test)]
mod tests;
