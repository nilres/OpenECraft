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

# A page or file has a line: grep's options and the text follow the address. The answer
# is saved before it is searched. Never pipe curl (or docker logs) into grep -q or head:
# they stop reading at what they looked for, the sender fails at writing the rest
# ("curl: (23) Failure writing output") and, with pipefail, the check fails although
# the text is there; the larger the answer, the likelier (website/src/pages/tests.rs).
has() {
    local address=$1
    shift
    curl -fsS "$address" -o "$work/body" || { echo "no answer from $address" >&2; return 1; }
    grep -q "$@" "$work/body" || { echo "$address lacks: ${*: -1}" >&2; return 1; }
}

docker run -d --name "$name" -p "$port:8080" -e OPENECRAFT_EDITOR_URL=https://editor.example/ \
  -e OPENECRAFT_SITE_URL=https://site.example/openecraft \
  -e OPENECRAFT_STARS=1234 -e OPENECRAFT_CONTRIBUTORS=56 -e OPENECRAFT_NUMBERS_INTERVAL=0 \
  -e OPENECRAFT_VIDEOS_INTERVAL=0 \
  -e OPENECRAFT_LEGAL_NAME='ACME "Tools" GmbH' -e OPENECRAFT_LEGAL_ADDRESS='Street 1 | 12345 City' "$image" > /dev/null
for _ in $(seq 30); do curl -fs "$url/" > /dev/null && break; sleep 1; done
curl -fsS "$url/" -o site.html || { docker logs "$name"; exit 1; }
grep -q canvas-schematic site.html
# The links of the page come from the environment.
curl -fsS "$url/config.js" -o config.js
cat config.js
grep -q 'editor: "https://editor.example/"' config.js
grep -q 'repository: "https://github.com/' config.js
# The legal notice too, quotes escaped.
grep -qF 'name: "ACME \"Tools\" GmbH",' config.js
grep -qF 'address: "Street 1 | 12345 City",' config.js
if command -v node > /dev/null; then node --check config.js; fi
# The numbers of the project the image starts with; it can ask GitHub for newer ones.
has "$url/numbers.js" -xF 'window.OPENECRAFT_NUMBERS = { stars: 1234, contributors: 56 };'
grep -qF '<script src="numbers.js"></script>' site.html
# The videos: none until the image asked the channel, and the page that lists them.
has "$url/videos.js" -xF 'window.OPENECRAFT_VIDEOS = [];'
has "$url/videos/" -F '<script src="../videos.js"></script>'
has "$url/videos/de.html" -F 'data-videos'
grep -q 'videos: "https://www.youtube.com/@OpenECraft"' config.js
docker exec "$name" sh -c 'command -v curl' > /dev/null
# The tags of the public address, robots.txt and the sitemap.
grep -qF '<link rel="canonical" href="https://site.example/openecraft/" />' site.html
grep -qF '<meta property="og:image" content="https://site.example/openecraft/social.jpg" />' site.html
has "$url/robots.txt" -x 'Sitemap: https://site.example/openecraft/sitemap.xml'
has "$url/sitemap.xml" -F '<loc>https://site.example/openecraft/</loc>'
has "$url/index.html" -F 'rel="canonical"'
# The page in the other languages, each with its own address and the others'.
grep -qF '<link rel="alternate" hreflang="de" href="https://site.example/openecraft/de.html" />' site.html
curl -fsS "$url/de.html" -o de.html
grep -qF '<html lang="de">' de.html
grep -qF '<link rel="canonical" href="https://site.example/openecraft/de.html" />' de.html
has "$url/sitemap.xml" -F '<loc>https://site.example/openecraft/ja.html</loc>'
# The pages beside it, with their tags and in the sitemap.
curl -fsS "$url/compare/" -o compare.html
grep -qF '<link rel="canonical" href="https://site.example/openecraft/compare/" />' compare.html
grep -qF '<link rel="alternate" hreflang="de" href="https://site.example/openecraft/compare/de.html" />' compare.html
has "$url/compare/de.html" -F '<link rel="canonical" href="https://site.example/openecraft/compare/de.html" />'
has "$url/sitemap.xml" -F '<loc>https://site.example/openecraft/education/fr.html</loc>'
test "$(curl -s -o /dev/null -w '%{http_code}' "$url/compare")" = 301
# The pages the build writes: two folders deep, in their languages, with their tags.
curl -fsS "$url/symbols/contacts/" -o symbols.html
grep -qF '<link rel="canonical" href="https://site.example/openecraft/symbols/contacts/" />' symbols.html
grep -qF '<link rel="alternate" hreflang="de" href="https://site.example/openecraft/symbols/contacts/de.html" />' symbols.html
has "$url/symbols/contacts/de.html" -F '<html lang="de">'
test "$(curl -s -o /dev/null -w '%{http_code}' "$url/symbols/contacts")" = 301
curl -fsS -o /dev/null "$url/symbols/art/no.svg"
# The page of a symbol with its DXF drawing, in four languages.
curl -fsS "$url/symbols/no-contact/" -o symbol.html
grep -qF '<link rel="canonical" href="https://site.example/openecraft/symbols/no-contact/" />' symbol.html
grep -qF '<link rel="alternate" hreflang="es" href="https://site.example/openecraft/symbols/no-contact/es.html" />' symbol.html
if grep -q 'hreflang="ko"' symbol.html; then exit 1; fi
has "$url/symbols/no-contact/de.html" -F '<html lang="de">'
has "$url/symbols/art/no.dxf" -F 'ENTITIES'
has "$url/reference/ip-ratings/" -F 'IEC 60529'
has "$url/examples/conveyor/" -F 'page-1.svg'
curl -fsS -o /dev/null "$url/examples/conveyor/page-1.svg"
has "$url/examples/conveyor/conveyor.pdf" -a '^%PDF'
has "$url/faq/" -F '"@type": "FAQPage"'
has "$url/how-to/wire-list/fr.html" -F '<html lang="fr">'
has "$url/compare/eplan-alternative/" -F '<link rel="canonical" href="https://site.example/openecraft/compare/eplan-alternative/" />'
has "$url/sitemap.xml" -F '<loc>https://site.example/openecraft/symbols/contacts/ko.html</loc>'
# The documentation is in four languages, the news in English only: no other
# languages named.
curl -fsS "$url/docs/tutorial/" -o tutorial.html
grep -qF '<link rel="canonical" href="https://site.example/openecraft/docs/tutorial/" />' tutorial.html
grep -qF '<link rel="alternate" hreflang="fr" href="https://site.example/openecraft/docs/tutorial/fr.html" />' tutorial.html
if grep -q 'hreflang="ko"' tutorial.html; then exit 1; fi
has "$url/docs/tutorial/de.html" -F '<html lang="de">'
has "$url/sitemap.xml" -F '<loc>https://site.example/openecraft/docs/tutorial/</loc>'
has "$url/sitemap.xml" -F '<loc>https://site.example/openecraft/docs/tutorial/es.html</loc>'
curl -fsS "$url/news/" -o news.html
grep -qF 'feed.xml' news.html
if grep -q 'hreflang=' news.html; then exit 1; fi
has "$url/news/feed.xml" -F '<feed xml:base="https://site.example/openecraft/news/" xmlns="http://www.w3.org/2005/Atom">'
# The tools get the module from the script of the build.
has "$url/tools/voltage-drop-calculator/" -F 'data-tool="voltage_drop"'
has "$url/tools/cable-size-calculator/de.html" -F 'data-tool="cable_size"'
has "$url/build.js" '"module":\["openecraft-website-[0-9a-f]*\.js","openecraft-website-[0-9a-f]*_bg\.wasm"\]'
curl -fsS -o /dev/null "$url/tools.js"
# The module is named in the page and loaded by its script; the first sheet is a picture.
grep -q 'window.OPENECRAFT_MODULE = \["./openecraft-website-[0-9a-f]*\.js", "./openecraft-website-[0-9a-f]*_bg\.wasm"\]' site.html
if grep -q 'rel="modulepreload"' site.html; then exit 1; fi
curl -fsS -o /dev/null "$url/sheet.svg"
# The log has no whole address of a visitor.
docker logs "$name" > log.txt 2>&1
grep -q '"GET /de.html HTTP/1.1" 200' log.txt
grep 'GET /de.html' log.txt > visits.txt
if grep -vqE '^[0-9]+\.[0-9]+\.[0-9]+\.0 ' visits.txt; then exit 1; fi
# The routings of the examples come with the page.
has "$url/routing/index.json" '"conveyor.json"'
has "$url/routing/conveyor.json" '"routing"'
wasm=$(grep -o '[^"/]*_bg\.wasm' site.html | awk 'NR == 1')
test -n "$wasm"
type=$(curl -fsS -o /dev/null -w '%{content_type}' "$url/$wasm")
echo "$wasm: $type"
test "$type" = application/wasm
for file in site.js icon.svg apple-touch-icon.png images/schematic.webp social.jpg LICENSE-MIT THIRD-PARTY-LICENSES.html; do
  curl -fsS -o /dev/null "$url/$file"
done
# The license files are kept out of the index of a search engine, the page is not.
for file in LICENSE-MIT OFL-NotoSans.txt THIRD-PARTY-LICENSES.html editor/LICENSE-MIT; do
  curl -fsS -o /dev/null -D headers.txt "$url/$file"
  grep -qi '^x-robots-tag: noindex' headers.txt
  grep -qi '^x-content-type-options: nosniff' headers.txt
done
curl -fsS -o /dev/null -D headers.txt "$url/LICENSE-MIT"
grep -qi '^content-type: text/plain' headers.txt
curl -fsS -o /dev/null -D headers.txt "$url/"
if grep -qi '^x-robots-tag' headers.txt; then exit 1; fi
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
grep -qF '<meta name="description"' editor.html
editor=$(grep -o '[^"/]*_bg\.wasm' editor.html | awk 'NR == 1')
test -n "$editor"
curl -fsS -o /dev/null -D headers.txt -H 'Accept-Encoding: br' "$url/editor/$editor"
grep -qi '^content-encoding: br' headers.txt
# trunk names the script after its own hash, not after the one of the WebAssembly.
script=$(grep -o 'openecraft-[0-9a-f]*\.js' editor.html | awk 'NR == 1')
test -n "$script"
curl -fsS -o /dev/null -D headers.txt -H 'Accept-Encoding: br' "$url/editor/$script"
grep -qi '^content-encoding: br' headers.txt
grep -qi '^content-type: application/javascript' headers.txt
docker logs "$name"
echo "$image serves the website"
