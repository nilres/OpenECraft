// The tools of the OpenECraft website (content/tools/…): a form with values or a file
// of the visitor, handed to the module of the site (`src/tools.rs`, the code of the
// editor compiled to WebAssembly) and its answer shown. The module is loaded when the
// tool is used; nothing the visitor enters or opens leaves the browser.
"use strict";
(() => {
    const $ = (selector, root = document) => root.querySelector(selector);
    const tool = $("[data-tool]");
    const build = window.OPENECRAFT_BUILD || {};
    if (!tool) return;
    const form = $("form", tool);
    const name = tool.dataset.tool;
    const lang = document.documentElement.lang || "en";
    const show = (selector, shown) => {
        const e = $(selector, tool);
        if (e) e.hidden = !shown;
        return e;
    };
    let loading = null;
    // The file the answer shown is of: the files to save are written from it.
    let bytes = new Uint8Array();
    let fileName = "";

    function element(tag, text) {
        const e = document.createElement(tag);
        if (text !== undefined) e.textContent = text;
        return e;
    }

    // The module of the site, named by the build (build.js).
    function load() {
        const [script, wasm] = build.module || [];
        if (!script || !wasm) return Promise.reject(new Error("module"));
        loading ||= import(script).then(async module => {
            await module.default({ module_or_path: wasm });
            return module;
        });
        return loading;
    }

    function fail(message) {
        show(".tool-wait", false);
        show(".tool-out", false);
        show(".tool-error", true).textContent = message;
    }

    function facts(list) {
        const dl = $(".tool-facts", tool);
        if (!dl) return;
        dl.replaceChildren(...(list || []).flatMap(([label, value]) => [element("dt", label), element("dd", String(value))]));
    }

    function table(headers, rows) {
        const out = $(".tool-table", tool);
        if (!out) return;
        const head = element("tr");
        head.append(...(headers || []).map(h => element("th", h)));
        out.replaceChildren(head, ...(rows || []).map(row => {
            const tr = element("tr");
            tr.append(...row.map(cell => element("td", String(cell))));
            return tr;
        }));
    }

    function drawing(svg) {
        const out = $(".tool-drawing", tool);
        if (!out || !svg) return;
        const parsed = new DOMParser().parseFromString(svg, "image/svg+xml").documentElement;
        if (parsed.nodeName !== "svg") return;
        parsed.removeAttribute("width");
        parsed.removeAttribute("height");
        out.replaceChildren(parsed);
    }

    // The buttons of the files to save: each is written by the module when it is asked for.
    function files(module, names) {
        const out = $(".tool-files", tool);
        if (!out) return;
        for (const old of out.querySelectorAll("button")) old.remove();
        out.append(...(names || []).map(file => {
            const ending = file.split("_").pop();
            const b = element("button", ending.toUpperCase());
            b.className = "button";
            b.type = "button";
            b.addEventListener("click", () => {
                try {
                    const url = URL.createObjectURL(new Blob([module.tool_file(file, bytes)]));
                    const a = element("a");
                    a.href = url;
                    a.download = (fileName.replace(/\.[^.]*$/, "") || name) + "." + ending;
                    a.click();
                    setTimeout(() => URL.revokeObjectURL(url), 10000);
                } catch (e) {
                    fail(typeof e === "string" ? e : e.message);
                }
            });
            return b;
        }));
    }

    async function run() {
        const values = {};
        for (const field of form.elements) if (field.name && field.type !== "file") values[field.name] = field.value;
        const input = $('input[type="file"]', form);
        const file = input && input.files[0];
        if (input && !file) return;
        show(".tool-error", false);
        show(".tool-failed", false);
        show(".tool-wait", !loading || !tool.dataset.loaded);
        let module;
        try {
            module = await load();
            if (file) {
                bytes = new Uint8Array(await file.arrayBuffer());
                fileName = file.name;
            }
        } catch (e) {
            show(".tool-wait", false);
            show(".tool-failed", true);
            return;
        }
        tool.dataset.loaded = "1";
        show(".tool-wait", false);
        let answer;
        try {
            answer = JSON.parse(module.tool(name, JSON.stringify(values), bytes, lang));
        } catch (e) {
            return fail(typeof e === "string" ? e : e.message);
        }
        if (answer.error) return fail(answer.error);
        const summary = $(".tool-summary", tool);
        if (summary) {
            summary.textContent = answer.summary || "";
            summary.classList.toggle("ok", answer.ok === true);
            summary.classList.toggle("bad", answer.ok === false);
        }
        facts(answer.facts);
        table(answer.headers, answer.rows);
        drawing(answer.svg);
        files(module, answer.files);
        show(".tool-out", true);
    }

    form.addEventListener("submit", e => {
        e.preventDefault();
        run();
    });
    // A file is shown as soon as it is chosen; a calculator follows its values once
    // the module is there.
    form.addEventListener("change", e => {
        if (e.target.type === "file" || (tool.hasAttribute("data-live") && tool.dataset.loaded)) run();
    });
    // A file dropped on the tool.
    const input = $('input[type="file"]', form);
    if (input) {
        tool.addEventListener("dragover", e => e.preventDefault());
        tool.addEventListener("drop", e => {
            e.preventDefault();
            if (!e.dataTransfer.files.length) return;
            input.files = e.dataTransfer.files;
            run();
        });
    }
})();
