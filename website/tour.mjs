// Records the screen recording of the first screen (tour.webm with its still picture
// tour.jpg, docs/website.md): a headless Edge shows the built site, this script works
// in its live demos with real mouse events while the browser sends what it draws
// (screencast), and a second page encodes the frames in real time (MediaRecorder).
// Needs only Edge and Node; on Windows start it from PowerShell.
//
//   cd website; trunk serve --release          (the site at http://localhost:8084/)
//   node website/tour.mjs http://localhost:8084/ website [folder for the browser profile]
import { spawn } from "node:child_process";
import { writeFileSync, mkdirSync, rmSync } from "node:fs";
import { join } from "node:path";

const [site = "http://localhost:8084/", outDir = ".", scratch = "."] = process.argv.slice(2);
const EDGE = "C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe";
const PORT = 9333;
const [W, H] = [1280, 720];
const sleep = ms => new Promise(r => setTimeout(r, ms));
const log = (...a) => console.log(new Date().toISOString().slice(11, 19), ...a);

const profile = join(scratch, "edge-profile");
rmSync(profile, { recursive: true, force: true });
mkdirSync(profile, { recursive: true });
const edge = spawn(EDGE, [
    "--headless=new", "--lang=en-US", "--accept-lang=en-US,en", `--remote-debugging-port=${PORT}`, `--user-data-dir=${profile}`, `--window-size=${W + 30},${H + 95}`,
    "--hide-scrollbars", "--force-device-scale-factor=1", "--disable-background-timer-throttling",
    "--disable-renderer-backgrounding", "--disable-backgrounding-occluded-windows", "--no-first-run",
    "--enable-unsafe-swiftshader", "--autoplay-policy=no-user-gesture-required", "about:blank",
], { stdio: "ignore" });
const quit = code => { try { edge.kill(); } catch (e) { /* gone */ } process.exit(code); };
setTimeout(() => { console.error("gave up after 6 minutes"); quit(2); }, 360000);
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

// ---- the page of the demos ---------------------------------------------------------------
const page = await tab("about:blank");
await page.call("Page.bringToFront");
await page.call("Emulation.setFocusEmulationEnabled", { enabled: true });
await page.call("Page.navigate", { url: site + "?example=conveyor" });
log("loading the site");
const until = async (expression, seconds = 60) => {
    for (let i = 0; i < seconds * 4; i++) {
        if (await page.js(expression).catch(() => false)) return;
        await sleep(250);
    }
    log("waited in vain for " + expression);
};
await until(`document.querySelectorAll('#examples button').length > 3 && document.querySelector('#stat-devices').textContent !== '–'`);
// A pointer the recording shows: the browser draws none without a screen.
await page.js(`(() => {
    const c = document.createElement('div');
    c.id = 'tour-pointer';
    c.style.cssText = 'position:fixed;left:0;top:0;width:22px;height:22px;z-index:99999;pointer-events:none;translate:640px 300px;' +
        'background:no-repeat url("data:image/svg+xml,' + encodeURIComponent('<svg xmlns="http://www.w3.org/2000/svg" width="22" height="22"><path d="M2 1l0 17 4.6-4.3 3 7 2.9-1.2-3-6.9 6.3-.4z" fill="#fff" stroke="#000" stroke-width="1.3"/></svg>') + '")';
    document.body.append(c);
    addEventListener('mousemove', e => c.style.translate = e.clientX + 'px ' + e.clientY + 'px', true);
    for (const band of document.querySelectorAll('.band')) band.classList.add('seen');
})()`);

const frames = [];
let mouse = { x: 640, y: 300 };
const patient = (promise, what) => Promise.race([promise, sleep(3000)]);
const dispatch = (type, extra = {}) => patient(page.call("Input.dispatchMouseEvent", { type, x: mouse.x, y: mouse.y, button: "none", ...extra }), type);
async function move(x, y, ms = 600) {
    const from = { ...mouse };
    const steps = Math.max(2, Math.round(ms / 16));
    for (let i = 1; i <= steps; i++) {
        const t = i / steps, e = t < 0.5 ? 2 * t * t : 1 - (-2 * t + 2) ** 2 / 2;
        mouse = { x: from.x + (x - from.x) * e, y: from.y + (y - from.y) * e };
        await dispatch("mouseMoved", extra.held ? { button: "left", buttons: 1 } : {});
        await sleep(16);
    }
}
const extra = { held: false };
async function click(x, y, ms = 600) {
    await move(x, y, ms);
    await dispatch("mousePressed", { button: "left", buttons: 1, clickCount: 1 });
    await sleep(60);
    await dispatch("mouseReleased", { button: "left", buttons: 0, clickCount: 1 });
}
async function drag(x0, y0, x1, y1, ms = 1500) {
    await move(x0, y0, 500);
    await dispatch("mousePressed", { button: "left", buttons: 1, clickCount: 1 });
    extra.held = true;
    await move(x1, y1, ms);
    extra.held = false;
    await dispatch("mouseReleased", { button: "left", buttons: 0, clickCount: 1 });
}
// The middle of an element (or a point inside it, as fractions of its size).
const at = (selector, fx = 0.5, fy = 0.5) => page.js(`(() => { const e = ${selector}; if (!e) return null; const r = e.getBoundingClientRect(); return [r.left + r.width * ${fx}, r.top + r.height * ${fy}]; })()`);
async function clickOn(selector, ms = 600, fx = 0.5, fy = 0.5) {
    log("click", selector.slice(0, 70), frames.length);
    const p = await at(selector, fx, fy);
    if (!p) throw new Error("nothing at " + selector);
    await click(p[0], p[1], ms);
}
const q = s => `document.querySelector(${JSON.stringify(s)})`;
// Scrolls a section below the bar, smoothly.
async function show(selector, offset = 70) {
    log("show", selector);
    await page.js(`(() => { const e = ${q(selector)}; scrollTo({ top: e.getBoundingClientRect().top + scrollY - ${offset}, behavior: 'smooth' }); })()`);
    await sleep(1100);
}
const settled = () => log("settling") || until(`!document.querySelector('.stage.loading') && document.querySelector('#issue-count').textContent !== '…'`);

// ---- the recording ---------------------------------------------------------------------------
listeners.push(message => {
    if (message.method !== "Page.screencastFrame" || message.sessionId !== page.sessionId) return;
    frames.push({ data: message.params.data, t: message.params.metadata.timestamp });
    send("Page.screencastFrameAck", { sessionId: message.params.sessionId }, page.sessionId).catch(() => {});
});
await page.js(`scrollTo(0, 0)`);
await sleep(800);
await page.call("Page.startScreencast", { format: "jpeg", quality: 88, maxWidth: W, maxHeight: H, everyNthFrame: 1 });
log("recording");
const marks = {};
const mark = name => marks[name] = frames.length;

// 1. The first screen: the sheet the editor draws.
await move(900, 330, 900);
await sleep(1600);

// 2. The schematic editor: pages, a symbol placed, undone.
await show("#studio .studio", 110);
await clickOn(`${q('.stage[data-view="schematic"] .veil')}`, 700);
await sleep(900);
mark("schematic");
for (const index of [2, 3]) {
    await clickOn(`document.querySelectorAll('#pages li')[${index}]`, 600);
    await sleep(1500);
}
await clickOn(q('[data-tool="lamp"]'), 600);
await sleep(300);
await clickOn(q('#canvas-schematic'), 800, 0.72, 0.62);
await sleep(1500);
await clickOn(q('[data-tool="select"]'), 500);
await sleep(500);
await clickOn(q('#undo'), 500);
await sleep(1200);

// 3. Another example.
await page.js(`${q('#examples')}.scrollIntoView({ block: 'center', behavior: 'smooth' })`);
await sleep(800);
await clickOn(q('[data-example="star_delta_starter"]'), 600);
await sleep(600);
await settled();
await show("#studio .studio", 110);
await sleep(1800);
await page.js(`${q('#examples')}.scrollIntoView({ block: 'center', behavior: 'smooth' })`);
await sleep(700);
await clickOn(q('[data-example="conveyor"]'), 600);
await sleep(600);
await settled();

// 4. The panel in 3D: turned, zoomed, the mounting plate, back.
await show("#layout .stage-wrap", 90);
await sleep(600);
await settled();
await clickOn(q('.stage[data-view="layout"] .veil'), 700);
await sleep(1200);
mark("panel");
const [cx, cy] = await at(q('#canvas-layout'));
await drag(cx - 160, cy - 20, cx + 180, cy + 40, 2200);
await sleep(500);
await drag(cx + 120, cy + 60, cx - 60, cy - 30, 1600);
await sleep(400);
for (let i = 0; i < 5; i++) {
    await patient(page.call("Input.dispatchMouseEvent", { type: "mouseWheel", x: mouse.x, y: mouse.y, deltaX: 0, deltaY: -120 }), "wheel");
    await sleep(160);
}
await sleep(1300);
await clickOn(q('[data-view-mode="front"]'), 600);
await sleep(1700);
await clickOn(q('[data-view-mode="3d"]'), 500);
await sleep(1500);

// 5. The reports.
await show("#reports .reports", 100);
await sleep(700);
for (const name of ["Terminal diagram", "Cable diagram", "Bill of materials"]) {
    await clickOn(`[...document.querySelectorAll('#report-list li')].find(li => li.textContent === ${JSON.stringify(name)})`, 600).catch(() => {});
    await sleep(1200);
}

// 6. Another language, and what follows it.
await show("#languages", 200);
await clickOn(q('[data-language="de"]'), 700);
await sleep(700);
await settled();
await show("#reports .reports", 100);
await sleep(1500);
await show("#studio .studio", 110);
await sleep(600);
await settled();
await sleep(1500);
await page.js(`${q('[data-language="en"]')}.click()`);
await sleep(500);
await settled();
await page.js(`scrollTo({ top: 0, behavior: 'smooth' })`);
await sleep(1800);

await page.call("Page.stopScreencast");
const seconds = frames.at(-1).t - frames[0].t;
log(`${frames.length} frames, ${seconds.toFixed(1)} s`, JSON.stringify(marks));

// ---- the encoding ----------------------------------------------------------------------------
const size = (jpeg => {
    for (let i = 2; i < jpeg.length;) {
        if (jpeg[i + 1] === 0xc0 || jpeg[i + 1] === 0xc2) return [jpeg.readUInt16BE(i + 7) & ~1, jpeg.readUInt16BE(i + 5) & ~1];
        i += 2 + jpeg.readUInt16BE(i + 2);
    }
    return [W, H];
})(Buffer.from(frames[0].data, "base64"));
log("frames of", size.join(" × "));
const [FW, FH] = size;
const coder = await tab("about:blank");
await coder.call("Page.bringToFront");
await coder.js(`(() => {
    const canvas = document.createElement('canvas');
    canvas.width = ${FW}; canvas.height = ${FH};
    document.body.append(canvas);
    const ctx = canvas.getContext('2d');
    ctx.fillStyle = '#070b14'; ctx.fillRect(0, 0, ${FW}, ${FH});
    const type = ['video/webm;codecs=vp9', 'video/webm;codecs=vp8', 'video/webm'].find(t => MediaRecorder.isTypeSupported(t));
    const recorder = new MediaRecorder(canvas.captureStream(30), { mimeType: type, videoBitsPerSecond: 1400000 });
    const chunks = [];
    recorder.ondataavailable = e => e.data.size && chunks.push(e.data);
    window.tour = {
        type,
        start: () => recorder.start(1000),
        draw: async data => { const image = await createImageBitmap(await (await fetch('data:image/jpeg;base64,' + data)).blob()); ctx.drawImage(image, 0, 0, ${FW}, ${FH}); image.close(); },
        stop: () => new Promise(done => {
            recorder.onstop = () => { const reader = new FileReader(); reader.onload = () => done(reader.result.split(',')[1]); reader.readAsDataURL(new Blob(chunks, { type: 'video/webm' })); };
            recorder.stop();
        }),
    };
})()`);
log("encoding as", await coder.js("tour.type"));
await coder.js(`tour.draw(${JSON.stringify(frames[0].data)})`);
await coder.js("tour.start()", false);
const began = Date.now();
for (const frame of frames) {
    const due = (frame.t - frames[0].t) * 1000 - (Date.now() - began);
    if (due > 0) await sleep(due);
    await coder.js(`tour.draw(${JSON.stringify(frame.data)})`);
}
await sleep(400);
const video = Buffer.from(await coder.js("tour.stop()"), "base64");

mkdirSync(outDir, { recursive: true });
writeFileSync(join(outDir, "tour.webm"), withDuration(video, (Date.now() - began)));
const still = frames[Math.min(frames.length - 1, (marks.panel || 0) + Math.round((frames.length - (marks.panel || 0)) * 0.12))];
writeFileSync(join(outDir, "tour.jpg"), Buffer.from(still.data, "base64"));
log(`tour.webm: ${(video.length / 1e6).toFixed(2)} MB`);
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
