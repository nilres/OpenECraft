#!/bin/sh
# The videos of the project on the page /videos/ of the OpenECraft website
# (website/content/videos.html, docs/website.md): the list of the videos of a YouTube
# channel with their pictures, written at every start of the Docker image of the site
# and kept current while it runs; nginx answers /videos.js and /videos/<id>.jpg with
# the files (website.conf). The channel is OPENECRAFT_VIDEOS_CHANNEL (its id, UC…;
# empty: no videos), asked for its feed at once and then every
# OPENECRAFT_VIDEOS_INTERVAL seconds (21600, four times a day; 0: never). A feed that
# does not come leaves the list as it was. It is the server that asks: the browser of a
# visitor gets the list and the pictures from the site and reaches YouTube only by
# following a link.
#
#   sh website-videos.sh          the file, and the loop in the background
#   sh website-videos.sh fetch    asks once and writes the file
set -u
dir=${OPENECRAFT_VIDEOS_DIR:-/tmp/openecraft}
feed=${OPENECRAFT_VIDEOS_FEED:-https://www.youtube.com/feeds/videos.xml}
pictures=${OPENECRAFT_VIDEOS_PICTURES:-https://i.ytimg.com/vi}
watch=${OPENECRAFT_VIDEOS_WATCH:-https://www.youtube.com/watch?v=}
interval=${OPENECRAFT_VIDEOS_INTERVAL:-21600}
file=$dir/videos.js

case $interval in
    '' | *[!0-9]*) interval=0 ;;
esac
# The id of a channel: letters, digits, - and _ only.
channel=${OPENECRAFT_VIDEOS_CHANNEL:-}
case $channel in
    *[!A-Za-z0-9_-]*) channel= ;;
esac

empty() {
    mkdir -p "$dir"
    [ -s "$file" ] || printf 'window.OPENECRAFT_VIDEOS = [];\n' > "$file"
}

# The feed (Atom) as the script of the page: of every entry its id, address, title,
# day and description, as JavaScript strings. An id with other characters is left out.
list() {
    WATCH=$watch awk '
function plain(s) {
    gsub(/&lt;/, "<", s); gsub(/&gt;/, ">", s); gsub(/&quot;/, "\"", s); gsub(/&#39;/, "\047", s); gsub(/&apos;/, "\047", s); gsub(/&amp;/, "\\&", s)
    return s
}
function js(s,    out, i, c) {
    s = plain(s)
    out = ""
    for (i = 1; i <= length(s); i++) {
        c = substr(s, i, 1)
        if (c == "\\" || c == "\"") out = out "\\" c
        else if (c == "\n") out = out "\\n"
        else if (c == "<") out = out "\\u003c"
        else if (c != "\r" && c != "\t") out = out c
        else if (c == "\t") out = out " "
    }
    return "\"" out "\""
}
function inner(line, tag,    s) {
    s = line
    sub("^.*<" tag "[^>]*>", "", s)
    sub("</" tag ">.*$", "", s)
    return s
}
BEGIN { print "window.OPENECRAFT_VIDEOS = ["; count = 0 }
/<entry>/ { entry = 1; id = ""; title = ""; day = ""; about = ""; text = 0 }
entry && /<yt:videoId>/ { id = inner($0, "yt:videoId") }
entry && /<title>/ && title == "" { title = inner($0, "title") }
entry && /<published>/ { day = substr(inner($0, "published"), 1, 10) }
entry && /<media:description>/ {
    about = $0
    sub(/^.*<media:description>/, "", about)
    if (about ~ /<\/media:description>/) sub(/<\/media:description>.*$/, "", about)
    else text = 1
    next
}
entry && /<media:description\/>/ { about = "" }
text {
    if ($0 ~ /<\/media:description>/) { line = $0; sub(/<\/media:description>.*$/, "", line); about = about "\n" line; text = 0 }
    else about = about "\n" $0
    next
}
/<\/entry>/ {
    entry = 0
    if (id ~ /^[A-Za-z0-9_-]+$/ && day ~ /^[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]$/) {
        printf "    { id: \"%s\", url: %s, title: %s, day: \"%s\", about: %s },\n", id, js(ENVIRON["WATCH"] id), js(title), day, js(about)
        count++
    }
}
END { print "];" }'
}

# The feed and the picture of every video; the list is written when the feed came.
fetch() {
    mkdir -p "$dir/videos"
    curl -fsS --max-time 30 "$feed?channel_id=$channel" -o "$dir/videos.xml.new" || return 1
    grep -q '<feed' "$dir/videos.xml.new" || { rm -f "$dir/videos.xml.new"; return 1; }
    list < "$dir/videos.xml.new" > "$file.new" && mv "$file.new" "$file"
    rm -f "$dir/videos.xml.new"
    for id in $(sed -n 's/^    { id: "\([A-Za-z0-9_-]*\)".*/\1/p' "$file"); do
        [ -s "$dir/videos/$id.jpg" ] && continue
        if curl -fsS --max-time 30 "$pictures/$id/mqdefault.jpg" -o "$dir/videos/$id.jpg.new"; then
            mv "$dir/videos/$id.jpg.new" "$dir/videos/$id.jpg"
        else
            rm -f "$dir/videos/$id.jpg.new"
        fi
    done
}

if [ "${1:-}" = fetch ]; then
    [ -n "$channel" ] || { echo "website-videos: OPENECRAFT_VIDEOS_CHANNEL is no id of a channel" >&2; exit 1; }
    fetch || { echo "website-videos: no feed from $feed" >&2; exit 1; }
    exit 0
fi

empty
if [ "$interval" -gt 0 ] && [ -n "$channel" ] && command -v curl > /dev/null; then
    (
        while :; do
            fetch || echo "website-videos: no feed from $feed, the videos stay" >&2
            sleep "$interval"
        done
    ) < /dev/null &
fi
exit 0
