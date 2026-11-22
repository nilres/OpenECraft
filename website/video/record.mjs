// Records a narrated video of the web editor: how the first example project is drawn.
// A headless Edge shows the editor, this script works in it with real mouse and key
// events while the browser sends what it draws (screencast); a second page encodes the
// frames in real time with the narration (audio/<name>.mp3, spoken by tts_edge.py from
// narration.json) as the sound track (MediaRecorder). Needs only Edge and Node; on
// Windows start it from PowerShell.
//
//   node record.mjs http://localhost:8083/ <output folder> [record|check] [scale]
//
// The steps are written for the editor in a window of 1280 x 720. `scale` (1, 1.5 or
// 2; 1.5 by default) makes the window that much larger and sets the UI scale of the
// editor (View > UI scale) to it, so the layout stays and texts and symbols are
// larger and sharper: 1.5 gives a video of 1920 x 1080, 2 one of 2560 x 1440.
import { spawn } from "node:child_process";
import { writeFileSync, readFileSync, mkdirSync, rmSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const [site = "http://localhost:8083/", outDir = here, mode = "", scaleArg = "1.5"] = process.argv.slice(2);
const SCALE = Number(scaleArg);
// The entries of the UI scale menu, from its top.
const SCALES = [0.75, 0.9, 1, 1.1, 1.25, 1.5, 2];
if (![1, 1.5, 2].includes(SCALE)) { console.error("scale: 1, 1.5 or 2"); process.exit(1); }
const EDGE = "C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe";
const PORT = 9355;
const [W, H] = [1280, 720];
// What the window of a headless Edge takes for itself: the page gets the rest, and
// the browser sends only what the window shows (the size of a frame is checked below).
const FRAME = [30, 129];
// The size of the video.
const [VW, VH] = [Math.round(W * SCALE / 2) * 2, Math.round(H * SCALE / 2) * 2];
const sleep = ms => new Promise(r => setTimeout(r, ms));
const log = (...a) => console.log(new Date().toISOString().slice(11, 19), ...a);

// The narration: its texts and the length of each sound file.
const texts = JSON.parse(readFileSync(join(here, "narration.json"), "utf8"));
// Spoken by tts_edge.py (audio/<name>.mp3 with their lengths in audio/lengths.json).
const lengths = JSON.parse(readFileSync(join(here, "audio", "lengths.json"), "utf8"));
const sounds = {};
for (const name of Object.keys(texts)) sounds[name] = { wav: readFileSync(join(here, "audio", name + ".mp3")), seconds: lengths[name] };

const profile = join(here, "edge-profile-record");
rmSync(profile, { recursive: true, force: true });
mkdirSync(profile, { recursive: true });
const edge = spawn(EDGE, [
    "--headless=new", "--lang=en-US", "--accept-lang=en-US,en", `--remote-debugging-port=${PORT}`, `--user-data-dir=${profile}`, `--window-size=${VW + FRAME[0]},${VH + FRAME[1]}`,
    "--hide-scrollbars", "--force-device-scale-factor=1", "--disable-background-timer-throttling",
    "--disable-renderer-backgrounding", "--disable-backgrounding-occluded-windows", "--no-first-run",
    "--enable-unsafe-swiftshader", "--autoplay-policy=no-user-gesture-required", "about:blank",
], { stdio: "ignore" });
const quit = code => { try { edge.kill(); } catch (e) { /* gone */ } process.exit(code); };
setTimeout(() => { console.error("gave up after 15 minutes"); quit(2); }, 900000);
process.on("unhandledRejection", e => { console.error(e); quit(3); });

let version;
for (let i = 0; i < 60 && !version; i++) {
    try { version = await (await fetch(`http://127.0.0.1:${PORT}/json/version`)).json(); } catch (e) { await sleep(500); }
}
if (!version) { console.error("Edge did not start"); quit(1); }
const ws = new WebSocket(version.webSocketDebuggerUrl);
await new Promise((ok, no) => { ws.onopen = ok; ws.onerror = no; });
let next = 0;
const waiting = new Map();
const listeners = [];
ws.onmessage = event => {
    const message = JSON.parse(event.data);
    if (message.id && waiting.has(message.id)) {
        const { ok, no } = waiting.get(message.id);
        waiting.delete(message.id);
        message.error ? no(new Error(JSON.stringify(message.error))) : ok(message.result);
    } else for (const l of listeners) l(message);
};
const send = (method, params = {}, sessionId) => new Promise((ok, no) => {
    const id = ++next;
    waiting.set(id, { ok, no });
    ws.send(JSON.stringify({ id, method, params, sessionId }));
});
async function tab(url) {
    const { targetId } = await send("Target.createTarget", { url });
    const { sessionId } = await send("Target.attachToTarget", { targetId, flatten: true });
    const call = (method, params) => send(method, params, sessionId);
    await call("Page.enable");
    await call("Runtime.enable");
    const js = async (expression, awaitPromise = true) => {
        const r = await call("Runtime.evaluate", { expression, awaitPromise, returnByValue: true });
        if (r.exceptionDetails) throw new Error(r.exceptionDetails.exception?.description || r.exceptionDetails.text);
        return r.result.value;
    };
    return { targetId, sessionId, call, js };
}

// ---- the editor ---------------------------------------------------------------------------
const page = await tab("about:blank");
// A headless browser takes a page for hidden after a while and then draws it once a
// second: the page being worked with is brought to the front again and again.
let front = page;
setInterval(() => front.call("Page.bringToFront").catch(() => {}), 1000);
await page.call("Page.bringToFront");
await page.call("Emulation.setDeviceMetricsOverride", { width: VW, height: VH, deviceScaleFactor: 1, mobile: false });
await page.call("Emulation.setFocusEmulationEnabled", { enabled: true });
await page.call("Page.navigate", { url: site });
log("loading the editor");
await sleep(15000);
// A pointer the recording shows: the browser draws none without a screen.
await page.js(`(() => {
    const c = document.createElement('div');
    c.style.cssText = 'position:fixed;left:0;top:0;width:22px;height:22px;z-index:99999;pointer-events:none;transform-origin:0 0;scale:${SCALE};translate:${640 * SCALE}px ${420 * SCALE}px;' +
        'background:no-repeat url("data:image/svg+xml,' + encodeURIComponent('<svg xmlns="http://www.w3.org/2000/svg" width="22" height="22"><path d="M2 1l0 17 4.6-4.3 3 7 2.9-1.2-3-6.9 6.3-.4z" fill="#fff" stroke="#000" stroke-width="1.3"/></svg>') + '")';
    document.body.append(c);
    addEventListener('mousemove', e => c.style.translate = e.clientX + 'px ' + e.clientY + 'px', true);
})()`);

// The pointer in the pixels the steps are written in; `zoom` is how many pixels of
// the window one of them is (the UI scale, once it is set).
let mouse = { x: 640, y: 420 };
let zoom = 1;
const patient = promise => Promise.race([promise, sleep(3000)]);
const dispatch = (type, extra = {}) => patient(page.call("Input.dispatchMouseEvent", { type, x: mouse.x * zoom, y: mouse.y * zoom, button: "none", ...extra }));
async function move(x, y, ms = 550, held = null) {
    const from = { ...mouse };
    const steps = Math.max(2, Math.round(ms / 16));
    for (let i = 1; i <= steps; i++) {
        const t = i / steps, e = t < 0.5 ? 2 * t * t : 1 - (-2 * t + 2) ** 2 / 2;
        mouse = { x: from.x + (x - from.x) * e, y: from.y + (y - from.y) * e };
        await dispatch("mouseMoved", held ? { button: held, buttons: held === "left" ? 1 : 2 } : {});
        await sleep(16);
    }
}
async function click(x, y, ms = 550) {
    await move(x, y, ms);
    await sleep(120);
    await dispatch("mousePressed", { button: "left", buttons: 1, clickCount: 1 });
    await sleep(60);
    await dispatch("mouseReleased", { button: "left", buttons: 0, clickCount: 1 });
    await sleep(250);
}
async function drag(x0, y0, x1, y1, button = "left", ms = 1200) {
    await move(x0, y0, 500);
    await dispatch("mousePressed", { button, buttons: button === "left" ? 1 : 2, clickCount: 1 });
    await move(x1, y1, ms, button);
    await dispatch("mouseReleased", { button, buttons: 0, clickCount: 1 });
    await sleep(250);
}
async function key(name, modifiers = 0) {
    const codes = { Escape: 27, Enter: 13 };
    const vk = codes[name] ?? name.toUpperCase().charCodeAt(0);
    const code = name.length === 1 ? "Key" + name.toUpperCase() : name;
    const text = name === "Enter" ? "\r" : undefined;
    await page.call("Input.dispatchKeyEvent", { type: text ? "keyDown" : "rawKeyDown", key: name, code, windowsVirtualKeyCode: vk, nativeVirtualKeyCode: vk, modifiers, text });
    await page.call("Input.dispatchKeyEvent", { type: "keyUp", key: name, code, windowsVirtualKeyCode: vk, nativeVirtualKeyCode: vk, modifiers });
    await sleep(200);
}
async function type(text, each = 75) {
    for (const c of text) { await page.call("Input.insertText", { text: c }); await sleep(each); }
    await sleep(300);
}
async function shot(name) {
    const { data } = await page.call("Page.captureScreenshot", { format: "png" });
    writeFileSync(join(outDir, name + ".png"), Buffer.from(data, "base64"));
}

// Before the recording: the UI scale of the editor (in a window wider than 1280 the
// ribbon shows its View tab whole, with the menu at this place), ...
if (SCALE !== 1) {
    await click(211, 37, 200);
    await sleep(500);
    await click(1318, 87, 200);
    await sleep(500);
    await click(1298, 115 + 24 * SCALES.indexOf(SCALE), 200);
    await sleep(1200);
    zoom = SCALE;
    mouse = { x: mouse.x / zoom, y: mouse.y / zoom };
    await click(69, 37, 200);
    await sleep(400);
}
// ... and the Messages pane makes room for the page.
await click(998, 597, 200);
await click(1032, 648, 200);
await move(640, 420, 200);
await sleep(600);

// ---- the recording --------------------------------------------------------------------------
const frames = [];
listeners.push(message => {
    if (message.method !== "Page.screencastFrame" || message.sessionId !== page.sessionId) return;
    frames.push({ data: Buffer.from(message.params.data, "base64"), t: message.params.metadata.timestamp, wall: (Date.now() - began) / 1000 });
    send("Page.screencastFrameAck", { sessionId: message.params.sessionId }, page.sessionId).catch(() => {});
});
const jpegSize = jpeg => {
    for (let i = 2; i < jpeg.length;) {
        if (jpeg[i + 1] === 0xc0 || jpeg[i + 1] === 0xc2) return [jpeg.readUInt16BE(i + 7), jpeg.readUInt16BE(i + 5)];
        i += 2 + jpeg.readUInt16BE(i + 2);
    }
    return [0, 0];
};
const began = Date.now();
await page.call("Page.startScreencast", { format: "jpeg", quality: 85, maxWidth: VW, maxHeight: VH, everyNthFrame: 2 });
log("recording");
await move(650, 425, 300);
await sleep(500);
{
    const [w, h] = frames.length ? jpegSize(frames[0].data) : [0, 0];
    if (w !== VW || h !== VH) { console.error(`the frames are ${w} x ${h}, not ${VW} x ${VH}: the window does not show the whole page (FRAME)`); quit(4); }
}
// What is said when, in seconds of the recording.
const cues = [];
// Says a text while `actions` run, and waits until both are done.
async function say(name, actions = async () => {}) {
    const at = (Date.now() - began) / 1000;
    cues.push({ name, at });
    log("say", name, at.toFixed(1));
    await actions();
    const left = at + sounds[name].seconds + 0.5 - (Date.now() - began) / 1000;
    if (left > 0) await sleep(left * 1000);
}

// The pixel of a point of the page in mm, after the view is set below.
const P = (x, y) => [225.2 + x * 3.065, 196.85 + y * 3.065];
const [TAB_Y, SEARCH, PAGE_TAB] = [37, [1125, 185], [227, 159]];
async function search(text) {
    await click(...SEARCH);
    await key("a", 2);
    await type(text);
    await sleep(500);
}
// A symbol or device of the Insert center placed on the page.
async function place(text, entry, x, y) {
    await search(text);
    await click(...entry);
    await click(...P(x, y), 800);
    await sleep(400);
    await key("Escape");
}
async function wire(points) {
    for (const p of points) await click(...P(...p), 600);
    await key("Enter");
    await sleep(300);
}
const DEVICE = [1115, 324];

await sleep(700);
await say("intro", async () => {
    await move(520, 300, 1200);
    await sleep(2500);
    await move(330, 542, 1200);
    await sleep(3000);
    await move(269, 307, 1200);
});
await say("new", async () => {
    await click(269, 307);
    await sleep(1200);
    await click(...PAGE_TAB);
});
await say("zoom", async () => {
    await click(211, TAB_Y);
    await sleep(500);
    await click(34, 75);
    await sleep(900);
    for (let i = 0; i < 3; i++) { await click(110, 87, 300); await sleep(350); }
    await sleep(400);
    await drag(500, 350, 764, 560, "right");
});
if (mode === "check") { await move(1000, 690, 300); await sleep(400); await shot("check-view"); }
if (mode === "check") { writeFileSync(join(outDir, "check-frame.jpg"), frames.at(-1).data); }
await say("pot24", async () => {
    await sleep(2500);
    await search("24");
    await sleep(600);
    await click(1145, 263);
    await sleep(600);
    await click(...P(24, 32), 900);
    await sleep(900);
    await key("Escape");
});
await say("pot0", async () => {
    await place("0V", [1165, 263], 24, 148);
});
await say("breaker", async () => {
    await sleep(6500);
    await place("MCB-1P-C6", DEVICE, 60, 56);
});
await say("s1", async () => {
    await place("PB-GN-1NO", DEVICE, 60, 92);
    await place("LMP-GN-24", DEVICE, 60, 124);
});
await say("s2", async () => {
    await place("SEL-2POS-1NO", DEVICE, 120, 92);
    await place("LMP-RD-24", DEVICE, 120, 124);
});
if (mode === "check") await shot("check-devices");
await say("wire1", async () => {
    await sleep(1500);
    await click(121, TAB_Y);
    await sleep(700);
    await click(283, 75);
    await sleep(4500);
    await wire([[24, 36], [60, 36], [60, 52]]);
});
await say("wire2", async () => {
    await wire([[60, 60], [60, 88]]);
    await wire([[60, 72], [120, 72], [120, 88]]);
});
await say("wire3", async () => {
    await wire([[24, 152], [120, 152], [120, 128]]);
    await wire([[60, 128], [60, 152]]);
    await key("Escape");
    await move(...P(150, 100), 700);
});
if (mode === "check") await shot("check-wires");
await say("text", async () => {
    await click(...P(60, 92), 700);
    await sleep(900);
    await click(1200, 581);
    await type("Test lamp");
    await key("Enter");
    await sleep(600);
    await click(...P(120, 92), 700);
    await sleep(700);
    await click(1200, 581);
    await type("Light on / off");
    await key("Enter");
    await sleep(400);
    await click(...P(170, 110), 600);
});
await say("check", async () => {
    await move(950, 700, 1200);
});
if (mode === "check") await shot("check-status");
await say("bom", async () => {
    await sleep(1800);
    await click(430, TAB_Y);
    await sleep(700);
    await click(235, 72);
    await sleep(1800);
    await click(374, 306);
    await sleep(2500);
    await click(928, 237);
    await sleep(1500);
    await click(1246, 157);
    await sleep(800);
    await move(640, 330, 900);
});
if (mode === "check") await shot("check-bom");
await say("outro", async () => {
    await click(75, 244);
    await sleep(800);
    await click(211, TAB_Y);
    await sleep(500);
    await click(34, 75);
    await sleep(600);
    await move(700, 420, 900);
});
await sleep(800);
await page.call("Page.stopScreencast");
if (mode === "check") await shot("check-end");
const seconds = frames.at(-1).wall;
log(`${frames.length} frames, ${seconds.toFixed(1)} s`);
writeFileSync(join(outDir, "cues.json"), JSON.stringify(cues, null, 1));
if (mode === "check") quit(0);

// ---- the encoding ----------------------------------------------------------------------------
const coder = await tab("about:blank");
front = coder;
await coder.call("Page.bringToFront");
await coder.js(`(() => {
    const canvas = document.createElement('canvas');
    canvas.width = ${VW}; canvas.height = ${VH};
    document.body.append(canvas);
    const ctx = canvas.getContext('2d');
    ctx.fillStyle = '#f5f5f5'; ctx.fillRect(0, 0, ${VW}, ${VH});
    const audio = new AudioContext();
    const out = audio.createMediaStreamDestination();
    const stream = new MediaStream([...canvas.captureStream(30).getVideoTracks(), ...out.stream.getAudioTracks()]);
    const type = ['video/webm;codecs=vp9,opus', 'video/webm;codecs=vp8,opus', 'video/webm'].find(t => MediaRecorder.isTypeSupported(t));
    const recorder = new MediaRecorder(stream, { mimeType: type, videoBitsPerSecond: Math.round(2500000 * ${SCALE} * ${SCALE} * 0.8), audioBitsPerSecond: 96000 });
    const chunks = [];
    const spoken = [];
    recorder.ondataavailable = e => e.data.size && chunks.push(e.data);
    const bytes = text => Uint8Array.from(atob(text), c => c.charCodeAt(0));
    window.tour = {
        type,
        state: () => audio.state,
        sound: async (wav, at) => { spoken.push({ buffer: await audio.decodeAudioData(bytes(wav).buffer), at }); },
        start: async () => {
            await audio.resume();
            const t0 = audio.currentTime + 0.05;
            for (const s of spoken) { const node = audio.createBufferSource(); node.buffer = s.buffer; node.connect(out); node.start(t0 + s.at); }
            recorder.start(1000);
        },
        draw: async data => { const image = await createImageBitmap(new Blob([bytes(data)], { type: 'image/jpeg' })); ctx.drawImage(image, 0, 0, ${VW}, ${VH}); image.close(); },
        stop: () => new Promise(done => {
            recorder.onstop = () => { const reader = new FileReader(); reader.onload = () => done(reader.result.split(',')[1]); reader.readAsDataURL(new Blob(chunks, { type: 'video/webm' })); };
            recorder.stop();
        }),
    };
})()`);
log("encoding as", await coder.js("tour.type"), `${VW} x ${VH}`);
for (const cue of cues) await coder.js(`tour.sound(${JSON.stringify(sounds[cue.name].wav.toString("base64"))}, ${cue.at})`);
await coder.js(`tour.draw(${JSON.stringify(frames[0].data.toString("base64"))})`);
await coder.js("tour.start()");
log("audio", await coder.js("tour.state()"));
const started = Date.now();
// The cues and the frames count from the start of the recording.
for (const frame of frames) {
    const due = frame.wall * 1000 - (Date.now() - started);
    if (due > 0) await sleep(due);
    await coder.js(`tour.draw(${JSON.stringify(frame.data.toString("base64"))})`);
}
const end = cues.at(-1).at + sounds[cues.at(-1).name].seconds + 1;
const rest = end * 1000 - (Date.now() - started);
if (rest > 0) await sleep(rest);
await sleep(400);
const video = Buffer.from(await coder.js("tour.stop()"), "base64");
mkdirSync(outDir, { recursive: true });
writeFileSync(join(outDir, "first-circuit.webm"), withDuration(video, Date.now() - started));
writeFileSync(join(outDir, "first-circuit.jpg"), frames[Math.round(frames.length * 0.72)].data);
log(`first-circuit.webm: ${(video.length / 1e6).toFixed(2)} MB, ${((Date.now() - started) / 1000).toFixed(0)} s`);
quit(0);

// A recording of the browser has no length in its head, so a player shows none and
// cannot jump: writes it into the Info element of the WebM (milliseconds).
function withDuration(file, ms) {
    const vint = at => {
        let length = 1;
        while (length <= 8 && !(file[at] & (0x80 >> (length - 1)))) length++;
        let value = BigInt(file[at] & (0xff >> length));
        for (let i = 1; i < length; i++) value = (value << 8n) | BigInt(file[at + i]);
        return { length, value };
    };
    const info = file.indexOf(Buffer.from([0x15, 0x49, 0xa9, 0x66]));
    if (info < 0 || info > 4096) return file;
    const size = vint(info + 4);
    const body = file.subarray(info + 4 + size.length, info + 4 + size.length + Number(size.value));
    if (body.indexOf(Buffer.from([0x44, 0x89])) >= 0) return file;
    const duration = Buffer.alloc(11);
    duration.set([0x44, 0x89, 0x88]);
    duration.writeDoubleBE(ms, 3);
    const total = Number(size.value) + 11;
    const header = Buffer.alloc(8);
    header.writeBigUInt64BE((1n << 56n) | BigInt(total));
    return Buffer.concat([file.subarray(0, info + 4), header, body, duration, file.subarray(info + 4 + size.length + Number(size.value))]);
}
