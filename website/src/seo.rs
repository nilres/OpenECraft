//! What search engines and link previews read of the page: the tags of its head, the
//! structured data, the gallery and the parts that need the public address of the site
//! (`installer/docker/website-site.sh`). The tests keep them together.

/// The variable of the Docker image with the public address of the site.
pub const SITE_URL: &str = "OPENECRAFT_SITE_URL";

/// The pictures of the gallery: `images/<name>.webp`, made smaller from the screenshot
/// `docs/images/<name>.png` of the documentation.
pub const GALLERY: [&str; 6] = ["schematic", "panel-3d", "heat-map", "wiring-assistant", "pneumatics", "terminal-diagram"];

#[cfg(test)]
mod tests;
