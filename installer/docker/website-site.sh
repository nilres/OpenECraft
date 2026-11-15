#!/bin/sh
# The parts of the OpenECraft website that need its public address (docs/website.md):
# the canonical address, the addresses of the page in the other languages and the
# preview image of index.html and of the pages of the languages (de.html, …; between
# their markers "site address"), robots.txt with the sitemap, and sitemap.xml. The
# address is taken from OPENECRAFT_SITE_URL; without one the pages stay as built,
# robots.txt names no sitemap and there is no sitemap.xml.
#
#   website-site.sh [<folder of the built page> [<output folder>]]
#
# The Docker image runs it at every start with the defaults (installer/docker/Dockerfile,
# nginx answers with the files of /tmp/openecraft); for a page deployed by hand run it
# once on the built page: OPENECRAFT_SITE_URL=https://… sh website-site.sh website/dist website/dist
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

# The address as an attribute value of HTML and XML.
esc=$(printf '%s' "$url" | sed 's/&/\&amp;/g; s/"/\&quot;/g; s/</\&lt;/g; s/>/\&gt;/g')

# The languages the page was built in beside English: de.html, fr.html, …
langs=
for file in "$page"/[a-z][a-z].html; do
    if [ -f "$file" ]; then langs="$langs $(basename "$file" .html)"; fi
done

# One page with its tags: $1 the file, $2 its address below the site ("" or de.html).
tags() {
    SITE_URL=$esc SITE_PATH=$2 SITE_LANGS=$langs awk '
BEGIN { url = ENVIRON["SITE_URL"]; own = url ENVIRON["SITE_PATH"]; count = split(ENVIRON["SITE_LANGS"], langs, " ") }
/<!-- \/site address -->/ { skip = 0 }
skip { next }
{ print }
/<!-- site address / {
    skip = 1
    if (url == "") next
    match($0, /^[ \t]*/)
    indent = substr($0, 1, RLENGTH)
    print indent "<link rel=\"canonical\" href=\"" own "\" />"
    if (count > 0) {
        print indent "<link rel=\"alternate\" hreflang=\"x-default\" href=\"" url "\" />"
        print indent "<link rel=\"alternate\" hreflang=\"en\" href=\"" url "\" />"
        for (i = 1; i <= count; i++) print indent "<link rel=\"alternate\" hreflang=\"" langs[i] "\" href=\"" url langs[i] ".html\" />"
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

{
    printf 'User-agent: *\nAllow: /\n'
    if [ -n "$url" ]; then printf 'Sitemap: %ssitemap.xml\n' "$url"; fi
} > "$out/robots.txt"

if [ -n "$url" ]; then
    {
        printf '<?xml version="1.0" encoding="UTF-8"?>\n'
        printf '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n'
        printf '  <url><loc>%s</loc></url>\n' "$esc"
        for lang in $langs; do printf '  <url><loc>%s%s.html</loc></url>\n' "$esc" "$lang"; done
        printf '</urlset>\n'
    } > "$out/sitemap.xml"
else
    rm -f "$out/sitemap.xml"
fi
