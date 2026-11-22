// Short "Nothing to install": the start page of the web editor, an example project
// picked from it, its schematic page and its bill of materials. Two views of the same
// place in the Short: the left of the start page, then the page of the project (the
// second covers the first once the project is open).

export const start = "start";
export const before = ["start"];
export const views = {
    start: async () => {},
    // The page of the project that was opened: its tab, and once to another page and
    // back, as a page is drawn whole only from its second visit.
    project: async api => {
        await api.tap(227, 159, 250);
        await api.sleep(350);
        await api.tap(100, 296, 200);
        await api.sleep(450);
        await api.tap(100, 270, 200);
        await api.sleep(450);
        await api.glide(560, 440, 200);
        await api.sleep(250);
    },
};
export const PARTS = {
    start: { from: [206, 236, 540, 390], to: [0, 500, 1080, 780] },
    project: { from: [276, 200, 668, 482], to: [0, 500, 1080, 780] },
};
export const FOOTER = [1372, "editor.openecraft.org"];
export const CAPTIONS = {
    hook: ["Electrical CAD.", "Nothing to install."],
    pick: ["Open the web editor,", "pick an example."],
    all: ["Schematic, parts list,", "checks: in your browser."],
    end: ["Free.", "Open source."],
};

// The editor as it opens, with its pages set to show whole (View > Fit page, which the
// project opened on camera keeps), then the start page.
export async function prepare({ click, sleep }) {
    await click(227, 159, 250);
    await sleep(600);
    await click(211, 37, 250);
    await sleep(500);
    await click(34, 75, 250);
    await sleep(900);
    await click(294, 159, 250);
    await sleep(800);
}

export async function play({ say, show, tap, glide, sleep }) {
    await say("hook", async () => {
        await glide(400, 330, 900);
        await sleep(400);
    });
    await say("pick", async () => {
        // The first example of the start page, then its page.
        await tap(330, 542, 700);
        await sleep(900);
        await show("project");
    });
    await say("all", async () => {
        // Its bill of materials, and back to the schematic.
        await tap(100, 296, 400);
        await sleep(1700);
        await tap(100, 270, 300);
        await sleep(600);
    });
    await say("end", async () => {
        await glide(600, 420, 700);
    });
}
