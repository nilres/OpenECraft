// The OpenECraft website: connects the page to the studio (the editor's code compiled
// to WebAssembly, `src/studio.rs`). The studio answers `call(function, argument)` with
// JSON and draws the canvases; everything else on the page is drawn here from those
// answers. Texts are written in English and translated through `t` / data-t.
"use strict";
(() => {
    const $ = (selector, root = document) => root.querySelector(selector);
    const $$ = (selector, root = document) => [...root.querySelectorAll(selector)];
    const CONSOLE_LIMIT = 6000;
    const COMMAND_ROWS = 80;

    // The language of the page as it was written: de.html shows German.
    const pageLang = document.documentElement.lang || "en";
    // The module and its WebAssembly, named by trunk in the page (Trunk.toml).
    const [moduleUrl, wasmUrl] = (window.OPENECRAFT_MODULE || []).map(a => new URL(a, document.baseURI).href);
    let loading = null;
    // The canvas a visitor clicked before the module was there.
    let wanted = null;
    let api = null;
    let texts = {};
    let state = null;
    let stateText = "";
    let sheetKey = "";
    let placesKey = "";
    let report = "";
    let reports = [];
    let commands = [];
    let exporters = [];
    const history = [];
    let historyAt = 0;
    // Examples about to be opened: their loading animation stays until they are.
    let opening = 0;
    // The worker that searches the 3D routing (worker.js), once it is ready, and the
    // number of the search it was last asked for.
    let worker = null;
    let searching = null;
    // The routings found, by language and project: an example opened again, or shown
    // in a language it was shown in before, is not searched again. `kept` has those of
    // this visit, `store` (a cache of the browser, named after the build of the
    // module) those of earlier visits too.
    const KEPT = 24;
    const kept = new Map();
    let store = null;
    // The routings of the examples from the build of the page (routing/index.json).
    let prebuilt = [];
    // The searches the worker is working on: the key of each by its number.
    const jobs = new Map();

    const t = english => texts[english] || english;
    const fill = (text, ...values) => values.reduce((out, v) => out.replace("{}", v), text);

    function call(method, arg = "") {
        const answer = JSON.parse(api.call(method, String(arg)));
        if (answer && answer.error) throw new Error(answer.error);
        return answer;
    }

    function element(tag, className, text) {
        const e = document.createElement(tag);
        if (className) e.className = className;
        if (text !== undefined) e.textContent = text;
        return e;
    }

    // ---- static parts: links, texts, sections fading in -------------------------------

    function links() {
        // Where the links lead is set in config.js; a link without an address is hidden.
        const site = window.OPENECRAFT_SITE || {};
        const address = name => String(site[name] || "").trim();
        const folder = name => address(name).replace(/\/+$/, "");
        const repository = folder("repository");
        const targets = {
            editor: address("editor"),
            repository,
            downloads: address("downloads") || (repository && repository + "/releases/latest"),
            documentation: folder("documentation") || (repository && repository + "/blob/master/docs"),
            issues: address("issues") || (repository && repository + "/issues"),
        };
        for (const a of $$("[data-link]")) {
            const target = targets[a.dataset.link];
            a.hidden = !target;
            if (target) a.href = target + (a.dataset.path || "");
            if (a.dataset.link !== "editor") a.rel = "noopener";
        }
    }

    function legal() {
        // The legal notice of config.js (legal): shown only with the operator's name, a
        // line of it only with its value. Texts, never markup.
        const notice = (window.OPENECRAFT_SITE || {}).legal || {};
        const field = name => String(notice[name] || "").trim();
        const lines = name => field(name).split(/\s*(?:\n|\|)\s*/).filter(Boolean);
        for (const e of $$("[data-legal]")) {
            const name = e.dataset.legal;
            if (name === "address") {
                lines(name).forEach((line, i) => e.append(...(i ? [document.createElement("br"), line] : [line])));
            } else if (name === "text" || name === "privacy") {
                for (const line of lines(name)) e.append(element("p", "", line));
            } else {
                e.textContent = field(name);
            }
            if (name === "email") e.href = "mailto:" + field(name);
            (e.closest("dl > div") || e).hidden = !field(name);
        }
        // The licenses and the privacy notice are always there, the legal notice with a name.
        for (const name of ["licenses", "privacy", "legal"]) {
            const dialog = $("#" + name);
            const open = $(`#${name}-open`);
            if (!dialog.showModal || (name === "legal" && !field("name"))) {
                open.hidden = true;
                continue;
            }
            open.hidden = false;
            open.addEventListener("click", e => {
                e.preventDefault();
                dialog.showModal();
            });
            // A link to #legal, #privacy or #licenses opens it as well.
            const follow = () => location.hash === "#" + name && !dialog.open && dialog.showModal();
            addEventListener("hashchange", follow);
            follow();
        }
    }

    function translate() {
        for (const e of $$("[data-t]")) {
            e.dataset.t ||= e.textContent.trim();
            e.textContent = t(e.dataset.t);
        }
        for (const e of $$("[data-t-placeholder]")) e.placeholder = t(e.dataset.tPlaceholder);
        for (const e of $$("[data-t-alt]")) e.alt = t(e.dataset.tAlt);
    }

    function reveal() {
        // By position, not with an observer: a section must never stay hidden because
        // the browser did not report it (a tab in the background, printing).
        const bands = $$(".band");
        const look = () => {
            for (const band of bands) if (band.getBoundingClientRect().top < innerHeight * 0.9) band.classList.add("seen");
        };
        for (const band of bands) band.classList.add("reveal");
        addEventListener("scroll", look, { passive: true });
        addEventListener("resize", look);
        look();
    }

    // ---- the canvases --------------------------------------------------------------------

    function stages() {
        const start = new IntersectionObserver(entries => {
            for (const entry of entries) {
                const stage = entry.target;
                if (!entry.isIntersecting) {
                    stage.classList.remove("live");
                } else if (!stage.dataset.started) {
                    stage.dataset.started = "1";
                    stage.classList.add("waiting");
                    try {
                        api.start($("canvas", stage).id, stage.dataset.view);
                    } catch (e) {
                        $(".veil span", stage).textContent = t("The demo could not start in this browser. It needs WebGL and WebAssembly.");
                    }
                    setTimeout(() => stage.classList.remove("waiting"), 1500);
                }
            }
        }, { rootMargin: "200px" });
        for (const stage of $$(".stage")) {
            start.observe(stage);
            $(".veil", stage).addEventListener("click", () => activate(stage));
        }
        // A click elsewhere gives the wheel back to the page.
        document.addEventListener("pointerdown", e => {
            for (const stage of $$(".stage.live")) if (!stage.parentElement.contains(e.target)) stage.classList.remove("live");
        });
    }

    function activate(stage) {
        stage.classList.add("live");
        $("canvas", stage).focus({ preventScroll: true });
    }

    // ---- the state of the project ------------------------------------------------------

    function poll() {
        let next;
        try {
            next = call("state");
        } catch (e) {
            return;
        }
        if (!next || next.busy) return;
        if (next.search !== null && next.search !== searching) search(next.search);
        for (const stage of $$(".stage")) stage.classList.toggle("loading", next.loading || opening > 0);
        const text = JSON.stringify(next);
        if (text === stateText) return;
        const before = state;
        state = next;
        stateText = text;
        render(before);
    }

    function render(before) {
        const pages = $("#pages");
        pages.replaceChildren(...state.pages.map((p, i) => {
            const li = element("li", i === state.page ? "on" : "");
            li.append(element("b", "", p.name), element("span", "", p.description));
            li.addEventListener("click", () => act("page", i));
            return li;
        }));
        // Keeps the page shown in view inside the list, without scrolling the document.
        const on = $(".on", pages);
        if (on && (on.offsetTop < pages.scrollTop || on.offsetTop + on.offsetHeight > pages.scrollTop + pages.clientHeight)) {
            pages.scrollTop = on.offsetTop - pages.clientHeight / 2;
        }

        const issues = $("#issues");
        const problems = state.issues.filter(i => i.severity !== "info").length;
        const count = $("#issue-count");
        count.textContent = state.loading ? "…" : state.issues.length;
        count.className = "count " + (problems ? "bad" : "ok");
        if (state.issues.length) {
            issues.replaceChildren(...state.issues.map((issue, i) => {
                const li = element("li", issue.severity);
                li.append(element("span", "", issue.message));
                li.addEventListener("click", () => act("issue", i));
                return li;
            }));
        } else {
            issues.replaceChildren(element("li", "none", state.loading ? t("Loading…") : t("No messages. The project passes the check.")));
        }

        $("#stat-devices").textContent = state.devices;
        $("#stat-connections").textContent = state.connections;
        $("#stat-pages").textContent = state.pages.length;
        $("#hint").textContent = state.message || state.hint;
        $("#undo").disabled = !state.undo;
        $("#redo").disabled = !state.redo;
        for (const b of $$("[data-tool]")) b.classList.toggle("on", b.dataset.tool === state.tool);
        for (const b of $$("[data-view-mode]")) b.classList.toggle("on", b.dataset.viewMode === state.view);
        $("#wires").classList.toggle("on", state.show_wires);
        places();
        for (const b of $$("[data-example]")) b.classList.toggle("on", b.dataset.example === state.example);

        if (!state.loading) {
            sheet();
            if (report) showReport();
            if (!before || before.lang !== state.lang) $("#language").value = state.lang;
        }
    }

    // The enclosures, the rooms and the site of the project, for the panel layout view.
    function places() {
        const select = $("#place");
        const key = JSON.stringify([state.places, state.site, state.lang]);
        if (key !== placesKey) {
            placesKey = key;
            const option = (value, text) => {
                const o = element("option", "", text);
                o.value = value;
                return o;
            };
            const group = (label, room) => {
                const g = document.createElement("optgroup");
                g.label = label;
                state.places.forEach((p, i) => p.room === room && g.append(option(i, p.name)));
                return g;
            };
            select.replaceChildren(...(state.site ? [option("site", t("Site"))] : []), ...[group(t("Enclosures"), false), group(t("Rooms"), true)].filter(g => g.children.length));
            select.hidden = select.options.length < 2;
        }
        select.value = state.place;
        $("#drawing").textContent = state.place_kind === "site" ? t("Site plan") : state.place_kind === "room" ? t("Floor plan") : t("Mounting plate");
    }

    // The sheet of the hero: the page being edited, as the SVG export draws it.
    function sheet() {
        const key = stateText;
        if (key === sheetKey) return;
        sheetKey = key;
        try {
            const svg = new DOMParser().parseFromString(call("svg", state.page), "image/svg+xml").documentElement;
            if (svg.nodeName !== "svg") return;
            svg.removeAttribute("width");
            svg.removeAttribute("height");
            $("#hero-sheet").replaceChildren(svg);
            $(".sheet-note").hidden = false;
        } catch (e) { /* the page went away meanwhile: the next state draws it */ }
    }

    // A function of the studio after a click, with the state drawn at once.
    function act(method, arg = "") {
        try {
            call(method, arg);
        } catch (e) {
            print(e.message, "err");
        }
        poll();
    }

    // ---- the worker ------------------------------------------------------------------------

    // The 3D routing of a big project takes seconds. A page has one thread, so a
    // second instance of the module searches it in a worker and the page stays
    // usable; without a worker (an old browser) the page searches it itself.
    function startWorker() {
        const [module, wasm] = [moduleUrl, wasmUrl];
        if (!module || !wasm || !window.Worker) return;
        let started;
        const failed = () => {
            started.terminate();
            if (worker !== started) return;
            worker = null;
            try {
                call("worker", "off");
            } catch (e) { /* the search under way ends below */ }
            if (searching !== null) routed(searching, "");
        };
        try {
            started = new Worker("worker.js", { type: "module" });
        } catch (e) {
            return;
        }
        started.onerror = failed;
        started.onmessage = event => {
            const answer = event.data;
            if (answer.failed) {
                failed();
            } else if (answer.ready) {
                worker = started;
                call("worker", "on");
            } else {
                found(answer.search, answer.routing);
            }
        };
        started.postMessage({ module, wasm });
        openStore(module);
        fetch("routing/index.json").then(answer => answer.json()).then(files => {
            if (Array.isArray(files)) prebuilt = files;
        }).catch(() => {});
    }

    // The routing of an unchanged example from the build of the page: the one of its
    // language, or the one for every language. The worker takes it if it fits.
    async function built(job) {
        const file = [`${job.example}.${job.lang}.json`, `${job.example}.json`].find(f => prebuilt.includes(f));
        if (!job.example || !file) return "";
        try {
            const answer = await fetch("routing/" + file);
            return answer.ok ? await answer.text() : "";
        } catch (e) {
            return "";
        }
    }

    // The cache of the browser for the routings of this build of the module; those
    // of other builds are dropped. Without one (no secure context, private mode) the
    // routings are kept for this visit only.
    async function openStore(module) {
        try {
            const name = "openecraft-routing-" + module.split("/").pop();
            for (const other of await caches.keys()) if (other.startsWith("openecraft-routing-") && other !== name) await caches.delete(other);
            store = await caches.open(name);
        } catch (e) {
            store = null;
        }
    }

    // The address of a routing in the cache of the browser: a digest of what it is of.
    async function address(key) {
        const digest = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(key));
        return "routing/" + [...new Uint8Array(digest)].map(b => b.toString(16).padStart(2, "0")).join("");
    }

    async function stored(key) {
        if (!store) return "";
        try {
            const answer = await store.match(await address(key));
            return answer ? await answer.text() : "";
        } catch (e) {
            return "";
        }
    }

    function keep(key, routing) {
        kept.delete(key);
        kept.set(key, routing);
        if (kept.size > KEPT) kept.delete(kept.keys().next().value);
    }

    // The routing for the search the studio asks for: one kept from before, or the
    // worker searches it for the project.
    function search(number) {
        if (!worker) return routed(number, "");
        let job;
        try {
            job = call("project");
        } catch (e) {
            return;
        }
        if (!job || job.busy) return;
        searching = job.search;
        const key = job.lang + "\n" + job.project;
        if (kept.has(key)) return routed(job.search, kept.get(key));
        stored(key).then(routing => {
            if (searching !== job.search) return;
            if (routing) {
                keep(key, routing);
                routed(job.search, routing);
                return;
            }
            return built(job).then(text => {
                if (searching !== job.search) return;
                if (!worker) return routed(job.search, "");
                jobs.set(job.search, key);
                worker.postMessage({ ...job, prebuilt: text });
            });
        });
    }

    // The answer of the worker: kept for the next time, then handed to the studio.
    function found(number, routing) {
        const key = jobs.get(number);
        jobs.delete(number);
        if (key && routing) {
            keep(key, routing);
            if (store) address(key).then(a => store.put(a, new Response(routing))).catch(() => {});
        }
        routed(number, routing);
    }

    function routed(number, routing) {
        if (number === searching) searching = null;
        try {
            call("routed", `${number}\n${routing}`);
        } catch (e) { /* a canvas is drawing: the studio asks again */ }
        poll();
    }

    // Opens an example. Without the worker its analysis keeps the browser busy for a
    // moment, so the canvases show the loading animation first: it is drawn before
    // the work starts and goes on turning meanwhile.
    function open(example) {
        for (const stage of $$(".stage")) stage.classList.add("loading");
        let opened = false;
        opening += 1;
        const go = () => {
            if (opened) return;
            opened = true;
            opening -= 1;
            act("open", example);
        };
        requestAnimationFrame(() => requestAnimationFrame(go));
        // A page that is not drawn (a tab in the background) gets no frames.
        setTimeout(go, 250);
    }

    // ---- examples, reports, exports, languages ---------------------------------------

    function loadExamples() {
        const examples = call("examples");
        $("#fact-examples").textContent = examples.length;
        const chip = example => {
            const b = element("button", "", example.title);
            b.dataset.example = example.key;
            b.title = example.description;
            b.addEventListener("click", () => open(example.key));
            return b;
        };
        $("#examples").replaceChildren(...examples.map(chip));
        $("#layout-examples").replaceChildren(...examples.filter(e => e.layout).map(chip));
    }

    function loadReports() {
        reports = call("reports");
        if (!reports.some(r => r.name === report)) report = (reports.find(r => r.name === "bom") || reports[0])?.name || "";
        listReports();
        if (state && !state.loading) showReport();
    }

    function listReports() {
        const words = $("#report-filter").value.toLowerCase().split(/\s+/).filter(Boolean);
        const shown = reports.filter(r => words.every(w => (r.title + " " + r.name).toLowerCase().includes(w)));
        $("#report-list").replaceChildren(...shown.map(r => {
            const li = element("li", r.name === report ? "on" : "");
            li.append(element("span", "", r.title));
            li.addEventListener("click", () => {
                report = r.name;
                listReports();
                showReport();
            });
            return li;
        }));
    }

    function showReport() {
        let table;
        try {
            table = call("report", report);
        } catch (e) {
            $("#report-title").textContent = e.message;
            $("#report-table").replaceChildren();
            return;
        }
        if (!table || table.busy) return;
        $("#report-title").textContent = table.title;
        const head = element("tr");
        head.append(...table.headers.map(h => element("th", "", h)));
        const rows = table.rows.map(row => {
            const tr = element("tr");
            tr.append(...row.map(cell => element("td", "", cell)));
            return tr;
        });
        $("#report-table").replaceChildren(head, ...rows);
        $("#report-more").textContent = table.total > table.rows.length ? fill(t("The first {} of {} rows"), table.rows.length, table.total) : "";
    }

    function loadExports() {
        exporters = call("exporters");
        const groups = new Map();
        for (const x of exporters) groups.set(x.group, [...(groups.get(x.group) || []), x]);
        $("#exports").replaceChildren(...[...groups].map(([title, list]) => {
            const group = element("div", "export-group");
            const buttons = element("div", "chips");
            buttons.append(...list.map(x => {
                const b = element("button", "", x.title);
                b.append(element("small", "", "." + x.extension));
                b.addEventListener("click", () => download(x));
                return b;
            }));
            group.append(element("h4", "", title), buttons);
            return group;
        }));
    }

    function download(exporter) {
        try {
            const url = URL.createObjectURL(new Blob([api.export_file(exporter.name)]));
            const a = element("a");
            a.href = url;
            a.download = `${state.example}.${exporter.extension}`;
            a.click();
            setTimeout(() => URL.revokeObjectURL(url), 10000);
        } catch (e) {
            alert(typeof e === "string" ? e : e.message);
        }
    }

    function loadLanguages() {
        const languages = call("languages");
        $("#language").hidden = false;
        $("#language").replaceChildren(...languages.map(l => {
            const o = element("option", "", l.name);
            o.value = l.code;
            return o;
        }));
        $("#language-chips").replaceChildren(...languages.map(l => {
            const b = element("button", "", l.name);
            b.dataset.language = l.code;
            b.addEventListener("click", () => setLanguage(l.code));
            return b;
        }));
    }

    function setLanguage(code) {
        try {
            call("language", code);
        } catch (e) {
            return;
        }
        localized(code);
        poll();
    }

    // Everything that is written in the language of the studio.
    function localized(code) {
        texts = call("texts");
        document.documentElement.lang = code;
        $("#language").value = code;
        for (const b of $$("[data-language]")) b.classList.toggle("on", b.dataset.language === code);
        translate();
        loadExamples();
        loadReports();
        loadExports();
        commands = call("commands");
        $("#fact-commands").textContent = commands.length;
        listCommands();
        stateText = "";
    }

    // ---- the console ---------------------------------------------------------------------

    function print(text, className) {
        const out = $("#console-out");
        out.append(element("div", className || "", text.length > CONSOLE_LIMIT ? text.slice(0, CONSOLE_LIMIT) + "\n…" : text));
        out.scrollTop = out.scrollHeight;
    }

    function help(name) {
        if (!name) {
            print(commands.map(c => c.name).join("  "), "dim");
            print(fill(t("{} commands. Type help and a name for its parameters, for example: help rename_device"), commands.length));
            return;
        }
        const command = commands.find(c => c.name === name);
        if (!command) {
            print(fill(t("Unknown command: {}"), name), "err");
            return;
        }
        print(`${command.name}\n${command.help}`);
        for (const p of command.params) print(`  --${p.name} ${p.hint}`.padEnd(30) + (p.required ? "" : t("(optional)") + " ") + p.help, "dim");
    }

    function run(line) {
        line = line.trim();
        if (!line) return;
        history.push(line);
        historyAt = history.length;
        print("> " + line, "cmd");
        const [first, second] = line.split(/\s+/);
        if (first === "help") {
            help(second);
            return;
        }
        try {
            print(call("run", line));
        } catch (e) {
            print(e.message, "err");
        }
        poll();
    }

    function listCommands() {
        const words = $("#command-filter").value.toLowerCase().split(/\s+/).filter(Boolean);
        const shown = commands.filter(c => words.every(w => (c.name + " " + c.help).toLowerCase().includes(w))).slice(0, COMMAND_ROWS);
        $("#command-list").replaceChildren(...shown.map(c => {
            const li = element("li");
            li.append(element("b", "", c.name), element("span", "", c.help));
            li.title = c.help;
            li.addEventListener("click", () => {
                $("#console-in").value = c.name + " ";
                $("#console-in").focus();
                print("> help " + c.name, "cmd");
                help(c.name);
            });
            return li;
        }));
    }

    // ---- start ---------------------------------------------------------------------------

    function controls() {
        for (const b of $$("[data-tool]")) b.addEventListener("click", () => {
            act("tool", b.dataset.tool);
            activate($('.stage[data-view="schematic"]'));
        });
        for (const b of $$("[data-view-mode]")) b.addEventListener("click", () => act("view", b.dataset.viewMode));
        for (const b of $$("[data-call]")) b.addEventListener("click", () => act(b.dataset.call));
        for (const b of $$("[data-line]")) b.addEventListener("click", () => run(b.dataset.line));
        $("#language").addEventListener("change", e => setLanguage(e.target.value));
        $("#place").addEventListener("change", e => {
            act("place", e.target.value);
            activate($('.stage[data-view="layout"]'));
        });
        $("#report-filter").addEventListener("input", listReports);
        $("#command-filter").addEventListener("input", listCommands);
        $("#console-form").addEventListener("submit", e => {
            e.preventDefault();
            run($("#console-in").value);
            $("#console-in").value = "";
        });
        $("#console-in").addEventListener("keydown", e => {
            if (e.key !== "ArrowUp" && e.key !== "ArrowDown") return;
            e.preventDefault();
            historyAt = Math.max(0, Math.min(history.length, historyAt + (e.key === "ArrowUp" ? -1 : 1)));
            e.target.value = history[historyAt] || "";
        });
    }

    // The module is 40 MB: the page is shown first. It is loaded when the visitor
    // starts to use the page, and otherwise once the page is there and the browser has
    // time, unless the visitor asked the browser to save data.
    function load() {
        loading ||= import(moduleUrl).then(async module => {
            await module.default({ module_or_path: wasmUrl });
            api = module;
            boot();
        }).catch(() => {
            for (const veil of $$(".veil span")) veil.textContent = t("The demo could not start in this browser. It needs WebGL and WebAssembly.");
        });
        return loading;
    }

    function whenNeeded() {
        for (const stage of $$(".stage")) $(".veil", stage).addEventListener("click", () => wanted = stage, { once: true });
        if (!moduleUrl) return;
        const uses = ["scroll", "pointerdown", "keydown", "touchstart"];
        const start = () => {
            for (const use of uses) removeEventListener(use, start);
            load();
        };
        for (const use of uses) addEventListener(use, start, { passive: true });
        if (location.hash) return start();
        if (navigator.connection && navigator.connection.saveData) return;
        const idle = () => window.requestIdleCallback ? requestIdleCallback(start, { timeout: 3000 }) : setTimeout(start, 1500);
        if (document.readyState === "complete") idle();
        else addEventListener("load", idle, { once: true });
    }

    function boot() {
        loadLanguages();
        controls();
        stages();
        startWorker();
        poll();
        // A page written in a language shows it; the English one follows the browser.
        let lang = state ? state.lang : "en";
        if (pageLang !== "en" && pageLang !== lang) {
            try {
                call("language", pageLang);
                lang = pageLang;
            } catch (e) { /* a language the module does not have */ }
        }
        localized(lang);
        if (wanted) activate(wanted);
        print(t("These are the commands of the command line and the MCP server, run on the project of the demo above."), "dim");
        print(t("Type help for the commands, or pick one on the right."), "dim");
        poll();
        setInterval(poll, 300);
    }

    links();
    legal();
    reveal();
    whenNeeded();
})();
