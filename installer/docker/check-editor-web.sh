#!/usr/bin/env bash
# Starts the image of the web editor once and checks that it serves the page, the
# WebAssembly as application/wasm and as the build compressed it, the license files,
# the worker, the routings of the examples, the legal notice of the operator and the
# public address from the environment; used by the CI workflows (ci.yml, docker-web.yml).
#
#   bash installer/docker/check-editor-web.sh [image] [port]
set -euo pipefail

image=${1:-openecraft-web:ci}
port=${2:-8080}
name=openecraft-web-check
work=$(mktemp -d)
trap 'docker rm -f "$name" > /dev/null 2>&1 || true; rm -rf "$work"' EXIT

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

docker run -d --name "$name" -p "$port:8080" -e OPENECRAFT_LEGAL_NAME='ACME "Tools" GmbH' \
  -e OPENECRAFT_SITE_URL=https://editor.example "$image" > /dev/null
for _ in $(seq 30); do curl -fs "http://127.0.0.1:$port/" > /dev/null && break; sleep 1; done
curl -fsS "http://127.0.0.1:$port/" -o "$work/index.html" || { docker logs "$name"; exit 1; }
grep -q openecraft_canvas "$work/index.html"
wasm=$(grep -o '[^"/]*_bg\.wasm' "$work/index.html" | awk 'NR == 1')
test -n "$wasm"
type=$(curl -fsS -o /dev/null -w '%{content_type}' "http://127.0.0.1:$port/$wasm")
echo "$wasm: $type"
test "$type" = application/wasm
# The license files are kept out of the index of a search engine, the page is not.
for file in LICENSE-MIT LICENSE-APACHE OFL-NotoSans.txt THIRD-PARTY-LICENSES.html; do
  curl -fsS -o /dev/null -D "$work/headers.txt" "http://127.0.0.1:$port/$file"
  grep -qi '^x-robots-tag: noindex' "$work/headers.txt"
done
curl -fsS -o /dev/null -D "$work/headers.txt" "http://127.0.0.1:$port/"
if grep -qi '^x-robots-tag' "$work/headers.txt"; then exit 1; fi
grep -qF '<meta name="description"' "$work/index.html"
# The tags of the public address, robots.txt, the sitemap and the preview image.
grep -qF '<link rel="canonical" href="https://editor.example/" />' "$work/index.html"
grep -qF '<meta property="og:image" content="https://editor.example/social.jpg" />' "$work/index.html"
has "http://127.0.0.1:$port/index.html" -F 'rel="canonical"'
has "http://127.0.0.1:$port/robots.txt" -x 'Sitemap: https://editor.example/sitemap.xml'
has "http://127.0.0.1:$port/sitemap.xml" -F '<url><loc>https://editor.example/</loc></url>'
curl -fsS -o /dev/null "http://127.0.0.1:$port/social.jpg"
# The worker that searches the 3D routing of an opened project, the routings of the
# examples from the build and the catalog of a language the page fetches when it is
# used.
for file in worker.js routing/index.json i18n/ja.tsv; do
  curl -fsS -o /dev/null "http://127.0.0.1:$port/$file"
done
# The WebAssembly is sent as the build of the image compressed it, with Brotli to who
# takes it, and is cached for long; the worker, which has no hash in its name, is not.
curl -fsS -o "$work/wasm.gz" -D "$work/headers.txt" -H 'Accept-Encoding: gzip' "http://127.0.0.1:$port/$wasm"
grep -qi '^content-encoding: gzip' "$work/headers.txt"
grep -qi '^cache-control: .*immutable' "$work/headers.txt"
gzip -t "$work/wasm.gz"
curl -fsS -o "$work/wasm.br" -D "$work/headers.txt" -H 'Accept-Encoding: gzip, deflate, br, zstd' "http://127.0.0.1:$port/$wasm"
grep -qi '^content-encoding: br' "$work/headers.txt"
grep -qi '^content-type: application/wasm' "$work/headers.txt"
grep -qi '^vary: accept-encoding' "$work/headers.txt"
grep -qi '^cache-control: .*immutable' "$work/headers.txt"
test "$(wc -c < "$work/wasm.br")" -lt "$(wc -c < "$work/wasm.gz")"
# trunk names the script after its own hash, not after the one of the WebAssembly.
script=$(grep -o 'openecraft-[0-9a-f]*\.js' "$work/index.html" | awk 'NR == 1')
test -n "$script"
curl -fsS -o /dev/null -D "$work/headers.txt" -H 'Accept-Encoding: br' "http://127.0.0.1:$port/$script"
grep -qi '^content-encoding: br' "$work/headers.txt"
grep -qi '^content-type: application/javascript' "$work/headers.txt"
curl -fsS -o /dev/null -D "$work/headers.txt" -H 'Accept-Encoding: br' "http://127.0.0.1:$port/worker.js"
grep -qi '^cache-control: no-cache' "$work/headers.txt"
if grep -qi '^content-encoding: br' "$work/headers.txt"; then exit 1; fi
grep -q 'src="config.js"' "$work/index.html"
has "http://127.0.0.1:$port/config.js" -F 'name: "ACME \"Tools\" GmbH",'
docker logs "$name"
echo "$image serves the web editor"
