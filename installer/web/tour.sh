#!/bin/sh
# The screen recording of the website and its still picture (website/tour.webm,
# tour.jpg; docs/website.md) are no files of the repository: they are assets of the
# release `website-media`. Run from the repository root before the website is built:
# fetches those that are not there. Without them the site is built without a recording,
# so a failed download only warns. OPENECRAFT_TOUR_URL names another folder; empty: none.
url=${OPENECRAFT_TOUR_URL-https://github.com/nilres/OpenECraft/releases/download/website-media}
[ -n "$url" ] || exit 0
for file in tour.webm tour.jpg; do
    [ -s "website/$file" ] && continue
    if curl -fsSL --retry 2 -o "website/$file.part" "$url/$file"; then
        mv "website/$file.part" "website/$file"
        echo "fetched website/$file"
    else
        rm -f "website/$file.part"
        echo "warning: no $file at $url, the website is built without it" >&2
    fi
done
