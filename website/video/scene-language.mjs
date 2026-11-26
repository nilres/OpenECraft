// Short "It speaks your language": the bill of materials of the demo project is open,
// and the language of the editor is changed in the status bar: German, French, Spanish,
// Japanese. The ribbon and the generated page follow at once. The window is shown as
// three parts: the ribbon, the page and the list of the languages.

// The languages in the order they are shown: the language button of the status bar
// (its place depends on the texts beside it), the row of the language in its list and
// the left edge of the list, which opens above the button.
const LANGS = { en: [1040, 225, 1021], de: [972, 249, 950], fr: [970, 273, 949], es: [998, 297, 977], ja: [1026, 657, 1009] };
const BAR = 707;
// The entry of a language in the list while the editor shows another one.
const entry = (shown, lang) => [LANGS[shown][2] + 30, LANGS[lang][1]];
const ORDER = Object.keys(LANGS);

export const start = "en";
export const before = ["en"];
// A view is the editor in one language with the list open. The window between two of
// them (the list closes with a choice) is not shown.
export const views = {};
export const PARTS = {};
for (const [i, lang] of ORDER.entries()) {
    const [x, y, left] = LANGS[lang];
    views[lang] = async api => {
        if (i > 0) await api.tap(...entry(ORDER[i - 1], lang), 60);
        await api.tap(x, BAR, 120);
        await api.glide(left + 30, y - 30, 120);
    };
    // The parts of a later language lie over those of the one before.
    PARTS["ribbon-" + lang] = { view: lang, from: [200, 28, 340, 96], to: [20, 400, 827, 233] };
    PARTS["page-" + lang] = { view: lang, from: [222, 240, 440, 455], to: [20, 653, 827, 855] };
    PARTS["list-" + lang] = { view: lang, from: [left, 212, 84, 482], to: [867, 400, 193, 1108] };
}
export const FOOTER = [1640, "openecraft.org"];
export const CAPTIONS = {
    hook: ["Electrical CAD", "in your language."],
    german: ["One click:", "the editor is German."],
    page: ["The drawing follows:", "the bill of materials."],
    more: ["French, Spanish,", "Japanese: 20 in all."],
    end: ["Free. Open source.", "Runs in your browser."],
};

// The demo project is open: its bill of materials, near, with the head of the table.
export async function prepare({ click, drag, sleep }) {
    await click(100, 364, 250);
    await sleep(900);
    await click(227, 159, 250);
    await sleep(600);
    // A page is drawn whole only from its second visit.
    await click(100, 338, 250);
    await sleep(700);
    await click(100, 364, 250);
    await sleep(800);
    await click(211, 37, 250);
    await sleep(500);
    await click(34, 75, 250);
    await sleep(800);
    for (let i = 0; i < 3; i++) { await click(110, 87, 250); await sleep(200); }
    await drag(500, 400, 710, 610, "right", 900);
    await sleep(400);
    // The web editor fetches the texts of Japanese when it is chosen first: before the
    // camera starts, and back to English.
    await click(LANGS.en[0], BAR, 250);
    await sleep(600);
    await click(...entry("en", "ja"), 250);
    await sleep(3500);
    await click(LANGS.ja[0], BAR, 250);
    await sleep(600);
    await click(...entry("ja", "en"), 250);
    await sleep(1500);
}

export async function play({ say, show: front, glide, sleep, shot, mode }) {
    // A trial run keeps a picture of every view.
    const show = async name => { await front(name); if (mode === "check") await shot("language-" + name); };
    await say("hook", async () => {
        await glide(...entry("en", "ja"), 1300);
        await glide(...entry("en", "de"), 1000);
    });
    await say("german", async () => {
        await show("de");
        await glide(380, 88, 700);
    });
    await say("page", async () => {
        await glide(400, 290, 800);
        await glide(520, 330, 900);
        await glide(...entry("de", "fr"), 800);
    });
    await say("more", async () => {
        await show("fr");
        await sleep(200);
        await glide(...entry("fr", "es"), 350);
        await show("es");
        await sleep(200);
        await glide(...entry("es", "ja"), 600);
        await show("ja");
    });
    await say("end", async () => {
        await glide(480, 300, 1100);
    });
}
