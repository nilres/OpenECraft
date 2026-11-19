//! What the build knows and the pages need: `build.js`, a script beside the main page.
//! It names the module of the site for the pages that run it without being built by
//! trunk (the tools), the screen recording of the first screen when the site has one
//! (`website/tour.webm`) and the numbers of the project on GitHub when the build was
//! given them (`OPENECRAFT_STARS`, `OPENECRAFT_CONTRIBUTORS`).

use serde_json::json;

/// The file of the script below the site.
pub const FILE: &str = "build.js";

/// The recording of the first screen and its still picture, in `website/`.
pub const TOUR: [&str; 2] = ["tour.webm", "tour.jpg"];

/// The variables of the build with the stars and the contributors of the repository.
pub const NUMBERS: [&str; 2] = ["OPENECRAFT_STARS", "OPENECRAFT_CONTRIBUTORS"];

/// The script of the module and its WebAssembly, as trunk names them in the main page
/// (`window.OPENECRAFT_MODULE = ["./….js", "./…_bg.wasm"]`, `Trunk.toml`).
pub fn module(index: &str) -> Option<[String; 2]> {
    let list = index.split("window.OPENECRAFT_MODULE = [").nth(1)?.split(']').next()?;
    let mut names = list.split(',').map(|name| name.trim().trim_matches('"').trim_start_matches("./").to_string());
    let (script, wasm) = (names.next()?, names.next()?);
    (script.ends_with(".js") && wasm.ends_with(".wasm")).then_some([script, wasm])
}

/// A number of the build: digits only, 0 for anything else.
pub fn number(value: Option<&str>) -> u64 {
    value.and_then(|v| v.trim().parse().ok()).unwrap_or(0)
}

/// The script. The addresses are those of the files beside it, whatever page loads it.
pub fn script(module: Option<&[String; 2]>, tour: bool, poster: bool, stars: u64, contributors: u64) -> String {
    let data = json!({
        "module": module.map(|m| m.to_vec()).unwrap_or_default(),
        "tour": if tour { TOUR[0] } else { "" },
        "poster": if tour && poster { TOUR[1] } else { "" },
        "stars": stars,
        "contributors": contributors,
    });
    format!(
        "// Written by the build of the site (crates/openecraft-site, docs/website.md).\n\"use strict\";\n(() => {{\n    const here = document.currentScript.src;\n    const beside = file => file ? new URL(file, here).href : \"\";\n    const build = {data};\n    build.module = build.module.map(beside);\n    build.tour = beside(build.tour);\n    build.poster = beside(build.poster);\n    window.OPENECRAFT_BUILD = build;\n}})();\n"
    )
}

#[cfg(test)]
mod tests;
