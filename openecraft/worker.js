// The worker of the web editor: a second instance of the module searches the 3D
// routing of a project being opened, which takes seconds for a big one and would
// stop the page (a page has one thread). The editor (openecraft_ui::io::RoutingWorker)
// sends the module once, then projects, and takes the routing of each back. An
// unchanged example takes its routing from the build of the page (routing/) if it
// fits the wiring of the project.
"use strict";

let loading = null;
let prebuilt = null;

async function start(job) {
    const api = await import(job.module);
    await api.default({ module_or_path: job.wasm });
    return api;
}

// The routing of an example from the build of the page: the one of its language or
// the one for every language (index.json lists them; a debug build has none).
async function built(job) {
    if (!job.example) return "";
    try {
        prebuilt ||= fetch("routing/index.json").then(answer => answer.ok ? answer.json() : []).catch(() => []);
        const files = await prebuilt;
        const file = [`${job.example}.${job.lang}.json`, `${job.example}.json`].find(f => Array.isArray(files) && files.includes(f));
        if (!file) return "";
        const answer = await fetch("routing/" + file);
        return answer.ok ? await answer.text() : "";
    } catch (error) {
        return "";
    }
}

// The texts of the language of a search: the module has the catalogs of English,
// German, French and Spanish only and is handed the one of another language.
async function language(api, lang) {
    if (api.language_loaded(lang)) return;
    try {
        const answer = await fetch(`i18n/${lang}.tsv`);
        if (answer.ok) api.install_language(lang, await answer.text());
    } catch (error) { /* searched in English */ }
}

onmessage = async event => {
    const job = event.data;
    if (job.module) {
        loading = start(job);
        loading.catch(() => postMessage({ failed: true }));
        return;
    }
    let routing = "";
    try {
        const api = await loading;
        await language(api, job.lang);
        routing = api.search_routing(job.project, job.lang, await built(job));
    } catch (error) { /* without a routing the editor searches it itself */ }
    postMessage({ search: job.search, routing });
};
