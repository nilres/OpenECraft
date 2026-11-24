// Short "It checks your work": a wire of the demo project is deleted, and the Messages
// pane under the page names the connection points left open; undone, they are gone.
// One view of the window, shown as two parts: the page and the messages.

// The wire between the contact of -K1 and the lamp -P1, Undo of the title bar and
// the border between the page and the Messages pane as the window opens.
const [WIRE, UNDO, BORDER] = [[651, 395], [157, 12], 586];

export const start = "window";
export const before = ["window"];
export const views = { window: async () => {} };
export const PARTS = {
    page: { view: "window", from: [437, 205, 330, 290], to: [90, 400, 900, 791] },
    messages: { view: "window", from: [209, 558, 455, 122], to: [0, 1240, 1080, 290] },
};
export const FOOTER = [1640, "openecraft.org"];
export const CAPTIONS = {
    hook: ["Electrical CAD that", "checks your work."],
    cut: ["Delete one wire."],
    found: ["It names every", "open connection."],
    undo: ["Undo.", "The warnings are gone."],
    end: ["Free. Open source.", "Runs in your browser."],
};
// The rows of the three warnings.
export const MARKS = { found: { part: "messages", row: [49, 70] } };

// The demo project is open: its control circuit, near, with a larger Messages pane
// that shows the warnings only.
export async function prepare({ click, drag, sleep }) {
    // The Messages pane under the page again, which the recorder closed: View > Reset pane layout.
    await click(211, 37, 250);
    await sleep(500);
    await click(835, 109, 250);
    await sleep(900);
    await click(227, 159, 250);
    await sleep(600);
    await click(100, 312, 250);
    await sleep(800);
    await click(211, 37, 250);
    await sleep(500);
    // A page is drawn whole only from its second visit.
    await click(100, 286, 250);
    await sleep(700);
    await click(100, 312, 250);
    await sleep(800);
    await drag(600, BORDER, 600, 520, "left", 900);
    await sleep(500);
    await click(34, 75, 250);
    await sleep(800);
    for (let i = 0; i < 3; i++) { await click(110, 87, 250); await sleep(450); }
    await drag(400, 300, 620, 420, "right", 900);
    await sleep(400);
    await drag(600, 522, 600, 497, "left", 600);
    await sleep(500);
    // Without the notes of the demo project.
    await click(380, 570, 250);
    await sleep(600);
    await click(720, 330, 250);
    await sleep(500);
}

export async function play({ say, tap, glide, press, sleep }) {
    await say("hook", async () => {
        await glide(560, 300, 1100);
        await sleep(400);
    });
    await say("cut", async () => {
        await tap(...WIRE, 450);
        await sleep(250);
        await press("Delete");
    });
    await say("found", async () => {
        await glide(520, 640, 700);
    });
    await say("undo", async () => {
        await tap(...UNDO, 500);
        await glide(700, 330, 400);
    });
    await say("end", async () => {
        await glide(600, 300, 900);
    });
}
