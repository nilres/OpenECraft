// Pictures of a recorded video at given seconds, for looking at a take (README.md): a
// headless Edge plays the file from a small local server and draws the frames into a
// canvas. On Windows start it from PowerShell.
//
//   node frames.mjs <video.webm> <folder> <seconds> [<seconds> ...]
//
// Writes <folder>/<name of the video>-<seconds>.jpg; a video wider than 1280 pixels is
// drawn that wide. Without seconds, a picture every tenth of the length.
import { spawn } from "node:child_process";
import { writeFileSync, mkdirSync, rmSync, statSync, createReadStream } from "node:fs";
import { join, basename, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { createServer } from "node:http";

const here = dirname(fileURLToPath(import.meta.url));
const [file, dir = ".", ...at] = process.argv.slice(2);
if (!file) { console.error("node frames.mjs <video.webm> <folder> <seconds> ..."); process.exit(1); }
const EDGE = "C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe";
const [PORT, HTTP] = [9366, 9367];
const sleep = ms => new Promise(r => setTimeout(r, ms));

// The file, with the ranges a player asks for when it jumps.
const size = statSync(file).size;
const server = createServer((req, res) => {
    if (req.url !== "/video") { res.setHeader("content-type", "text/html"); res.end("<body></body>"); return; }
    const range = /bytes=(\d+)-(\d*)/.exec(req.headers.range || "");
    const from = range ? Number(range[1]) : 0, to = range && range[2] ? Number(range[2]) : size - 1;
    res.writeHead(range ? 206 : 200, { "content-type": "video/webm", "accept-ranges": "bytes", "content-length": to - from + 1, ...(range ? { "content-range": `bytes ${from}-${to}/${size}` } : {}) });
    createReadStream(file, { start: from, end: to }).pipe(res);
}).listen(HTTP, "127.0.0.1");

const profile = join(here, "edge-profile-frames");
rmSync(profile, { recursive: true, force: true });
mkdirSync(profile, { recursive: true });
mkdirSync(dir, { recursive: true });
const edge = spawn(EDGE, ["--headless=new", `--remote-debugging-port=${PORT}`, `--user-data-dir=${profile}`, "--no-first-run", "--autoplay-policy=no-user-gesture-required", "--enable-unsafe-swiftshader", "about:blank"], { stdio: "ignore" });
const quit = code => { try { edge.kill(); } catch (e) { /* gone */ } server.close(); process.exit(code); };
let version;
for (let i = 0; i < 60 && !version; i++) {
    try { version = await (await fetch(`http://127.0.0.1:${PORT}/json/version`)).json(); } catch (e) { await sleep(500); }
}
if (!version) { console.error("Edge did not start"); quit(1); }
const ws = new WebSocket(version.webSocketDebuggerUrl);
await new Promise((ok, no) => { ws.onopen = ok; ws.onerror = no; });
let next = 0;
const waiting = new Map();
ws.onmessage = event => {
    const message = JSON.parse(event.data);
    if (!message.id || !waiting.has(message.id)) return;
    const { ok, no } = waiting.get(message.id);
    waiting.delete(message.id);
    message.error ? no(new Error(JSON.stringify(message.error))) : ok(message.result);
};
const send = (method, params = {}, sessionId) => new Promise((ok, no) => {
    const id = ++next;
    waiting.set(id, { ok, no });
    ws.send(JSON.stringify({ id, method, params, sessionId }));
});
const { targetId } = await send("Target.createTarget", { url: `http://127.0.0.1:${HTTP}/` });
const { sessionId } = await send("Target.attachToTarget", { targetId, flatten: true });
const js = async expression => {
    const r = await send("Runtime.evaluate", { expression, awaitPromise: true, returnByValue: true }, sessionId);
    if (r.exceptionDetails) throw new Error(r.exceptionDetails.exception?.description || r.exceptionDetails.text);
    return r.result.value;
};
await sleep(500);
// The whole file in memory: a recording of the browser has no index to jump by.
const info = await js(`(async () => {
    const video = document.createElement('video');
    video.muted = true;
    video.src = URL.createObjectURL(await (await fetch('/video')).blob());
    await new Promise((ok, no) => { video.onloadedmetadata = ok; video.onerror = () => no(new Error('the video does not load')); });
    window.video = video;
    window.canvas = document.createElement('canvas');
    const k = Math.min(1, 1280 / video.videoWidth);
    canvas.width = Math.round(video.videoWidth * k); canvas.height = Math.round(video.videoHeight * k);
    window.at = async t => {
        await new Promise(ok => { video.onseeked = ok; video.currentTime = t; });
        await new Promise(ok => video.requestVideoFrameCallback ? (video.requestVideoFrameCallback(ok), setTimeout(ok, 400)) : setTimeout(ok, 200));
        canvas.getContext('2d').drawImage(video, 0, 0, canvas.width, canvas.height);
        return canvas.toDataURL('image/jpeg', 0.9).split(',')[1];
    };
    return { seconds: video.duration, width: video.videoWidth, height: video.videoHeight };
})()`);
console.log(`${basename(file)}: ${info.width} x ${info.height}, ${Number(info.seconds).toFixed(1)} s`);
const times = at.length ? at.map(Number) : Array.from({ length: 9 }, (_, i) => Math.round(info.seconds * (i + 1) / 10));
const name = basename(file).replace(/\.[^.]+$/, "");
for (const t of times) {
    const data = await js(`at(${t})`);
    const out = join(dir, `${name}-${String(t).padStart(3, "0")}.jpg`);
    writeFileSync(out, Buffer.from(data, "base64"));
    console.log(out);
}
quit(0);
