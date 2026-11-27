"""Posts news of OpenECraft to the announcements channel of its Discord server
(.github/workflows/discord.yml).

    discord.py release              the release of the event (GITHUB_EVENT_PATH)
    discord.py youtube <seen file>  the videos of the channel not in <seen file> yet

The address of the channel's webhook is DISCORD_WEBHOOK_URL; without it nothing is
posted and the posts are printed (a fork, a dry run). A seen file that does not exist
yet is written with the videos of the feed and nothing is posted, so that a lost cache
does not post the whole channel again.
"""

import json
import os
import sys
import urllib.request
import xml.etree.ElementTree as ET

CHANNEL = os.environ.get("YOUTUBE_CHANNEL", "UCyqRmnEuKZZSQcZ8HSqmewA")
FEED = "https://www.youtube.com/feeds/videos.xml?channel_id=" + CHANNEL
NS = {"atom": "http://www.w3.org/2005/Atom", "yt": "http://www.youtube.com/xml/schemas/2015"}
# Discord takes at most 4096 characters in the description of an embed.
LIMIT = 3800
# At most this many videos in one run, the oldest first.
MOST = 5


def post(message):
    url = os.environ.get("DISCORD_WEBHOOK_URL", "").strip()
    message["allowed_mentions"] = {"parse": []}
    if not url:
        print("DISCORD_WEBHOOK_URL is not set, not posted:")
        print(json.dumps(message, indent=2, ensure_ascii=False))
        return
    request = urllib.request.Request(
        url,
        data=json.dumps(message).encode("utf-8"),
        headers={"Content-Type": "application/json", "User-Agent": "OpenECraft-Announcements (github.com/nilres/OpenECraft)"},
    )
    with urllib.request.urlopen(request, timeout=30) as answer:
        print("posted:", answer.status)


def shorten(text):
    text = text.replace("\r\n", "\n").strip()
    if len(text) <= LIMIT:
        return text
    cut = text.rfind("\n", 0, LIMIT)
    return text[: cut if cut > 0 else LIMIT].rstrip() + "\n\n…"


def release():
    with open(os.environ["GITHUB_EVENT_PATH"], encoding="utf-8") as file:
        event = json.load(file)["release"]
    tag = event["tag_name"]
    name = event.get("name") or tag
    notes = shorten(event.get("body") or "")
    post({
        "content": f"**OpenECraft {tag}** is out.",
        "embeds": [{
            "title": name,
            "url": event["html_url"],
            "description": notes + f"\n\n[Downloads and full notes]({event['html_url']})",
            "color": 0x2B6CC4,
        }],
    })


def videos():
    request = urllib.request.Request(FEED, headers={"User-Agent": "OpenECraft-Announcements"})
    with urllib.request.urlopen(request, timeout=30) as answer:
        root = ET.fromstring(answer.read())
    found = []
    for entry in root.findall("atom:entry", NS):
        found.append({
            "id": entry.findtext("yt:videoId", namespaces=NS),
            "title": entry.findtext("atom:title", namespaces=NS),
            "published": entry.findtext("atom:published", namespaces=NS),
        })
    return [v for v in found if v["id"]]


def changed():
    # The workflow keeps the seen file only when it changed.
    if "GITHUB_OUTPUT" in os.environ:
        with open(os.environ["GITHUB_OUTPUT"], "a", encoding="utf-8") as file:
            file.write("changed=true\n")


def youtube(seen_file):
    feed = videos()
    if not os.path.exists(seen_file):
        with open(seen_file, "w", encoding="utf-8") as file:
            file.writelines(v["id"] + "\n" for v in feed)
        print(f"{seen_file} written with {len(feed)} videos, nothing posted")
        changed()
        return
    with open(seen_file, encoding="utf-8") as file:
        seen = {line.strip() for line in file if line.strip()}
    new = sorted((v for v in feed if v["id"] not in seen), key=lambda v: v["published"] or "")
    for video in new[-MOST:]:
        post({"content": f"New video: **{video['title']}**\nhttps://www.youtube.com/watch?v={video['id']}"})
    print(f"{len(new)} new videos")
    if not os.environ.get("DISCORD_WEBHOOK_URL", "").strip():
        return
    # Older ones beyond MOST are marked seen without a post.
    with open(seen_file, "a", encoding="utf-8") as file:
        file.writelines(v["id"] + "\n" for v in new)
    if new:
        changed()


if __name__ == "__main__":
    if sys.argv[1:2] == ["release"]:
        release()
    elif sys.argv[1:2] == ["youtube"] and len(sys.argv) == 3:
        youtube(sys.argv[2])
    else:
        sys.exit(__doc__)
