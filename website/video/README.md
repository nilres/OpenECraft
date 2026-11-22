# Videos of OpenECraft

How the videos of the YouTube channel [@OpenECraft](https://www.youtube.com/@OpenECraft)
are made: a headless Edge shows the web editor, a script works in it with real mouse
and key events, the browser sends what it draws, and a second page encodes the frames
with the narration as the sound track. Needs Edge, Node and Python; no ffmpeg.

## Files

| File | What it is |
| --- | --- |
| `record.mjs` | Recorder of a tutorial (16:9): the steps of "first circuit", narrated |
| `narration.json` | The texts of the tutorial, one per step |
| `make_short.py` | Writes `short.mjs`: the head of `record.mjs` with the engine of the Shorts |
| `short.mjs` | Recorder of a Short (1080 × 1920); generated, do not edit |
| `scene-<name>.mjs` | A Short: what happens before the camera starts (`prepare`), its steps (`play`), the views of the window (`views`), what is cut out and where it goes (`PARTS`), captions (`CAPTIONS`), a marked row (`MARKS`) |
| `<name>.json` | The texts of a Short |
| `tts_edge.py` | Speaks a text file into `audio/` or `audio-<name>/` (MP3 and `lengths.json`) |
| `explore.mjs` | A headless Edge with the editor to drive by hand, for finding the positions the steps click at |

The videos, the sound files, `short.mjs`, the screenshots and the profile folders of
the browser are not kept in the repository (`.gitignore`): every one of them is made
again by the steps below. All commands are run in `website/video/`.

## A new video, step by step

1. Serve the web editor: `trunk serve --config openecraft/Trunk.toml --release --port 8083 --dist target/dist-release`
   (the launch configuration `web-release`). A change of the branch rebuilds it, which
   takes a quarter of an hour.
2. Write the texts (`<name>.json`, one entry per step; "Open E Craft" is spoken right).
3. Speak them: `python tts_edge.py en-US-AndrewNeural <name>`. It needs the package
   `edge-tts` (`pip install edge-tts` in an environment of Python 3.12) and sends the
   texts to Microsoft's speech service.
4. Write the steps: for a tutorial in `record.mjs` (`say("name", async () => { … })`),
   for a Short a `scene-<name>.mjs`, then `python make_short.py`.
5. Record, **from PowerShell** (headless Edge does not start from Git Bash):
   - tutorial: `node record.mjs http://localhost:8083/ <folder> record 1.5` (1920 × 1080)
   - Short: `node --max-old-space-size=6000 short.mjs http://localhost:8083/ <folder> record 2 <name>`
   - `check` instead of `record` runs the steps without encoding and writes screenshots.
6. Look at frames of the result at every step before it goes anywhere: a take can be
   wrong although the script ran through.
7. Upload in YouTube Studio (channel OpenECraft, not the personal one of the account).

## What the steps are written in

All positions are pixels of the editor in a window of 1280 × 720, whatever the size of
the video: the recorder makes the window larger and sets *View › UI scale* to match
(1.5 for 1920 × 1080, 2 for the Shorts), so the layout stays. A point of a schematic
page in mm is `P(x, y)` after the view was set by *Fit page*, three times *Zoom in*
and the right drag of the script.

To find positions, drive the editor by hand first: `explore.mjs` keeps a headless
Edge open and takes commands over HTTP (click, type, screenshot; its head says how).
The status bar of a screenshot shows the coordinates of the pointer on the page.

## What went wrong before, so it does not again

- **One frame a second.** A headless browser takes a page for hidden after a while and
  draws it once a second. The recorders bring the page to the front every second.
- **Frames cut off.** The window of a headless Edge keeps 30 × 129 pixels for itself
  (`FRAME`). The recorders stop when a frame is not the size of the video.
- **Larger text.** A device scale factor does not work with the editor's canvas; the
  UI scale of the editor does.
- **Lost clicks at twice the size.** The editor takes long for a frame at 2560 × 1440,
  and a click or key that is over within one is lost now and then. The Shorts use
  slower clicks; steps before the camera starts are the least reliable, so a scene
  should use the demo or an example project as it opens instead of drawing one.
- **A page is drawn whole only from its second visit**, and its first picture after a
  tab change is stale until the pointer moves.
- **Picture behind the sound.** The encoder leaves out frames that are late.
- **The parts list beside the drawing** does not exist in the editor: *Dock* puts the
  reports into a tab over the page. A Short shows both by cutting each out while its
  tab is in front.
- **Seeking** in the recorded file needs a player that reads it whole; YouTube
  encodes it again.

## YouTube

- Channel: `https://www.youtube.com/@OpenECraft`, id `UCyqRmnEuKZZSQcZ8HSqmewA`;
  playlist of the walkthroughs: "OpenECraft example projects".
- A vertical video of up to three minutes becomes a Short by itself; `#shorts` in the
  title helps.
- Every video: "not made for kids", links to the web editor, the website and the
  source code, and the line that the narration is a synthetic voice. No mention of
  other products in titles and descriptions.
- Custom thumbnails and clickable links need the one-time verification of the channel.
- The website lists the videos by itself (`installer/docker/website-videos.sh`).
