// Short "From schematic to cabinet": a schematic page of the demo project above, its
// control cabinet in 3D below, turned and zoomed. Two views: the Schematic workspace
// with the page and the Panel layout workspace with the 3D view.

// The workspace menu of the title bar and its entries.
const [WORKSPACES, SCHEMATIC, PANEL] = [[340, 12], [316, 40], [324, 65]];

export const start = "page";
export const before = ["page"];
export const views = {
    page: async api => { await api.tap(...WORKSPACES, 250); await api.tap(...SCHEMATIC, 200); await api.sleep(1500); },
    cabinet: async api => { await api.tap(...WORKSPACES, 250); await api.tap(...PANEL, 200); await api.sleep(2500); },
};
export const PARTS = {
    page: { from: [357, 211, 506, 358], to: [150, 400, 780, 552] },
    cabinet: { from: [226, 205, 748, 465], to: [0, 980, 1080, 671] },
};
export const FOOTER = [1712, "openecraft.org"];
export const CAPTIONS = {
    hook: ["Draw the schematic."],
    place: ["Place the devices", "in the cabinet."],
    wires: ["Every wire is routed", "and measured for you."],
    end: ["Free. Open source.", "Runs in your browser."],
};

// The demo project is open: its cabinet in 3D, fitted, then its second page whole.
export async function prepare({ click, sleep }) {
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
    await click(858, 75, 250);
    await click(848, 281, 250);
    await sleep(2500);
    await click(...WORKSPACES, 250);
    await click(...SCHEMATIC, 250);
    await sleep(2500);
    await click(100, 278, 250);
    await sleep(900);
    await click(227, 159, 250);
    await sleep(600);
    await click(211, 37, 250);
    await sleep(400);
    await click(34, 75, 250);
    await sleep(700);
    await sleep(500);
}

export async function play({ say, show, glide, pull, sleep }) {
    await say("hook", async () => {
        await glide(520, 330, 900);
        await sleep(500);
    });
    await say("place", async () => {
        await show("cabinet");
        await pull(520, 430, 690, 415, 1500);
    });
    await say("wires", async () => {
        await pull(690, 420, 520, 440, 2000);
    });
    await say("end", async () => {
        await pull(520, 440, 610, 425, 1500);
    });
}
