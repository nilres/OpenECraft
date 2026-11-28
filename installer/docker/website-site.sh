#!/bin/sh
# The parts of the OpenECraft website that need its public address (docs/website.md):
# the canonical address, the addresses of the page in the other languages and the
# preview image of index.html, of the pages of the languages (de.html, …) and of the
# pages beside it (compare/index.html, compare/de.html, symbols/contacts/index.html, …;
# between their markers "site address"), the addresses of the site in their structured
# data, robots.txt with the sitemap, sitemap.xml with the day every page last changed
# (pages.tsv of the build), the feed of the news (news/feed.xml), which names the
# address its links start from, llms.txt with its links, and the pages for an address
# that has none (404.html, 404.de.html, …), which name the folder of the site. The
# address is taken from OPENECRAFT_SITE_URL; without one the pages, the feed and
# llms.txt stay as built, robots.txt names no sitemap and there is no sitemap.xml.
#
#   website-site.sh [<folder of the built page> [<output folder>]]
#
# The Docker image runs it at every start with the defaults (installer/docker/Dockerfile,
# nginx answers with the files of /tmp/openecraft), and so does the image of the web
# editor for its one page; for a page deployed by hand run it once on the built page:
# OPENECRAFT_SITE_URL=https://… sh website-site.sh website/dist website/dist
set -eu
page=${1:-/usr/share/nginx/html}
out=${2:-/tmp/openecraft}
url=${OPENECRAFT_SITE_URL:-}
case "$url" in
    "") ;;
    http://*|https://*) case "$url" in */) ;; *) url="$url/" ;; esac ;;
    *) echo "website-site.sh: OPENECRAFT_SITE_URL is no http:// or https:// address, ignored: $url" >&2; url= ;;
esac
mkdir -p "$out"

# The address as an attribute value of HTML and XML, and inside a string of JSON.
esc=$(printf '%s' "$url" | sed 's/&/\&amp;/g; s/"/\&quot;/g; s/</\&lt;/g; s/>/\&gt;/g')
json=$(printf '%s' "$url" | sed 's/\\/\\\\/g; s/"/\\"/g')

# The address the page has already: the script run again on its own output. What was
# written whole with it is set back first, so that another address replaces it.
before=$(sed -n 's/.*<link rel="canonical" href="\([^"]*\)".*/\1/p' "$page/index.html" | sed -n 1p)
before=$(printf '%s' "$before" | sed 's/&quot;/"/g; s/&lt;/</g; s/&gt;/>/g; s/&amp;/\&/g')
before_json=$(printf '%s' "$before" | sed 's/\\/\\\\/g; s/"/\\"/g')

# For awk: `line` with every `from` replaced by `to`, both taken as they are written.
replaced='function replaced(line, from, to,    out, at) {
    out = ""
    while ((at = index(line, from)) > 0) {
        out = out substr(line, 1, at - 1) to
        line = substr(line, at + length(from))
    }
    return out line
}'

# The languages the page was built in beside English: de.html, fr.html, …
langs=
for file in "$page"/[a-z][a-z].html; do
    if [ -f "$file" ]; then langs="$langs $(basename "$file" .html)"; fi
done

# The pages beside it (compare/, symbols/contacts/, …; website/src/pages.rs and the
# pages written by the build, crates/openecraft-site): folders one or two deep whose
# index.html has the markers, each in the languages it was written in.
pages=
for file in "$page"/*/index.html "$page"/*/*/index.html; do
    if [ -f "$file" ] && grep -q '<!-- site address ' "$file"; then
        dir=${file#"$page"/}
        pages="$pages ${dir%/index.html}"
    fi
done

# The web editor in the folder editor/ beside the page is a page of the site, unless
# the links of the site lead to a web editor elsewhere (OPENECRAFT_EDITOR_URL of the
# image, an address): then that one is the page to find, which this copy names as its
# original, and the copy is left out of the sitemap.
editor=
case "${OPENECRAFT_EDITOR_URL:-}" in
    http://*|https://*) editor=$(printf '%s' "$OPENECRAFT_EDITOR_URL" | sed 's/&/\&amp;/g; s/"/\&quot;/g; s/</\&lt;/g; s/>/\&gt;/g') ;;
esac

# One page with its tags: $1 the file, $2 its address below the site ("" or de.html),
# $3 the folder of its languages ("" or compare/), $4 the languages it has beside
# English (those of the main page if left out), $5 its address where that is not one
# of the site. The addresses of the site in its structured data (JSON-LD) start at the
# folder of the site, seen from the page ("./", "../../"): they are written whole.
tags() {
    SITE_URL=$esc SITE_JSON=$json SITE_BEFORE=$before_json SITE_PATH=$2 SITE_BASE=${3:-} SITE_LANGS=${4-$langs} SITE_OWN=${5:-} awk "$replaced"'
BEGIN {
    url = ENVIRON["SITE_URL"]; json = ENVIRON["SITE_JSON"]; before = ENVIRON["SITE_BEFORE"]; folder = ENVIRON["SITE_BASE"]
    own = ENVIRON["SITE_OWN"]; base = url folder; count = split(ENVIRON["SITE_LANGS"], langs, " ")
    if (own == "") own = url ENVIRON["SITE_PATH"]
    depth = gsub("/", "/", folder)
    site = ""
    for (i = 0; i < depth; i++) site = site "../"
    if (site == "") site = "./"
    value = "\": \""
}
/<!-- \/site address -->/ { skip = 0 }
skip { next }
index($0, "<script type=\"application/ld+json\">") { data = 1 }
index($0, "</script>") { data = 0 }
data {
    if (before != "") $0 = replaced($0, value before, value site)
    if (json != "") $0 = replaced($0, value site, value json)
}
{ print }
/<!-- site address / {
    skip = 1
    if (url == "") next
    match($0, /^[ \t]*/)
    indent = substr($0, 1, RLENGTH)
    print indent "<link rel=\"canonical\" href=\"" own "\" />"
    if (count > 0) {
        print indent "<link rel=\"alternate\" hreflang=\"x-default\" href=\"" base "\" />"
        print indent "<link rel=\"alternate\" hreflang=\"en\" href=\"" base "\" />"
        for (i = 1; i <= count; i++) print indent "<link rel=\"alternate\" hreflang=\"" langs[i] "\" href=\"" base langs[i] ".html\" />"
    }
    print indent "<meta property=\"og:url\" content=\"" own "\" />"
    print indent "<meta property=\"og:image\" content=\"" url "social.jpg\" />"
    print indent "<meta property=\"og:image:width\" content=\"1200\" />"
    print indent "<meta property=\"og:image:height\" content=\"630\" />"
    print indent "<meta name=\"twitter:image\" content=\"" url "social.jpg\" />"
}' "$page/$1" > "$out/$1.new"
    mv "$out/$1.new" "$out/$1"
}

tags index.html ""
for lang in $langs; do tags "$lang.html" "$lang.html"; done
for dir in $pages; do
    mkdir -p "$out/$dir"
    # The documentation and the news are in English only.
    own=
    for lang in $langs; do
        if [ -f "$page/$dir/$lang.html" ]; then own="$own $lang"; fi
    done
    if [ "$dir" = editor ]; then own_address=$editor; else own_address=; fi
    tags "$dir/index.html" "$dir/" "$dir/" "$own" "$own_address"
    for lang in $own; do tags "$dir/$lang.html" "$dir/$lang.html" "$dir/" "$own"; done
done

# The pages for an address that has none (crates/openecraft-site, missing): shown for
# an address of any depth, so they name the folder their addresses start from, the
# public address or, without one, the root of the host.
for file in "$page"/404.html "$page"/404.[a-z][a-z].html; do
    [ -f "$file" ] || continue
    name=$(basename "$file")
    SITE_URL=${esc:-/} awk '
/<!-- \/site address -->/ { skip = 0 }
skip { next }
{ print }
/<!-- site address / {
    skip = 1
    match($0, /^[ \t]*/)
    print substr($0, 1, RLENGTH) "<base href=\"" ENVIRON["SITE_URL"] "\" />"
}' "$file" > "$out/$name.new"
    mv "$out/$name.new" "$out/$name"
done

# The feed: its links are relative to the folder of the news, named with the address
# (its first element, on a line of its own; run again, the address is replaced).
if [ -f "$page/news/feed.xml" ]; then
    mkdir -p "$out/news"
    SITE_URL=$esc awk '
BEGIN { url = ENVIRON["SITE_URL"] }
/^<feed / { $0 = "<feed" (url == "" ? "" : " xml:base=\"" url "news/\"") " xmlns=\"http://www.w3.org/2005/Atom\">" }
{ print }' "$page/news/feed.xml" > "$out/news/feed.xml.new"
    mv "$out/news/feed.xml.new" "$out/news/feed.xml"
fi

# The list of the pages for language models: its links start at the folder of the site
# ("./docs/tutorial/") and are written whole.
if [ -f "$page/llms.txt" ]; then
    SITE_URL=$url SITE_BEFORE=$before awk "$replaced"'
BEGIN { url = ENVIRON["SITE_URL"]; before = ENVIRON["SITE_BEFORE"] }
{
    if (before != "") $0 = replaced($0, "](" before, "](./")
    if (url != "") $0 = replaced($0, "](./", "](" url)
    print
}' "$page/llms.txt" > "$out/llms.txt.new"
    mv "$out/llms.txt.new" "$out/llms.txt"
fi

{
    printf 'User-agent: *\nAllow: /\n'
    if [ -n "$url" ]; then printf 'Sitemap: %ssitemap.xml\n' "$url"; fi
} > "$out/robots.txt"

# The sitemap: every page with the day it last changed, where the build knows it
# (pages.tsv: file, hash, day; crates/openecraft-site, changes).
if [ -n "$url" ]; then
    {
        printf 'index.html\t\n'
        for lang in $langs; do printf '%s.html\t%s.html\n' "$lang" "$lang"; done
        for dir in $pages; do
            if [ "$dir" = editor ] && [ -n "$editor" ]; then continue; fi
            printf '%s/index.html\t%s/\n' "$dir" "$dir"
            for lang in $langs; do
                if [ -f "$page/$dir/$lang.html" ]; then printf '%s/%s.html\t%s/%s.html\n' "$dir" "$lang" "$dir" "$lang"; fi
            done
        done
    } | SITE_URL=$esc SITE_DAYS="$page/pages.tsv" awk -F '\t' '
BEGIN {
    url = ENVIRON["SITE_URL"]; days = ENVIRON["SITE_DAYS"]
    while ((getline line < days) > 0) {
        sub(/\r$/, "", line)
        if (split(line, field, "\t") == 3 && field[3] ~ /^[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]$/) day[field[1]] = field[3]
    }
    print "<?xml version=\"1.0\" encoding=\"UTF-8\"?>"
    print "<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">"
}
{ printf "  <url><loc>%s%s</loc>%s</url>\n", url, $2, ($1 in day) ? "<lastmod>" day[$1] "</lastmod>" : "" }
END { print "</urlset>" }' > "$out/sitemap.xml.new"
    mv "$out/sitemap.xml.new" "$out/sitemap.xml"
else
    rm -f "$out/sitemap.xml"
fi
