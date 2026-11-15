#!/bin/sh
# The parts of the OpenECraft website that need its public address (docs/website.md):
# the canonical address and the preview image of index.html (between its markers
# "site address"), robots.txt with the sitemap, and sitemap.xml. The address is taken
# from OPENECRAFT_SITE_URL; without one index.html stays as built, robots.txt names
# no sitemap and there is no sitemap.xml.
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

SITE_URL=$esc awk '
BEGIN { url = ENVIRON["SITE_URL"] }
/<!-- \/site address -->/ { skip = 0 }
skip { next }
{ print }
/<!-- site address / {
    skip = 1
    if (url == "") next
    match($0, /^[ \t]*/)
    indent = substr($0, 1, RLENGTH)
    print indent "<link rel=\"canonical\" href=\"" url "\" />"
    print indent "<meta property=\"og:url\" content=\"" url "\" />"
    print indent "<meta property=\"og:image\" content=\"" url "social.jpg\" />"
    print indent "<meta property=\"og:image:width\" content=\"1200\" />"
    print indent "<meta property=\"og:image:height\" content=\"630\" />"
    print indent "<meta name=\"twitter:image\" content=\"" url "social.jpg\" />"
}' "$page/index.html" > "$out/index.html.new"
mv "$out/index.html.new" "$out/index.html"

{
    printf 'User-agent: *\nAllow: /\n'
    if [ -n "$url" ]; then printf 'Sitemap: %ssitemap.xml\n' "$url"; fi
} > "$out/robots.txt"

if [ -n "$url" ]; then
    {
        printf '<?xml version="1.0" encoding="UTF-8"?>\n'
        printf '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n'
        printf '  <url><loc>%s</loc></url>\n' "$esc"
        printf '</urlset>\n'
    } > "$out/sitemap.xml"
else
    rm -f "$out/sitemap.xml"
fi
