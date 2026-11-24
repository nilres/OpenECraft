// Short "The wires come along": the lamp of the demo project is turned and a contact
// mirrored, their wires follow, and the Messages pane under the page stays without
// a warning. One view of the window, shown as two parts: the page and the messages.

// The lamp -P1 and the contact of -K1 beside the start button.
const [LAMP, CONTACT] = [[760, 454], [589, 297]];

export const start = "window";
export const before = ["window"];
export const views = { window: async () => {} };
export const PARTS = {
    page: { view: "window", from: [475, 205, 330, 290], to: [90, 400, 900, 791] },
    messages: { view: "window", from: [209, 558, 455, 122], to: [0, 1240, 1080, 290] },
};
export const FOOTER = [1640, "openecraft.org"];
export const CAPTIONS = {
    hook: ["Turn a device.", "Its wires come along."],
    turn: ["Select the lamp,", "press R."],
    mirror: ["Press M", "to mirror a contact."],
    proof: ["Still connected.", "Nothing to redraw."],
    end: ["Free. Open source.", "Runs in your browser."],
};
// The row "No problems found".
export const MARKS = { proof: { part: "messages", row: [49, 24] } };

// The demo project is open: its control circuit, near, with a larger Messages pane
// that shows the warnings only (as in scene-checks.mjs).
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
    await drag(600, 586, 600, 520, "left", 900);
    await sleep(500);
    await click(34, 75, 250);
    await sleep(800);
    for (let i = 0; i < 5; i++) { await click(110, 87, 250); await sleep(450); }
    await drag(400, 300, 826, 460, "right", 900);
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
        await glide(680, 330, 1100);
        await sleep(400);
    });
    await say("turn", async () => {
        await tap(...LAMP, 500);
        await sleep(350);
        await press("r");
        await glide(700, 370, 400);
    });
    await say("mirror", async () => {
        await tap(...CONTACT, 500);
        await sleep(350);
        await press("m");
        await glide(680, 330, 400);
    });
    await say("proof", async () => {
        // Nothing selected, so the wires show as they are.
        await tap(680, 330, 400);
        await glide(520, 640, 600);
    });
    await say("end", async () => {
        await glide(600, 300, 900);
    });
}
