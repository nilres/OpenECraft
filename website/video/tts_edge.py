"""Speaks the texts of narration.json into audio/<name>.mp3 with a neural voice of
Microsoft's online service (the voices of Edge's "Read aloud", package edge-tts; the
texts are sent to Microsoft) and writes the length of each into audio/lengths.json.

    python tts_edge.py [voice] [name]

`name` is another file of texts beside this script (`<name>.json`); its sound files go
to `audio-<name>/`.
"""
import asyncio, json, os, sys
import edge_tts

HERE = os.path.dirname(os.path.abspath(__file__))
VOICE = sys.argv[1] if len(sys.argv) > 1 else "en-US-AndrewNeural"
NAME = sys.argv[2] if len(sys.argv) > 2 else "narration"
AUDIO = "audio" if NAME == "narration" else "audio-" + NAME


async def speak(name, text):
    talk = edge_tts.Communicate(text, VOICE, boundary="WordBoundary")
    end = 0.0
    with open(os.path.join(HERE, AUDIO, name + ".mp3"), "wb") as out:
        async for chunk in talk.stream():
            if chunk["type"] == "audio":
                out.write(chunk["data"])
            elif chunk["type"] in ("WordBoundary", "SentenceBoundary"):
                # Offsets and durations are in units of 100 ns.
                end = max(end, (chunk["offset"] + chunk["duration"]) / 1e7)
    return end + 0.4


async def main():
    os.makedirs(os.path.join(HERE, AUDIO), exist_ok=True)
    texts = json.load(open(os.path.join(HERE, NAME + ".json"), encoding="utf-8"))
    lengths = {}
    for name, text in texts.items():
        # The service refuses a request now and then: asked again after a pause.
        for attempt in range(6):
            try:
                lengths[name] = round(await speak(name, text), 2)
                break
            except edge_tts.exceptions.NoAudioReceived:
                await asyncio.sleep(4 + 4 * attempt)
        else:
            raise SystemExit("no audio for " + name)
        await asyncio.sleep(1.5)
        print(name, lengths[name])
    json.dump(lengths, open(os.path.join(HERE, AUDIO, "lengths.json"), "w"), indent=1)
    print("total", round(sum(lengths.values()), 1))


asyncio.run(main())
