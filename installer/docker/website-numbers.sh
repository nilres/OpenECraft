#!/bin/sh
# The numbers of the project on the first screen of the OpenECraft website (stars and
# contributors on GitHub; website/numbers.js, docs/website.md), written at every start
# of its Docker image and kept current while it runs; nginx answers /numbers.js with
# the file (website.conf). It starts with the numbers the image was built with
# (OPENECRAFT_STARS, OPENECRAFT_CONTRIBUTORS) and asks GitHub for those of the
# repository of the page (OPENECRAFT_REPOSITORY_URL) at once and then every
# OPENECRAFT_NUMBERS_INTERVAL seconds (3600; 0: never). A number GitHub does not give
# stays as it was; the visitors of the page never ask GitHub themselves.
#
#   sh website-numbers.sh          the file, and the loop in the background
#   sh website-numbers.sh fetch    asks once and writes the file
set -u
file=${OPENECRAFT_NUMBERS_FILE:-/tmp/openecraft/numbers.js}
api=${OPENECRAFT_NUMBERS_API:-https://api.github.com}
interval=${OPENECRAFT_NUMBERS_INTERVAL:-3600}

# Digits only, 0 for anything else.
number() {
    case ${1:-} in
        '' | *[!0-9]*) echo 0 ;;
        *) echo "$1" ;;
    esac
}
stars=$(number "${OPENECRAFT_STARS:-}")
contributors=$(number "${OPENECRAFT_CONTRIBUTORS:-}")
interval=$(number "$interval")

# owner/name of a repository on GitHub, empty for any other address.
repository=
case ${OPENECRAFT_REPOSITORY_URL:-} in
    https://github.com/*/*)
        repository=${OPENECRAFT_REPOSITORY_URL#https://github.com/}
        repository=${repository%/}
        repository=${repository%.git}
        case $repository in
            */*/* | *[!A-Za-z0-9._/-]*) repository= ;;
        esac
        ;;
esac

write() {
    mkdir -p "$(dirname "$file")"
    printf 'window.OPENECRAFT_NUMBERS = { stars: %s, contributors: %s };\n' "$stars" "$contributors" > "$file.new" && mv "$file.new" "$file"
}

# The stars are a number of the repository; the contributors are counted by the last
# page of their list with one on a page (no Link header: a single page).
fetch() {
    answer=$(curl -fsS --max-time 20 -H 'Accept: application/vnd.github+json' "$api/repos/$repository") || return 1
    value=$(printf '%s\n' "$answer" | sed -n 's/^ *"stargazers_count": *\([0-9][0-9]*\).*/\1/p' | head -n 1)
    [ -n "$value" ] && stars=$value
    answer=$(curl -fsS --max-time 20 -I -H 'Accept: application/vnd.github+json' "$api/repos/$repository/contributors?per_page=1&anon=true" | tr -d '\r') || return 1
    value=$(printf '%s\n' "$answer" | grep -i '^link:' | sed -n 's/.*[?&]page=\([0-9][0-9]*\)>; *rel="last".*/\1/p' | head -n 1)
    [ -n "$value" ] || value=1
    contributors=$value
}

if [ "${1:-}" = fetch ]; then
    [ -n "$repository" ] || { echo "website-numbers: OPENECRAFT_REPOSITORY_URL is no repository on GitHub" >&2; exit 1; }
    fetch || { echo "website-numbers: no answer from $api" >&2; exit 1; }
    write
    exit 0
fi

write
if [ "$interval" -gt 0 ] && [ -n "$repository" ] && command -v curl > /dev/null; then
    (
        while :; do
            if fetch; then write; else echo "website-numbers: no answer from $api, the numbers stay" >&2; fi
            sleep "$interval"
        done
    ) < /dev/null &
fi
exit 0
