// Short "Open the door": the control cabinet of the demo project in 3D with its door
// shut; the door opens, the devices on it and their wires turn with it, and it opens
// wider, to 180 degrees. One view of the window, shown as two parts: the button with
// the opening angle and the 3D view.

// The workspace menu of the title bar and its entry "Panel layout", the button
// "Open door" / "Close door" of the 3D view and the opening angle beside it.
const [WORKSPACES, PANEL, DOOR, ANGLE] = [[340, 12], [324, 65], [258, 181], [321, 181]];

export const start = "window";
export const before = ["window"];
export const views = { window: async () => {} };
export const PARTS = {
    button: { view: "window", from: [222, 170, 130, 24], to: [312, 400, 455, 84] },
    cabinet: { view: "window", from: [300, 205, 570, 485], to: [0, 510, 1080, 919] },
};
export const FOOTER = [1540, "openecraft.org"];
export const CAPTIONS = {
    hook: ["Open the cabinet door.", "In your browser."],
    open: ["One click:", "the door swings open."],
    wires: ["Lamps, buttons, wires:", "they turn with it."],
    angle: ["Open it wider,", "up to 180 degrees."],
    end: ["Free. Open source.", "Runs in your browser."],
};

// The demo project is open: its cabinet in 3D, fitted, the door shut, a little to the
// right, so that the open door has room at the left.
export async function prepare({ click, drag, sleep }) {
    await click(...WORKSPACES, 250);
    await click(...PANEL, 250);
    await sleep(3500);
    // The Messages pane of this workspace makes room for the cabinet.
    await click(1000, 598, 250);
    await click(1034, 648, 250);
    await sleep(700);
    await click(858, 75, 250);
    await click(863, 137, 250);
    await sleep(10000);
    await click(...DOOR, 250);
    await sleep(2500);
    await click(858, 75, 250);
    await click(848, 281, 250);
    await sleep(2500);
    await drag(600, 450, 670, 450, "right", 900);
    await sleep(800);
}

export async function play({ say, tap, glide, pull, sleep }) {
    await say("hook", async () => {
        await pull(560, 440, 584, 437, 800);
        await pull(584, 437, 560, 440, 800);
    });
    await say("open", async () => {
        await tap(...DOOR, 400);
        await glide(600, 450, 500);
    });
    await say("wires", async () => {
        await pull(600, 440, 560, 446, 900);
        await pull(560, 446, 600, 440, 900);
    });
    await say("angle", async () => {
        // A pixel is a degree; more than the 74 to 180, where it stops.
        await pull(...ANGLE, ANGLE[0] + 95, ANGLE[1], 1300);
        await glide(600, 450, 500);
    });
    await say("end", async () => {
        await pull(600, 440, 540, 448, 2200);
    });
}
