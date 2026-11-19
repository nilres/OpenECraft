# Writes the pages beside the main page in every language (src/pages.rs,
# docs/website.md): compare\de.html, ... beside compare\index.html. Run by trunk after
# the build on Windows (post_build hook of Trunk.toml, elsewhere pages.sh).

$ErrorActionPreference = 'Stop'
$dir = $env:TRUNK_STAGING_DIR
if (-not $dir) { throw 'run by trunk' }

foreach ($page in 'panel-builders', 'electricians', 'education', 'compare') {
    cargo run --release --quiet -p openecraft-i18n --example pages -- "$dir\$page"
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
