// Short "Watch the parts list": a lamp more in the schematic, and the bill of materials
// counts it. Two views of the window: the page and the bill of materials as a tab.

// The pixel of a point of the page in mm, after the view is set in `prepare`.
const P = (x, y) => [225.2 + x * 3.065, 196.85 + y * 3.065];
const [TAB_Y, SEARCH, PAGE_TAB, REPORTS_TAB, DEVICE] = [37, [1125, 185], [227, 159], [372, 159], [1115, 324]];

export const start = "table";
export const before = ["table", "page"];
export const views = {
    page: api => api.tap(...PAGE_TAB, 250),
    table: api => api.tap(...REPORTS_TAB, 250),
};
export const PARTS = {
    table: { from: [425, 240, 395, 98], to: [0, 400, 1080, 268] },
    page: { from: [240, 268, 460, 424], to: [62, 716, 955, 880] },
};
export const CAPTIONS = {
    hook: ["Free electrical CAD.", "Watch the parts list."],
    add: ["Place one more lamp", "and wire it."],
    count: ["The parts list", "counts it by itself."],
    undo: ["Undo.", "The list follows."],
    end: ["Free. Open source.", "Runs in your browser."],
};
// The row of the green lamp in the list.
export const MARKS = { count: { part: "table", row: [25.5, 23.5] }, undo: { part: "table", row: [25.5, 23.5] } };

// The first circuit, drawn fast, and its bill of materials as a tab beside the page.
export async function prepare({ click, drag, key, type, sleep }) {
    const place = async (text, entry, x, y) => {
        await click(...SEARCH, 250);
        await key("a", 2);
        await type(text, 20);
        await sleep(500);
        await click(...entry, 250);
        await click(...P(x, y), 250);
        await sleep(300);
        await key("Escape");
    };
    const wire = async points => {
        for (const p of points) await click(...P(...p), 250);
        await key("Enter");
        await sleep(250);
    };
    await click(269, 307, 250);
    await sleep(1200);
    await click(...PAGE_TAB, 250);
    await click(211, TAB_Y, 250);
    await sleep(400);
    await click(34, 75, 250);
    await sleep(700);
    for (let i = 0; i < 3; i++) { await click(110, 87, 200); await sleep(300); }
    await drag(500, 350, 764, 560, "right");
    await place("24", [1145, 263], 24, 32);
    await place("0V", [1165, 263], 24, 148);
    await place("MCB-1P-C6", DEVICE, 60, 56);
    await place("PB-GN-1NO", DEVICE, 60, 92);
    await place("SEL-2POS-1NO", DEVICE, 120, 92);
    await place("LMP-RD-24", DEVICE, 120, 124);
    // The green lamp last: the Insert center keeps showing it for the one placed on camera.
    await place("LMP-GN-24", DEVICE, 60, 124);
    await click(121, TAB_Y, 250);
    await sleep(400);
    await click(283, 75, 250);
    await sleep(300);
    await wire([[24, 36], [60, 36], [60, 52]]);
    await wire([[60, 60], [60, 88]]);
    await wire([[60, 72], [120, 72], [120, 88]]);
    await wire([[24, 152], [120, 152], [120, 128]]);
    await wire([[60, 128], [60, 152]]);
    await key("Escape");
    await click(430, TAB_Y, 250);
    await sleep(500);
    await click(235, 72, 250);
    await sleep(1500);
    await click(374, 306, 250);
    await sleep(800);
    await click(347, 191, 250);
    await sleep(1200);
    await click(121, TAB_Y, 250);
    await sleep(500);
}

export async function play({ say, show, tap, glide, press, sleep }) {
    await say("hook", async () => {
        await glide(...P(100, 110), 1200);
        await sleep(600);
    });
    await say("add", async () => {
        await tap(...DEVICE, 400);
        await tap(...P(90, 124), 650);
        await sleep(350);
        await press("Escape");
        await tap(283, 75, 350);
        for (const points of [[[90, 120], [90, 108], [60, 108]], [[90, 128], [90, 152]]]) {
            for (const p of points) await tap(...P(...p), 330);
            await press("Enter");
        }
        await press("Escape");
        await glide(...P(150, 110), 300);
        await sleep(300);
    });
    await say("count", async () => {
        await show("table");
        await glide(700, 300, 400);
    });
    await say("undo", async () => {
        await show("page");
        // Undone: both wires and the lamp.
        for (let i = 0; i < 3; i++) { await tap(157, 12, 200); await sleep(260); }
        await glide(...P(150, 110), 250);
        await sleep(350);
        await show("table");
    });
    await say("end", async () => {
        await show("page");
        await glide(...P(150, 100), 600);
    });
}
