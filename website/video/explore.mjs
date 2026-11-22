// For finding the positions a recording clicks at (README.md): a headless Edge with the
// web editor, driven over a small local HTTP server. On Windows start it from PowerShell.
//
//   node explore.mjs <url> <folder for screenshots> [1280x720]
//
// Then POST the body of an async function to http://127.0.0.1:9345/ ; it gets the
// helpers below as `h` and its result comes back as JSON:
//   curl --data-binary 'await h.click(269, 307); return await h.shot("new-project")' http://127.0.0.1:9345/
// http://127.0.0.1:9345/quit ends it.
import { spawn } from "node:child_process";
import { writeFileSync, mkdirSync, rmSync } from "node:fs";
import { join } from "node:path";
import { createServer } from "node:http";

const [site = "http://localhost:8083/", dir = "."] = process.argv.slice(2);
const EDGE = "C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe";
const PORT = 9344;
const [W, H] = (process.argv[4] || "1280x720").split("x").map(Number);
const sleep = ms => new Promise(r => setTimeout(r, ms));

const profile = join(dir, "edge-profile");
rmSync(profile, { recursive: true, force: true });
mkdirSync(profile, { recursive: true });
const edge = spawn(EDGE, [
    "--headless=new", "--lang=en-US", "--accept-lang=en-US,en", `--remote-debugging-port=${PORT}`, `--user-data-dir=${profile}`, `--window-size=${W},${H}`,
    "--hide-scrollbars", "--force-device-scale-factor=1", "--disable-background-timer-throttling",
    "--disable-renderer-backgrounding", "--disable-backgrounding-occluded-windows", "--no-first-run",
    "--enable-unsafe-swiftshader", "--autoplay-policy=no-user-gesture-required", "about:blank",
], { stdio: "ignore" });

let version;
for (let i = 0; i < 60 && !version; i++) {
    try { version = await (await fetch(`http://127.0.0.1:${PORT}/json/version`)).json(); } catch (e) { await sleep(500); }
}
const ws = new WebSocket(version.webSocketDebuggerUrl);
await new Promise((ok, no) => { ws.onopen = ok; ws.onerror = no; });
let next = 0;
const waiting = new Map();
ws.onmessage = event => {
    const message = JSON.parse(event.data);
    if (message.id && waiting.has(message.id)) {
        const { ok, no } = waiting.get(message.id);
        waiting.delete(message.id);
        message.error ? no(new Error(JSON.stringify(message.error))) : ok(message.result);
    }
};
const send = (method, params = {}, sessionId) => new Promise((ok, no) => {
    const id = ++next;
    waiting.set(id, { ok, no });
    ws.send(JSON.stringify({ id, method, params, sessionId }));
});
const { targetId } = await send("Target.createTarget", { url: "about:blank" });
const { sessionId } = await send("Target.attachToTarget", { targetId, flatten: true });
const call = (method, params) => send(method, params, sessionId);
await call("Page.enable");
await call("Runtime.enable");
await call("Emulation.setDeviceMetricsOverride", { width: W, height: H, deviceScaleFactor: 1, mobile: false });
await call("Emulation.setFocusEmulationEnabled", { enabled: true });
await call("Page.navigate", { url: site });
setInterval(() => call("Page.bringToFront").catch(() => {}), 1000);
const js = async expression => {
    const r = await call("Runtime.evaluate", { expression, awaitPromise: true, returnByValue: true });
    if (r.exceptionDetails) throw new Error(r.exceptionDetails.exception?.description || r.exceptionDetails.text);
    return r.result.value;
};
let mouse = { x: 640, y: 360 };
const dispatch = (type, extra = {}) => call("Input.dispatchMouseEvent", { type, x: mouse.x, y: mouse.y, button: "none", ...extra });
const h = {
    sleep, js, call,
    async move(x, y) { mouse = { x, y }; await dispatch("mouseMoved"); },
    async click(x, y, button = "left", count = 1) {
        await h.move(x, y);
        await sleep(40);
        await dispatch("mousePressed", { button, buttons: button === "left" ? 1 : 2, clickCount: count });
        await sleep(50);
        await dispatch("mouseReleased", { button, buttons: 0, clickCount: count });
        await sleep(150);
    },
    async dbl(x, y) { await h.click(x, y); await h.click(x, y, "left", 2); },
    async drag(x0, y0, x1, y1) {
        await h.move(x0, y0);
        await dispatch("mousePressed", { button: "left", buttons: 1, clickCount: 1 });
        for (let i = 1; i <= 12; i++) { mouse = { x: x0 + (x1 - x0) * i / 12, y: y0 + (y1 - y0) * i / 12 }; await dispatch("mouseMoved", { button: "left", buttons: 1 }); await sleep(20); }
        await dispatch("mouseReleased", { button: "left", buttons: 0, clickCount: 1 });
        await sleep(150);
    },
    async wheel(x, y, dy) { await h.move(x, y); await call("Input.dispatchMouseEvent", { type: "mouseWheel", x, y, deltaX: 0, deltaY: dy }); await sleep(100); },
    // A key by its name ("Escape", "Enter", "w", "F2", "Insert"), with modifiers (1 alt, 2 ctrl, 8 shift).
    async key(key, modifiers = 0) {
        const codes = { Escape: 27, Enter: 13, Tab: 9, Backspace: 8, Delete: 46, Insert: 45, ArrowUp: 38, ArrowDown: 40, ArrowLeft: 37, ArrowRight: 39, " ": 32, F2: 113, Home: 36, End: 35 };
        const vk = codes[key] ?? key.toUpperCase().charCodeAt(0);
        const code = key.length === 1 ? (/[a-z]/i.test(key) ? "Key" + key.toUpperCase() : key === " " ? "Space" : /\d/.test(key) ? "Digit" + key : "") : key;
        const text = key.length === 1 && !(modifiers & 3) ? key : key === "Enter" ? "\r" : undefined;
        await call("Input.dispatchKeyEvent", { type: text ? "keyDown" : "rawKeyDown", key, code, windowsVirtualKeyCode: vk, nativeVirtualKeyCode: vk, modifiers, text });
        await call("Input.dispatchKeyEvent", { type: "keyUp", key, code, windowsVirtualKeyCode: vk, nativeVirtualKeyCode: vk, modifiers });
        await sleep(120);
    },
    async type(text) { for (const c of text) { await call("Input.insertText", { text: c }); await sleep(15); } await sleep(120); },
    async shot(name = "shot") {
        const { data } = await call("Page.captureScreenshot", { format: "png" });
        const file = join(dir, name + ".png");
        writeFileSync(file, Buffer.from(data, "base64"));
        return file;
    },
};
createServer(async (req, res) => {
    let body = "";
    for await (const chunk of req) body += chunk;
    if (req.url === "/quit") { res.end("bye"); try { edge.kill(); } catch (e) { /* gone */ } process.exit(0); }
    try {
        const result = await new (Object.getPrototypeOf(async () => {}).constructor)("h", body)(h);
        res.end(JSON.stringify(result ?? null));
    } catch (e) {
        res.statusCode = 500;
        res.end(String(e.stack || e));
    }
}).listen(9345, "127.0.0.1", () => console.log("driver ready"));
