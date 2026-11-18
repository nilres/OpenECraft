#!/bin/sh
# Writes the pages beside the main page in every language (src/pages.rs,
# docs/website.md): compare/de.html, … beside compare/index.html. Run by trunk after
# the build (post_build hook of Trunk.toml; on Windows pages.ps1).
set -eu
dir=${TRUNK_STAGING_DIR:?run by trunk}
for page in panel-builders electricians education compare; do
    cargo run --release --quiet -p openecraft-i18n --example pages -- "$dir/$page"
done
