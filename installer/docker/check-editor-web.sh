#!/usr/bin/env bash
# Starts the image of the web editor once and checks that it serves the page, the
# WebAssembly as application/wasm and the license files; used by the CI workflows
# (ci.yml, docker-web.yml).
#
#   bash installer/docker/check-editor-web.sh [image] [port]
set -euo pipefail

image=${1:-openecraft-web:ci}
port=${2:-8080}
name=openecraft-web-check
work=$(mktemp -d)
trap 'docker rm -f "$name" > /dev/null 2>&1 || true; rm -rf "$work"' EXIT

docker run -d --name "$name" -p "$port:8080" "$image" > /dev/null
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
docker logs "$name"
echo "$image serves the web editor"
