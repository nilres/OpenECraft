#!/bin/sh
# Tells search engines of the pages of the OpenECraft website that changed with its
# build (IndexNow: Bing, Yandex, Seznam, Naver and others; docs/website.md), so that
# they read them now instead of at their next visit. The pages are those pages.tsv of
# the build gives the day it was built (crates/openecraft-site, changes); a build that
# changed nothing tells nothing. Needs a key, OPENECRAFT_INDEXNOW_KEY (8 to 128
# letters, digits and dashes, of your own choice: `openssl rand -hex 16`), and the
# public address of the site, OPENECRAFT_SITE_URL: the key is written beside the page
# (indexnow.txt; nginx answers /indexnow.txt with it, website.conf), where the search
# engines read it to see that the site itself is asking. Without a key nothing is
# written or sent. Nothing of a visitor is part of it: the request names addresses of
# the site.
#
#   sh website-indexnow.sh [<folder of the built page> [<output folder>]]
#                                 the key, and the request once the site answers
#   sh website-indexnow.sh <folder> <output folder> key     the key only
#   sh website-indexnow.sh <folder> <output folder> send    the request only, at once
#
# The Docker image runs it at every start (installer/docker/Dockerfile); a start of the
# same container tells of a build only once. For a site deployed by hand write the key
# before the upload (key) and send the request after it (send).
set -u
page=${1:-/usr/share/nginx/html}
out=${2:-/tmp/openecraft}
mode=${3:-}
key=${OPENECRAFT_INDEXNOW_KEY:-}
url=${OPENECRAFT_SITE_URL:-}
api=${OPENECRAFT_INDEXNOW_API:-https://api.indexnow.org/indexnow}
site=${OPENECRAFT_INDEXNOW_SITE:-http://127.0.0.1:8080}

mkdir -p "$out"
[ "$mode" = send ] || rm -f "$out/indexnow.txt"
case $key in
    '') exit 0 ;;
    *[!A-Za-z0-9-]*) echo "website-indexnow: OPENECRAFT_INDEXNOW_KEY takes letters, digits and dashes only, ignored" >&2; exit 0 ;;
esac
if [ ${#key} -lt 8 ] || [ ${#key} -gt 128 ]; then
    echo "website-indexnow: OPENECRAFT_INDEXNOW_KEY takes 8 to 128 characters, ignored" >&2
    exit 0
fi
[ "$mode" = send ] || printf '%s' "$key" > "$out/indexnow.txt"
[ "$mode" = key ] && exit 0

case $url in
    *'"'* | *'\'*) echo "website-indexnow: OPENECRAFT_SITE_URL is no address to send, nothing sent" >&2; exit 0 ;;
    http://*|https://*) case $url in */) ;; *) url="$url/" ;; esac ;;
    *) echo "website-indexnow: no OPENECRAFT_SITE_URL, nothing sent" >&2; exit 0 ;;
esac
[ -f "$page/pages.tsv" ] || exit 0
command -v curl > /dev/null || { echo "website-indexnow: no curl, nothing sent" >&2; exit 0; }

# The request: the pages of the day of the build, at most as many as one request takes.
count=$(SITE_URL=$url SITE_KEY=$key SITE_REQUEST="$out/indexnow.json" awk -F '\t' '
BEGIN { url = ENVIRON["SITE_URL"]; host = url; sub(/^[a-z]+:\/\//, "", host); sub(/\/.*$/, "", host) }
{ sub(/\r$/, "") }
/^# built / { built = substr($0, 9); next }
/^#/ { next }
$3 != "" && $3 == built && n < 10000 {
    address = $1
    sub(/index\.html$/, "", address)
    list = list (n++ ? "," : "") "\"" url address "\""
}
END {
    request = ENVIRON["SITE_REQUEST"]
    if (n) printf "{\"host\":\"%s\",\"key\":\"%s\",\"keyLocation\":\"%sindexnow.txt\",\"urlList\":[%s]}\n", host, ENVIRON["SITE_KEY"], url, list > request
    print n + 0
}' "$page/pages.tsv")
if [ "${count:-0}" -eq 0 ]; then
    rm -f "$out/indexnow.json"
    exit 0
fi

# A start of the same container does not tell of the same build again.
build=$(cksum < "$page/pages.tsv")
if [ "$mode" != send ] && [ "$(cat "$out/indexnow.sent" 2> /dev/null)" = "$build" ]; then exit 0; fi

send() {
    if curl -fsS --max-time 60 -o /dev/null -H 'Content-Type: application/json; charset=utf-8' --data-binary "@$out/indexnow.json" "$api"; then
        printf '%s\n' "$build" > "$out/indexnow.sent"
        echo "website-indexnow: told $api of $count changed pages"
    else
        echo "website-indexnow: no answer from $api, nothing told" >&2
        return 1
    fi
}

if [ "$mode" = send ]; then
    send
    exit $?
fi
(
    # The search engines read the key from the site: wait until it answers.
    tries=0
    until curl -fs -o /dev/null "$site/indexnow.txt"; do
        tries=$((tries + 1))
        [ "$tries" -lt 60 ] || { echo "website-indexnow: the site does not answer, nothing told" >&2; exit 0; }
        sleep 2
    done
    send
) < /dev/null &
exit 0
