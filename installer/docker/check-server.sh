#!/usr/bin/env bash
# Starts the image of the parts server or of the project server once and checks that
# it answers and brings the third-party licenses; used by the CI workflows (ci.yml,
# docker-parts-server.yml, docker-project-server.yml).
#
#   bash installer/docker/check-server.sh parts|project [image] [port]
set -euo pipefail

case "${1:-}" in
  parts)
    image=${2:-openecraft-parts-server:ci}
    inner=8090
    token=OPENECRAFT_TOKEN
    args=(--install-catalogs all)
    ;;
  project)
    image=${2:-openecraft-project-server:ci}
    inner=8091
    token=OPENECRAFT_PROJECT_TOKEN
    args=()
    ;;
  *)
    echo "usage: check-server.sh parts|project [image] [port]" >&2
    exit 2
    ;;
esac
port=${3:-$inner}
name=openecraft-$1-server-check
trap 'docker rm -f "$name" > /dev/null 2>&1 || true' EXIT

docker run -d --name "$name" -p "$port:$inner" -e "$token=ci-token" "$image" "${args[@]}" > /dev/null
for _ in $(seq 60); do curl -fs "http://127.0.0.1:$port/" > /dev/null && break; sleep 1; done
curl -fsS "http://127.0.0.1:$port/" > /dev/null || { docker logs "$name"; exit 1; }
docker exec "$name" ls /usr/share/doc/openecraft/THIRD-PARTY-LICENSES.html
docker logs "$name"
echo "$image serves the $1 server"
