"""Writes short.mjs: the head of record.mjs (browser, mouse, keys, the UI scale) with
the engine of the Shorts. A Short is a scene, `scene-<name>.mjs` beside it: what it
does before the camera starts, its steps, the parts of the window it shows and its
captions; its texts are `<name>.json`, spoken by tts_edge.py into `audio-<name>/`.

    python make_short.py
    node short.mjs http://localhost:8083/ <output folder> record 2 <name>
"""
import os
HERE = os.path.dirname(os.path.abspath(__file__))
src = open(os.path.join(HERE, "record.mjs"), encoding="utf-8").read()
head = src[:src.index("// ---- the recording ---")]
tail = src[src.index("// A recording of the browser has no length in its head"):]

def change(a, b):
    global head
    assert head.count(a) == 1, a[:70]
    head = head.replace(a, b)

change('''// Records a narrated video of the web editor: how the first example project is drawn.''', '''// Records a Short (1080 x 1920) of the web editor: parts of its window cut out and put
// one above the other, with captions. Written by make_short.py: the head is the one of
// record.mjs, the scene of a Short is scene-<name>.mjs.
// Records a narrated video of the web editor: how the first example project is drawn.''')
change('const [site = "http://localhost:8083/", outDir = here, mode = "", scaleArg = "1.5"]', 'const [site = "http://localhost:8083/", outDir = here, mode = "", scaleArg = "2", NAME = "parts-list"]')
change('import { fileURLToPath } from "node:url";', 'import { fileURLToPath, pathToFileURL } from "node:url";')
change('readFileSync(join(here, "narration.json"), "utf8")', 'readFileSync(join(here, NAME + ".json"), "utf8")')
change('join(here, "audio", "lengths.json")', 'join(here, "audio-" + NAME, "lengths.json")')
change('readFileSync(join(here, "audio", name + ".mp3"))', 'readFileSync(join(here, "audio-" + NAME, name + ".mp3"))')
change('"edge-profile-record"', '"edge-profile-short"')
# At twice the size the editor takes long for a frame, and a click or key that is over
# within one is lost now and then: the clicks and keys of the head are slower here.
change("""    await sleep(120);
    await dispatch("mousePressed", { button: "left", buttons: 1, clickCount: 1 });
    await sleep(60);
    await dispatch("mouseReleased", { button: "left", buttons: 0, clickCount: 1 });
    await sleep(250);
}
async function drag(""", """    await sleep(250);
    await dispatch("mousePressed", { button: "left", buttons: 1, clickCount: 1 });
    await sleep(250);
    await dispatch("mouseReleased", { button: "left", buttons: 0, clickCount: 1 });
    await sleep(500);
}
async function drag(""")
change("""modifiers, text });
    await page.call("Input.dispatchKeyEvent", { type: "keyUp", key: name, code, windowsVirtualKeyCode: vk, nativeVirtualKeyCode: vk, modifiers });
    await sleep(200);""", """modifiers, text });
    await sleep(150);
    await page.call("Input.dispatchKeyEvent", { type: "keyUp", key: name, code, windowsVirtualKeyCode: vk, nativeVirtualKeyCode: vk, modifiers });
    await sleep(450);""")

body = r'''// ---- the scene ------------------------------------------------------------------------------
const scene = await import(pathToFileURL(join(here, "scene-" + NAME + ".mjs")).href);

// The pointer moves without waiting for the window to draw each step.
async function glide(x, y, ms = 300, held = null) {
    const from = { ...mouse };
    const steps = Math.max(2, Math.round(ms / 33));
    for (let i = 1; i <= steps; i++) {
        const t = i / steps, e = t < 0.5 ? 2 * t * t : 1 - (-2 * t + 2) ** 2 / 2;
        mouse = { x: from.x + (x - from.x) * e, y: from.y + (y - from.y) * e };
        dispatch("mouseMoved", held ? { button: held, buttons: 1 } : {}).catch(() => {});
        await sleep(33);
    }
    await dispatch("mouseMoved", held ? { button: held, buttons: 1 } : {});
}
// A click without the pauses of a tutorial; the editor still gets a frame for each
// of its steps.
async function tap(x, y, ms = 300) {
    await glide(x, y, ms);
    await sleep(150);
    await dispatch("mousePressed", { button: "left", buttons: 1, clickCount: 1 });
    await sleep(160);
    await dispatch("mouseReleased", { button: "left", buttons: 0, clickCount: 1 });
    await sleep(300);
}
async function pull(x0, y0, x1, y1, ms = 1200) {
    await glide(x0, y0, 300);
    await sleep(120);
    await dispatch("mousePressed", { button: "left", buttons: 1, clickCount: 1 });
    await sleep(120);
    await glide(x1, y1, ms, "left");
    await sleep(100);
    await dispatch("mouseReleased", { button: "left", buttons: 0, clickCount: 1 });
    await sleep(200);
}
async function press(name) {
    const vk = { Escape: 27, Enter: 13 }[name];
    const text = name === "Enter" ? "\r" : undefined;
    await page.call("Input.dispatchKeyEvent", { type: text ? "keyDown" : "rawKeyDown", key: name, code: name, windowsVirtualKeyCode: vk, nativeVirtualKeyCode: vk, text });
    await sleep(130);
    await page.call("Input.dispatchKeyEvent", { type: "keyUp", key: name, code: name, windowsVirtualKeyCode: vk, nativeVirtualKeyCode: vk });
    await sleep(280);
}
async function wheel(x, y, ticks) {
    await glide(x, y, 250);
    for (let i = 0; i < Math.abs(ticks); i++) {
        await patient(page.call("Input.dispatchMouseEvent", { type: "mouseWheel", x: x * zoom, y: y * zoom, deltaX: 0, deltaY: ticks < 0 ? -120 : 120 }));
        await sleep(140);
    }
}

// The window shows one of the views of the scene at a time; a frame is of the view
// that was in front when it came (of none while they change).
let view = scene.start;
let began = Date.now();
const frames = [];
const cues = [];
const jpegSize = jpeg => {
    for (let i = 2; i < jpeg.length;) {
        if (jpeg[i + 1] === 0xc0 || jpeg[i + 1] === 0xc2) return [jpeg.readUInt16BE(i + 7), jpeg.readUInt16BE(i + 5)];
        i += 2 + jpeg.readUInt16BE(i + 2);
    }
    return [0, 0];
};
listeners.push(message => {
    if (message.method !== "Page.screencastFrame" || message.sessionId !== page.sessionId) return;
    frames.push({ data: Buffer.from(message.params.data, "base64"), wall: (Date.now() - began) / 1000, view });
    send("Page.screencastFrameAck", { sessionId: message.params.sessionId }, page.sessionId).catch(() => {});
});
// Brings a view to the front, as the scene says how.
async function show(name) {
    view = null;
    await scene.views[name](api);
    await sleep(250);
    view = name;
    // A frame of the view as it stands.
    await glide(mouse.x + 3, mouse.y + 30, 100);
    await sleep(150);
}
// Says a text while `actions` run, and waits until both are done.
async function say(name, actions = async () => {}) {
    const at = (Date.now() - began) / 1000;
    cues.push({ name, at });
    log("say", name, at.toFixed(1));
    await actions();
    const left = at + sounds[name].seconds + 0.35 - (Date.now() - began) / 1000;
    if (left > 0) await sleep(left * 1000);
}
const api = { click, tap, glide, pull, move, drag, key, press, type, wheel, sleep, shot, show, say, log, mode };

await scene.prepare(api);
await shot("short-" + NAME + "-before");
await page.call("Page.startScreencast", { format: "jpeg", quality: 85, maxWidth: VW, maxHeight: VH, everyNthFrame: 2 });
await glide(mouse.x + 5, mouse.y + 20, 200);
await sleep(600);
{
    const [w, h] = frames.length ? jpegSize(frames[0].data) : [0, 0];
    if (w !== VW || h !== VH) { console.error(`the frames are ${w} x ${h}, not ${VW} x ${VH}: the window does not show the whole page (FRAME)`); quit(4); }
}
// The views the video begins with, the last of them in front.
for (const name of scene.before) await show(name);
await sleep(300);
const lead = frames.length;
began = Date.now();
log("recording");
await scene.play(api);
await sleep(500);
await page.call("Page.stopScreencast");
log(`${frames.length} frames, ${((Date.now() - began) / 1000).toFixed(1)} s`);
await shot("short-" + NAME + "-after");
if (mode === "check") quit(0);

// ---- the composition ------------------------------------------------------------------------
// The parts of the scene: what each shows of the window, in the pixels the steps are
// written in, and where on the 1080 x 1920 of the Short. A part appears with the
// first frame of its view.
const [SW, SH] = [1080, 1920];
const coder = await tab("about:blank");
front = coder;
await coder.call("Page.bringToFront");
await coder.js(`(() => {
    const canvas = document.createElement('canvas');
    canvas.width = ${SW}; canvas.height = ${SH};
    document.body.append(canvas);
    const ctx = canvas.getContext('2d');
    const parts = ${JSON.stringify(scene.PARTS)}, zoom = ${SCALE}, footer = ${JSON.stringify(scene.FOOTER || [1660, "openecraft.org"])};
    let caption = [], mark = null;
    const pictures = {};
    const round = (x, y, w, h, r) => { ctx.beginPath(); ctx.roundRect(x, y, w, h, r); };
    function paint() {
        const back = ctx.createLinearGradient(0, 0, 0, ${SH});
        back.addColorStop(0, '#0b1424'); back.addColorStop(1, '#070b14');
        ctx.fillStyle = back; ctx.fillRect(0, 0, ${SW}, ${SH});
        ctx.textAlign = 'center';
        ctx.fillStyle = '#3ad6ff'; ctx.font = '600 44px "Segoe UI", Arial, sans-serif';
        ctx.fillText('OpenECraft', ${SW} / 2, 116);
        ctx.fillStyle = '#fff'; ctx.font = '700 70px "Segoe UI", Arial, sans-serif';
        caption.forEach((line, i) => ctx.fillText(line, ${SW} / 2, 232 + i * 86));
        for (const [name, p] of Object.entries(parts)) {
            const picture = pictures[p.view || name];
            if (!picture) continue;
            const [x, y, w, h] = p.to;
            ctx.save();
            round(x, y, w, h, 18); ctx.clip();
            ctx.fillStyle = '#fff'; ctx.fillRect(x, y, w, h);
            ctx.drawImage(picture, p.from[0] * zoom, p.from[1] * zoom, p.from[2] * zoom, p.from[3] * zoom, x, y, w, h);
            ctx.restore();
        }
        if (mark && pictures[parts[mark.part].view || mark.part]) {
            // A row of a part, in the pixels of the window.
            const p = parts[mark.part], [x, y, w] = p.to, k = w / p.from[2];
            ctx.strokeStyle = '#f5b82e'; ctx.lineWidth = 6;
            round(x + 8, y + mark.row[0] * k, w - 16, mark.row[1] * k, 12); ctx.stroke();
        }
        ctx.fillStyle = '#9fb0c8'; ctx.font = '500 40px "Segoe UI", Arial, sans-serif';
        ctx.fillText(footer[1], ${SW} / 2, footer[0]);
    }
    const audio = new AudioContext();
    const out = audio.createMediaStreamDestination();
    const stream = new MediaStream([...canvas.captureStream(30).getVideoTracks(), ...out.stream.getAudioTracks()]);
    const type = ['video/webm;codecs=vp9,opus', 'video/webm;codecs=vp8,opus', 'video/webm'].find(t => MediaRecorder.isTypeSupported(t));
    const recorder = new MediaRecorder(stream, { mimeType: type, videoBitsPerSecond: 4000000, audioBitsPerSecond: 96000 });
    const chunks = [];
    const spoken = [];
    recorder.ondataavailable = e => e.data.size && chunks.push(e.data);
    const bytes = text => Uint8Array.from(atob(text), c => c.charCodeAt(0));
    window.tour = {
        type,
        state: () => audio.state,
        sound: async (file, at) => { spoken.push({ buffer: await audio.decodeAudioData(bytes(file).buffer), at }); },
        start: async () => {
            await audio.resume();
            const t0 = audio.currentTime + 0.05;
            for (const s of spoken) { const node = audio.createBufferSource(); node.buffer = s.buffer; node.connect(out); node.start(t0 + s.at); }
            recorder.start(1000);
        },
        draw: async (name, data) => { const old = pictures[name]; pictures[name] = await createImageBitmap(new Blob([bytes(data)], { type: 'image/jpeg' })); if (old) old.close(); paint(); },
        say: (lines, marked) => { caption = lines; mark = marked; paint(); },
        stop: () => new Promise(done => {
            recorder.onstop = () => { const reader = new FileReader(); reader.onload = () => done(reader.result.split(',')[1]); reader.readAsDataURL(new Blob(chunks, { type: 'video/webm' })); };
            recorder.stop();
        }),
    };
})()`);
log("encoding as", await coder.js("tour.type"), `${SW} x ${SH}`);
for (const cue of cues) await coder.js(`tour.sound(${JSON.stringify(sounds[cue.name].wav.toString("base64"))}, ${cue.at})`);
// What the views looked like when the video begins.
for (const name of scene.before) {
    const first = frames.slice(0, lead).filter(f => f.view === name).at(-1);
    if (!first) { console.error("no frame of the view " + name); quit(5); }
    await coder.js(`tour.draw(${JSON.stringify(name)}, ${JSON.stringify(first.data.toString("base64"))})`);
}
const marked = name => JSON.stringify((scene.MARKS || {})[name] || null);
await coder.js(`tour.say(${JSON.stringify(scene.CAPTIONS[cues[0].name])}, ${marked(cues[0].name)})`);
await coder.js("tour.start()");
log("audio", await coder.js("tour.state()"));
const started = Date.now();
let shown = 0;
const rest_frames = frames.slice(lead);
for (const [i, frame] of rest_frames.entries()) {
    const due = frame.wall * 1000 - (Date.now() - started);
    if (due > 0) await sleep(due);
    // A frame that is late is left out when a newer one of its view is due too: the
    // picture never falls behind the sound.
    const next = rest_frames[i + 1];
    if (due < -60 && next && next.view === frame.view && next.wall * 1000 <= Date.now() - started) continue;
    while (shown + 1 < cues.length && cues[shown + 1].at <= frame.wall) {
        shown++;
        await coder.js(`tour.say(${JSON.stringify(scene.CAPTIONS[cues[shown].name])}, ${marked(cues[shown].name)})`);
    }
    if (frame.view) await coder.js(`tour.draw(${JSON.stringify(frame.view)}, ${JSON.stringify(frame.data.toString("base64"))})`);
}
const end = cues.at(-1).at + sounds[cues.at(-1).name].seconds + 0.8;
const rest = end * 1000 - (Date.now() - started);
if (rest > 0) await sleep(rest);
await sleep(300);
const video = Buffer.from(await coder.js("tour.stop()"), "base64");
mkdirSync(outDir, { recursive: true });
writeFileSync(join(outDir, "short-" + NAME + ".webm"), withDuration(video, Date.now() - started));
log(`short-${NAME}.webm: ${(video.length / 1e6).toFixed(2)} MB, ${((Date.now() - started) / 1000).toFixed(0)} s`);
quit(0);

'''
open(os.path.join(HERE, "short.mjs"), "w", encoding="utf-8", newline="\n").write(head + body + tail)
print("short.mjs written")
