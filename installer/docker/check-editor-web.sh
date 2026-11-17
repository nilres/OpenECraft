#!/usr/bin/env bash
# Starts the image of the web editor once and checks that it serves the page, the
# WebAssembly as application/wasm and as the build compressed it, the license files,
# the worker, the routings of the examples and the legal notice of the operator from
# the environment; used by the CI workflows (ci.yml, docker-web.yml).
#
#   bash installer/docker/check-editor-web.sh [image] [port]
set -euo pipefail

image=${1:-openecraft-web:ci}
port=${2:-8080}
name=openecraft-web-check
work=$(mktemp -d)
trap 'docker rm -f "$name" > /dev/null 2>&1 || true; rm -rf "$work"' EXIT

docker run -d --name "$name" -p "$port:8080" -e OPENECRAFT_LEGAL_NAME='ACME "Tools" GmbH' "$image" > /dev/null
for _ in $(seq 30); do curl -fs "http://127.0.0.1:$port/" > /dev/null && break; sleep 1; done
curl -fsS "http://127.0.0.1:$port/" -o "$work/index.html" || { docker logs "$name"; exit 1; }
grep -q openecraft_canvas "$work/index.html"
wasm=$(grep -o '[^"/]*_bg\.wasm' "$work/index.html" | head -n 1)
test -n "$wasm"
type=$(curl -fsS -o /dev/null -w '%{content_type}' "http://127.0.0.1:$port/$wasm")
echo "$wasm: $type"
test "$type" = application/wasm
for file in LICENSE-MIT LICENSE-APACHE OFL-NotoSans.txt THIRD-PARTY-LICENSES.html; do
  curl -fsS -o /dev/null "http://127.0.0.1:$port/$file"
done
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
script=${wasm%_bg.wasm}.js
curl -fsS -o /dev/null -D "$work/headers.txt" -H 'Accept-Encoding: br' "http://127.0.0.1:$port/$script"
grep -qi '^content-encoding: br' "$work/headers.txt"
grep -qi '^content-type: application/javascript' "$work/headers.txt"
curl -fsS -o /dev/null -D "$work/headers.txt" -H 'Accept-Encoding: br' "http://127.0.0.1:$port/worker.js"
grep -qi '^cache-control: no-cache' "$work/headers.txt"
if grep -qi '^content-encoding: br' "$work/headers.txt"; then exit 1; fi
grep -q 'src="config.js"' "$work/index.html"
curl -fsS "http://127.0.0.1:$port/config.js" | grep -qF 'name: "ACME \"Tools\" GmbH",'
docker logs "$name"
echo "$image serves the web editor"
