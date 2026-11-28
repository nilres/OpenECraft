// The video "first circuit": how the first example project is drawn, from a new project
// to the bill of materials. Its steps, for record.mjs; its texts are narration.json.

let click, key, type, sleep;

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

export async function play(api) {
    ({ click, key, type, sleep } = api);
    const { say, move, drag, shot, frame, mode } = api;
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
    if (mode === "check") frame("check-frame");
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
}
