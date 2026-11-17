#!/bin/sh
# Compressed copies of the files of a web build beside them, made once by the build of
# the Docker images (installer/docker/Dockerfile) and sent by nginx as they are instead
# of compressing them for every visitor (editor-web.conf, website.conf): file.gz of the
# WebAssembly, the scripts and the other big texts, and file.br (Brotli, a good third
# smaller, which browsers ask for over HTTPS) of the WebAssembly and its script, which
# trunk names after their hash. config.js is written at every start and has no copy.
#
#   sh installer/docker/compress.sh <folder>...
set -eu
jobs=$(nproc 2> /dev/null || echo 1)
find "$@" -type f \
    \( -name '*.wasm' -o -name '*.js' -o -name '*.css' -o -name '*.json' -o -name '*.tsv' -o -name '*.svg' -o -name 'THIRD-PARTY-LICENSES.html' \) \
    ! -name config.js -print0 | xargs -0 -r -n 1 -P "$jobs" gzip -9 -k -f
find "$@" -type f -regex '.*-[0-9a-f]+\(_bg\)?\.\(wasm\|js\)' -print0 \
    | xargs -0 -r -n 1 -P "$jobs" brotli -q 11 -k -f
