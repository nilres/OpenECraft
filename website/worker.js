// The worker of the OpenECraft website: a second instance of the module searches the
// 3D routing of a project, which takes seconds for a big one, so that the page can
// be scrolled and used meanwhile. The page (site.js) sends the module once, then
// projects, and takes the routing of each back.
"use strict";

let api;

onmessage = async event => {
    const job = event.data;
    if (job.module) {
        try {
            api = await import(job.module);
            await api.default({ module_or_path: job.wasm });
            postMessage({ ready: true });
        } catch (error) {
            postMessage({ failed: true });
        }
        return;
    }
    let routing = "";
    try {
        routing = api.search_routing(job.project, job.lang, job.prebuilt || "");
    } catch (error) { /* without a routing the page searches it itself */ }
    postMessage({ search: job.search, routing });
};
