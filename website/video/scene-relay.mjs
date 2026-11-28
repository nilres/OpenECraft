// Short "A relay that holds itself": the second example project without the
// self-holding contact of -K1. The contact is picked from the unplaced functions of
// the relay's parts, clicked into the page and wired in parallel to the start button;
// the contact image below the coil lists it. One view of the window, shown as two
// parts: the unplaced functions of the Insert center and the page.

// The pixel of a point of the page in mm, after the view is set in `prepare`.
const P = (x, y) => [150.1 + x * 3.074, -100.7 + y * 3.074];
// The search field of the Insert center and the second of the unplaced functions it
// finds for "K1" (the first is the main contacts).
const [SEARCH, UNPLACED] = [[1125, 185], [1098, 369]];
// The start button -S2 and the place of the contact beside it; the wires leave the
// current path above and below the button.
const [MAIN, HOLD, START] = [60, 88, 124];

export const start = "window";
export const before = ["window"];
export const views = { window: async () => {} };
export const PARTS = {
    unplaced: { view: "window", from: [1018, 333, 258, 72], to: [190, 395, 700, 195] },
    page: { view: "window", from: [232, 213, 291, 369], to: [163, 615, 754, 957] },
};
export const FOOTER = [1650, "openecraft.org"];
export const CAPTIONS = {
    hook: ["A relay that", "holds itself."],
    pick: ["Pick its free contact", "from the part."],
    wires: ["Two wires:", "parallel to Start."],
    image: ["It is -K1 13/14.", "The coil lists it."],
    end: ["Free. Open source.", "Runs in your browser."],
};
// The contact image below the coil.
export const MARKS = { image: { part: "page", row: [309, 56] } };

// The second example is open: its control circuit, near, without the self-holding
// contact and its wires; the Insert center lists the unplaced functions of -K1.
export async function prepare({ click, drag, key, press, type, move, sleep }) {
    await click(328, 585, 250);
    await sleep(2500);
    await click(100, 270, 250);
    await sleep(1000);
    await click(211, 37, 250);
    await sleep(500);
    await click(34, 75, 250);
    await sleep(900);
    for (let i = 0; i < 3; i++) { await click(110, 87, 250); await sleep(450); }
    await drag(400, 400, 590, 309.5, "right", 900);
    await sleep(500);
    // From right to left, the frame takes what it touches: the contact and its wires.
    await drag(...P(98, 146), ...P(68, 102), "left", 900);
    await sleep(500);
    await press("Delete");
    await sleep(600);
    await click(...SEARCH, 250);
    await key("a", 2);
    await type("K1");
    await sleep(600);
    await move(...P(112, 150), 400);
    await sleep(600);
}

export async function play({ say, tap, glide, press, sleep }) {
    await say("hook", async () => {
        await glide(...P(84, 176), 1200);
        await sleep(500);
        await glide(...P(100, 128), 1000);
    });
    await say("pick", async () => {
        await sleep(1500);
        await tap(...UNPLACED, 700);
        await sleep(300);
        await tap(...P(HOLD, START), 800);
        await press("Escape");
    });
    await say("wires", async () => {
        await glide(...P(74, START - 20), 300);
        await press("w");
        for (const p of [[MAIN, START - 16], [HOLD, START - 16], [HOLD, START - 4]]) await tap(...P(...p), 350);
        await press("Enter");
        for (const p of [[HOLD, START + 4], [HOLD, START + 16], [MAIN, START + 16]]) await tap(...P(...p), 350);
        await press("Enter");
        await press("Escape");
        await glide(...P(108, START + 6), 500);
    });
    await say("image", async () => {
        await glide(...P(HOLD + 8, START + 2), 700);
        await sleep(1300);
        await glide(...P(MAIN + 22, 206), 1100);
    });
    await say("end", async () => {
        await glide(...P(112, 150), 1200);
    });
}
