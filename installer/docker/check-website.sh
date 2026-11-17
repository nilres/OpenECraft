#!/usr/bin/env bash
# Starts the image of the website once and checks that it serves the page in its
# languages, the settings and the public address from the environment, the
# WebAssembly as the build compressed it, the license files and the web editor at
# /editor/; used by the CI workflows (ci.yml, docker-website.yml). The downloads go to
# a temporary folder: files in the checkout would change the build context.
#
#   bash installer/docker/check-website.sh [image] [port]
set -euo pipefail

image=${1:-openecraft-website:ci}
port=${2:-8088}
name=openecraft-website-check
url=http://127.0.0.1:$port
work=$(mktemp -d)
trap 'docker rm -f "$name" > /dev/null 2>&1 || true; rm -rf "$work"' EXIT
cd "$work"

docker run -d --name "$name" -p "$port:8080" -e OPENECRAFT_EDITOR_URL=https://editor.example/ \
  -e OPENECRAFT_SITE_URL=https://site.example/openecraft \
  -e OPENECRAFT_LEGAL_NAME='ACME "Tools" GmbH' -e OPENECRAFT_LEGAL_ADDRESS='Street 1 | 12345 City' "$image" > /dev/null
for _ in $(seq 30); do curl -fs "$url/" > /dev/null && break; sleep 1; done
curl -fsS "$url/" -o site.html || { docker logs "$name"; exit 1; }
grep -q canvas-schematic site.html
# The links of the page come from the environment.
curl -fsS "$url/config.js" | tee config.js
grep -q 'editor: "https://editor.example/"' config.js
grep -q 'repository: "https://github.com/' config.js
# The legal notice too, quotes escaped.
grep -qF 'name: "ACME \"Tools\" GmbH",' config.js
grep -qF 'address: "Street 1 | 12345 City",' config.js
if command -v node > /dev/null; then node --check config.js; fi
# The tags of the public address, robots.txt and the sitemap.
grep -qF '<link rel="canonical" href="https://site.example/openecraft/" />' site.html
grep -qF '<meta property="og:image" content="https://site.example/openecraft/social.jpg" />' site.html
curl -fsS "$url/robots.txt" | grep -qx 'Sitemap: https://site.example/openecraft/sitemap.xml'
curl -fsS "$url/sitemap.xml" | grep -qF '<loc>https://site.example/openecraft/</loc>'
curl -fsS "$url/index.html" | grep -qF 'rel="canonical"'
# The page in the other languages, each with its own address and the others'.
grep -qF '<link rel="alternate" hreflang="de" href="https://site.example/openecraft/de.html" />' site.html
curl -fsS "$url/de.html" -o de.html
grep -qF '<html lang="de">' de.html
grep -qF '<link rel="canonical" href="https://site.example/openecraft/de.html" />' de.html
curl -fsS "$url/sitemap.xml" | grep -qF '<loc>https://site.example/openecraft/ja.html</loc>'
# The module is named in the page and loaded by its script; the first sheet is a picture.
grep -q 'window.OPENECRAFT_MODULE = \["./openecraft-website-[0-9a-f]*\.js", "./openecraft-website-[0-9a-f]*_bg\.wasm"\]' site.html
if grep -q 'rel="modulepreload"' site.html; then exit 1; fi
curl -fsS -o /dev/null "$url/sheet.svg"
# The log has no whole address of a visitor.
docker logs "$name" 2>&1 | grep -q '"GET /de.html HTTP/1.1" 200'
if docker logs "$name" 2>&1 | grep 'GET /de.html' | grep -vqE '^[0-9]+\.[0-9]+\.[0-9]+\.0 '; then exit 1; fi
# The routings of the examples come with the page.
curl -fsS "$url/routing/index.json" | grep -q '"conveyor.json"'
curl -fsS "$url/routing/conveyor.json" | grep -q '"routing"'
wasm=$(grep -o '[^"/]*_bg\.wasm' site.html | head -n 1)
test -n "$wasm"
type=$(curl -fsS -o /dev/null -w '%{content_type}' "$url/$wasm")
echo "$wasm: $type"
test "$type" = application/wasm
for file in site.js icon.svg apple-touch-icon.png images/schematic.webp social.jpg LICENSE-MIT THIRD-PARTY-LICENSES.html; do
  curl -fsS -o /dev/null "$url/$file"
done
# The WebAssembly is sent as the build of the image compressed it.
curl -fsS -o wasm.gz -D headers.txt -H 'Accept-Encoding: gzip' "$url/$wasm"
grep -qi '^content-encoding: gzip' headers.txt
gzip -t wasm.gz
# With Brotli to who takes it, the one of the web editor as well.
curl -fsS -o wasm.br -D headers.txt -H 'Accept-Encoding: gzip, deflate, br, zstd' "$url/$wasm"
grep -qi '^content-encoding: br' headers.txt
grep -qi '^content-type: application/wasm' headers.txt
grep -qi '^vary: accept-encoding' headers.txt
grep -qi '^cache-control: .*immutable' headers.txt
test "$(wc -c < wasm.br)" -lt "$(wc -c < wasm.gz)"
curl -fsS "$url/editor/" -o editor.html
grep -q openecraft_canvas editor.html
editor=$(grep -o '[^"/]*_bg\.wasm' editor.html | head -n 1)
test -n "$editor"
curl -fsS -o /dev/null -D headers.txt -H 'Accept-Encoding: br' "$url/editor/$editor"
grep -qi '^content-encoding: br' headers.txt
curl -fsS -o /dev/null -D headers.txt -H 'Accept-Encoding: br' "$url/editor/${editor%_bg.wasm}.js"
grep -qi '^content-encoding: br' headers.txt
grep -qi '^content-type: application/javascript' headers.txt
docker logs "$name"
echo "$image serves the website"
