#!/bin/sh
# The days the pages of the website last changed are kept from build to build in the
# site itself: pages.tsv beside its main page (docs/website.md; crates/openecraft-site,
# changes). Run from the repository root before the website is built: fetches the file
# of the site that is online into website/pages.tsv, against which the build compares
# its pages. OPENECRAFT_PAGES_URL is its address (https://…/pages.tsv); empty: none,
# and a file that is there stays. Without the file no page has a day, so a failed
# download only warns: a sitemap without days is better than one with wrong ones.
url=${OPENECRAFT_PAGES_URL-}
[ -n "$url" ] || exit 0
rm -f website/pages.tsv
if curl -fsSL --retry 2 --max-time 60 -o website/pages.tsv.part "$url" && grep -q '^# built ' website/pages.tsv.part; then
    mv website/pages.tsv.part website/pages.tsv
    echo "fetched website/pages.tsv"
else
    rm -f website/pages.tsv.part
    echo "warning: no pages.tsv at $url, the pages of the website are built without the days of their last change" >&2
fi
