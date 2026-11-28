// The video "self-holding relay": how the second example project is drawn, from a new
// project to the terminal diagram. Its steps, for record.mjs; its texts are
// self-holding-relay.json. The rows are closer than in the example, so that the whole
// circuit is on the screen; the rails end at the last current path.

let click, move, key, type, sleep;

// The pixel of a point of the page in mm, after the view is set in "new".
const P = (x, y) => [274.05 + x * 2.4594, 157.3 + y * 2.4594];
const [TAB_Y, SEARCH, PAGE_TAB] = [37, [1125, 185], [227, 159]];
// What the Insert center finds: its first symbol, its first device, and the second of
// the unplaced functions of -K1 (the first is the main contacts).
const [SYMBOL, DEVICE, UNPLACED] = [[1145, 263], [1115, 324], [1098, 369]];
// The Properties pane: its tabs Device and Parts, the function text, "Add accessory"
// and the first accessory it offers.
const [DEVICE_TAB, PARTS_TAB, FUNCTION_TEXT, ACCESSORY, FIRST_ACCESSORY] = [[1035, 514], [1107, 514], [1200, 581], [1195, 593], [1056, 621]];
// The rows of the circuit: the rails, the circuit breaker, the terminals to the door,
// the push buttons, the contacts, the terminals back and the coil with the lamps.
const [TOP, BREAKER, OUT, STOP, START, BACK, COIL, BOTTOM] = [24, 44, 64, 88, 112, 140, 168, 208];
// Its current paths: the relay, its self-holding contact and the two lamps.
const [MAIN, HOLD, ON, OFF] = [60, 88, 160, 220];

// A click of the routine kind: the pointer is there faster than for one to be watched.
const tap = (x, y) => click(x, y, 220);
async function search(text) {
    await tap(...SEARCH);
    await key("a", 2);
    await type(text, 45);
    await sleep(300);
}
// An entry of the Insert center placed on the page.
async function put(entry, x, y) {
    await tap(...entry);
    await click(...P(x, y), 400);
    await sleep(300);
    await key("Escape");
}
async function place(text, x, y) {
    await search(text);
    await put(DEVICE, x, y);
}
// A potential, turned to point along its rail.
async function potential(text, y, each) {
    await search(text);
    await sleep(300);
    await tap(...SYMBOL);
    await move(...P(20, y), 450);
    await sleep(each);
    for (let i = 0; i < 3; i++) { await key("r"); await sleep(each); }
    await click(...P(20, y), 300);
    await sleep(400);
    await key("Escape");
}
async function wire(points) {
    for (const p of points) await click(...P(...p), 320);
    await key("Enter");
    await sleep(300);
}

export async function play(api) {
    ({ click, move, key, type, sleep } = api);
    const { say, drag, zoomAt, shot, frame, mode } = api;
    const check = async name => { if (mode === "check") { await sleep(600); await shot("check-" + name); } };
    await say("intro", async () => {
        await move(520, 300, 1200);
        await sleep(1200);
        await move(330, 585, 1200);
        await sleep(3200);
        await move(269, 307, 1200);
    });
    await say("new", async () => {
        await tap(269, 307);
        await sleep(700);
        await tap(...PAGE_TAB);
        await sleep(700);
        await tap(211, TAB_Y);
        await sleep(300);
        await tap(34, 75);
        await sleep(800);
        for (let i = 0; i < 2; i++) { await click(110, 87, 300); await sleep(350); }
        await sleep(300);
        await drag(500, 350, 677, 431, "right");
    });
    if (mode === "check") { await move(...P(100, 100), 300); await check("view"); frame("check-frame"); }
    await say("pot24", async () => {
        await sleep(2000);
        await potential("Potential +24V", TOP, 500);
    });
    await say("pot0", async () => {
        await potential("Potential 0V", BOTTOM, 150);
    });
    await check("potentials");
    await say("rails", async () => {
        await tap(121, TAB_Y);
        await sleep(500);
        await tap(283, 75);
        await sleep(1000);
        await wire([[24, TOP], [OFF, TOP]]);
        await wire([[24, BOTTOM], [OFF, BOTTOM]]);
        await key("Escape");
    });
    await check("rails");
    await say("breaker", async () => {
        await sleep(3800);
        await search("MCB-1P-C6");
        await sleep(400);
        await tap(...DEVICE);
        await move(...P(MAIN, BREAKER), 700);
        await sleep(600);
        await key("r");
        await sleep(800);
        await click(...P(MAIN, BREAKER), 300);
        await sleep(500);
        await key("Escape");
    });
    await say("terminal1", async () => {
        await sleep(3800);
        await place("TB-2.5", MAIN, OUT);
    });
    await say("buttons", async () => {
        await place("PB-RD-1NC", MAIN, STOP);
        await sleep(400);
        await place("PB-GN-1NO", MAIN, START);
    });
    await say("terminal2", async () => {
        await place("TB-2.5", MAIN, BACK);
    });
    await say("coil", async () => {
        await sleep(800);
        await place("CT-9A-24DC", MAIN, COIL);
    });
    await check("devices");
    await say("wires1", async () => {
        await sleep(1800);
        await move(...P(100, 40), 600);
        await key("w");
        await sleep(900);
        await wire([[MAIN, TOP], [MAIN, BREAKER - 4]]);
        await wire([[MAIN, BREAKER + 4], [MAIN, OUT - 4]]);
        await wire([[MAIN, COIL + 4], [MAIN, BOTTOM]]);
        await key("Escape");
        await move(...P(120, 100), 700);
    });
    await check("main");
    await say("accessory", async () => {
        await sleep(2300);
        await click(...P(MAIN, COIL));
        await sleep(1400);
        await click(...PARTS_TAB);
        await sleep(2400);
        await click(...ACCESSORY);
        await sleep(3600);
        await click(...FIRST_ACCESSORY);
        await sleep(600);
    });
    await check("accessory");
    await say("unplaced", async () => {
        await sleep(400);
        await search("K1");
        await sleep(800);
        await move(1120, 345, 700);
        await sleep(3000);
        await move(...UNPLACED, 500);
        await sleep(1500);
        await put(UNPLACED, HOLD, START);
    });
    await check("contact");
    await say("hold", async () => {
        await key("w");
        await sleep(400);
        await wire([[MAIN, START - 12], [HOLD, START - 12], [HOLD, START - 4]]);
        await wire([[HOLD, START + 4], [HOLD, START + 12], [MAIN, START + 12]]);
        await key("Escape");
        await move(...P(HOLD + 14, START), 800);
    });
    await check("hold");
    await say("lamp1", async () => {
        await sleep(2600);
        await put(UNPLACED, ON, START);
        await place("TB-2.5", ON, BACK);
        await place("LMP-GN-24", ON, COIL);
    });
    await say("lamp2", async () => {
        await search("K1");
        await sleep(300);
        await put(UNPLACED, OFF, START);
        await place("TB-2.5", OFF, BACK);
        await place("LMP-RD-24", OFF, COIL);
    });
    await check("lamps");
    await say("wires2", async () => {
        await move(...P(190, 60), 500);
        await key("w");
        await sleep(400);
        await wire([[ON, TOP], [ON, START - 4]]);
        await wire([[OFF, TOP], [OFF, START - 4]]);
        await wire([[ON, COIL + 4], [ON, BOTTOM]]);
        await wire([[OFF, COIL + 4], [OFF, BOTTOM]]);
        await key("Escape");
        await move(...P(120, 80), 700);
    });
    await check("circuit");
    await say("closeup", async () => {
        const near = P(96, 176);
        await sleep(300);
        await zoomAt(...near, 16);
        if (mode === "check") { await check("closeup"); frame("check-closeup"); }
        // What the view shows of the page is 2.23 times as large now.
        const N = (x, y) => [near[0] + (P(x, y)[0] - near[0]) * 2.23, near[1] + (P(x, y)[1] - near[1]) * 2.23];
        await sleep(1000);
        await move(...N(HOLD - 5, START + 2), 600);
        await sleep(2600);
        await move(...N(MAIN - 8, COIL), 700);
        await sleep(600);
        await move(...N(MAIN + 16, COIL + 9), 600);
        await sleep(2400);
        await move(...N(MAIN + 18, COIL + 20), 600);
        await sleep(2000);
        await zoomAt(...near, -16);
    });
    if (mode === "check") { await move(...P(100, 100), 300); await check("back"); }
    await say("text", async () => {
        await click(...P(MAIN, STOP));
        await sleep(500);
        await tap(...DEVICE_TAB);
        await sleep(400);
        await tap(...FUNCTION_TEXT);
        await type("Stop");
        await key("Enter");
        await sleep(400);
        await click(...P(MAIN, START));
        await sleep(400);
        await tap(...FUNCTION_TEXT);
        await type("Start");
        await key("Enter");
        await sleep(300);
        await tap(...P(120, 60));
    });
    await check("texts");
    await say("numbers", async () => {
        await tap(342, TAB_Y);
        await sleep(600);
        await tap(164, 75);
        await sleep(1800);
        await tap(399, 221);
        await sleep(900);
        await tap(437, 460);
        await sleep(1400);
        if (mode === "check") await check("numbering");
        await tap(1052, 98);
        await move(...P(110, 90), 800);
    });
    await check("numbers");
    await say("check", async () => {
        await sleep(600);
        await move(950, 700, 1200);
    });
    await say("terminals", async () => {
        await sleep(3000);
        await tap(430, TAB_Y);
        await sleep(500);
        await tap(235, 72);
        await sleep(1300);
        await tap(379, 331);
        await sleep(700);
        await move(820, 370, 900);
        if (mode === "check") await check("terminal-diagram");
        await sleep(2800);
        await click(986, 237);
        await sleep(1000);
    });
    await say("connections", async () => {
        await tap(372, 481);
        await sleep(700);
        await move(760, 330, 800);
        if (mode === "check") await check("connection-list");
        await sleep(3000);
        await click(965, 237);
        await sleep(1000);
        await tap(1246, 157);
    });
    await check("report-page");
    await say("pages", async () => {
        await tap(100, 270);
        await sleep(800);
        await tap(211, TAB_Y);
        await sleep(400);
        await tap(34, 75);
        await move(600, 400, 800);
    });
    await check("pages");
    await say("outro", async () => {
        await tap(75, 244);
        await sleep(900);
        await tap(34, 75);
        await sleep(600);
        await move(700, 420, 900);
    });
    await check("end");
}
