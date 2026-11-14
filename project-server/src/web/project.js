"use strict";
/// The web page of the project server, on top of common.js: the projects the user may
/// see and each of them read only (its pages by section or by structure, its devices,
/// the pages drawn as on paper). The administration is in admin.js.
let view = "projects";
/// The open project: its name, the page shown and the navigator tab.
let shown = { name: null, page: null, tab: "pages" };
/// The answer of `/view` for the open project.
let project = null;
let serverInfo = null;
let filter = "";
let deviceFilter = "";
let byStructure = load("byStructure") === true;
/// Zoom of the page: 0 fits the width, otherwise the scale of the paper's width.
let zoom = 0;
/// Pages drawn so far (object addresses) by `name|revision|page`.
let drawn = {};
/// Seconds between two looks at the open project (locks, new revisions).
const REFRESH = 20;

// Hooks of common.js
function adminAllowed() { return may("administer", "*"); }
async function signedIn() {}
function signedOut(left) {
  // Without sign-in the page shows what anonymous requests may do.
  footer().then(renderAccount);
  if (left) { if (view === "admin") view = "projects"; go(); }
}

function projectPath(name) { return "/projects/" + encodeURIComponent(name); }
function rightTitle(name) { const r = (T.rights || []).find(x => x.name === name); return r ? r.title : name; }
/// What the caller may do on a project: the highest built-in role all of whose rights
/// they have, the rights themselves in its tooltip (each right without such a role).
function rightsBadges(rights) {
  const has = new Set(rights || []);
  const role = (T.roles || []).filter(r => r.rights.every(x => has.has(x))).pop();
  if (!role) return (rights || []).map(r => '<span class="badge">' + esc(rightTitle(r)) + "</span>").join("");
  return '<span class="badge" title="' + esc((rights || []).map(rightTitle).join(", ")) + '">' + esc(role.title) + "</span>";
}

/// The view of the address: `#project=<name>&page=<id>&tab=devices`, `#admin/<tab>` or the list.
function stateHash() {
  if (view === "admin") return "admin/" + adminTab;
  if (view !== "project") return "";
  const p = new URLSearchParams();
  p.set("project", shown.name);
  if (shown.page !== null) p.set("page", String(shown.page));
  if (shown.tab !== "pages") p.set("tab", shown.tab);
  return p.toString();
}
function readHash() {
  const h = location.hash.replace(/^#/, "");
  if (h === "admin" || h.startsWith("admin/")) { view = "admin"; adminTab = h.slice(6) || "overview"; return; }
  const p = new URLSearchParams(h);
  if (!p.get("project")) { view = "projects"; return; }
  view = "project";
  const name = p.get("project");
  if (name !== shown.name) { project = null; zoom = 0; }
  const page = parseInt(p.get("page") || "", 10);
  shown = { name, page: Number.isFinite(page) ? page : null, tab: p.get("tab") === "devices" ? "devices" : "pages" };
}
function go() {
  const h = stateHash();
  if (location.hash.replace(/^#/, "") === h) route(); else location.hash = h;
}
function setNav() {
  document.getElementById("nav-projects").classList.toggle("active", view !== "admin");
  document.getElementById("nav-admin").classList.toggle("active", view === "admin");
}

function route() {
  readHash();
  setNav();
  if (view === "admin") showAdmin(); else if (view === "project") showProject(); else showProjects();
}

// The list of projects

async function showProjects() {
  let list;
  try { list = await api("/projects"); }
  catch (e) { content('<div class="status error">' + esc(e.status === 401 ? t("Sign in to see the projects") : e.message) + "</div>"); return; }
  if (view !== "projects") return;
  list.sort((a, b) => a.name.localeCompare(b.name));
  const draw = () => {
    const f = filter.trim().toLowerCase();
    const rows = list.filter(p => !f || p.name.toLowerCase().includes(f) || (p.title || "").toLowerCase().includes(f));
    let html = '<table><thead><tr><th>' + esc(t("Project")) + '</th><th class="hide-narrow">' + esc(t("Description")) + '</th><th class="num">' + esc(t("Pages")) + '</th><th class="num hide-narrow">' + esc(t("Revision")) + "</th><th>" +
      esc(t("In use by")) + '</th><th class="hide-narrow">' + esc(t("Your rights")) + "</th></tr></thead><tbody>";
    for (const p of rows) {
      html += '<tr data-name="' + esc(p.name) + '"><td><a href="#' + esc(new URLSearchParams({ project: p.name }).toString()) + '"><strong>' + esc(p.name) + "</strong></a></td>" +
        '<td class="hide-narrow">' + esc(p.title && p.title !== p.name ? p.title : "") + '</td><td class="num">' + p.pages + '</td><td class="num hide-narrow">' + p.revision + "</td><td>" +
        (p.users.length ? '<span class="badge locked">' + esc(p.users.join(", ")) + "</span>" : "") + '</td><td class="hide-narrow"><div class="rights">' +
        rightsBadges(p.rights) + "</div></td></tr>";
    }
    if (!rows.length) html += '<tr><td colspan="6" class="muted">' + esc(list.length ? t("No projects found") : session ? t("You may not see any project") : t("Sign in to see the projects")) + "</td></tr>";
    document.getElementById("project-list").innerHTML = html + "</tbody></table>";
    document.querySelectorAll("tr[data-name]").forEach(r => r.onclick = e => { if (e.target.closest("a")) return; openProject(r.dataset.name); });
  };
  content('<div class="toolbar"><strong>' + tf("{} projects", list.length) + '</strong><input id="filter" type="search" placeholder="' + esc(t("Filter projects…")) + '" aria-label="' + esc(t("Filter projects…")) + '" value="' + esc(filter) +
    '"></div><div class="table-wrap" id="project-list"></div>');
  document.getElementById("filter").oninput = e => { filter = e.target.value; draw(); };
  draw();
}

function openProject(name, page) {
  view = "project";
  if (name !== shown.name) { project = null; zoom = 0; }
  shown = { name, page: page ?? null, tab: shown.name === name ? shown.tab : "pages" };
  go();
}

// One project, read only

async function loadProject() {
  const r = await api(projectPath(shown.name) + "/view?lang=" + encodeURIComponent(T.lang));
  if (project && project.server_revision !== r.server_revision) forget(project.name);
  project = r;
  return r;
}

/// Drops the drawn pages of a project (it changed).
function forget(name) {
  for (const k of Object.keys(drawn)) if (k.startsWith(name + "|")) { URL.revokeObjectURL(drawn[k]); delete drawn[k]; }
}

async function showProject() {
  if (!project || project.name.toLowerCase() !== shown.name.toLowerCase()) {
    content('<div class="status">' + esc(t("Loading…")) + "</div>");
    try { await loadProject(); }
    catch (e) { content('<div class="status error">' + esc(e.status === 404 ? tf("Project {} not found", shown.name) : e.message) + "</div>"); return; }
  }
  if (view !== "project") return;
  const p = project;
  if (shown.page === null || !p.pages.some(x => x.id === shown.page)) shown.page = p.pages.length ? p.pages[0].id : null;
  const info = [["Description", p.description], ["Company", p.company], ["Created by", p.creator], ["Date", p.date], ["Revision", p.revision]].filter(([, v]) => v);
  let html = '<div class="project"><div class="head"><h2>' + esc(p.title || p.name) + "</h2>" + '<span class="badge readonly">' + esc(t("Read only")) + "</span>" +
    '<span class="muted">' + esc(p.name) + " · " + tf("Revision {} on the server", p.server_revision) + '</span><span class="rights">' + rightsBadges(p.rights) + "</span>" +
    (p.users.length ? '<span class="badge locked">' + tf("In use by {}", p.users.join(", ")) + "</span>" : "") +
    (p.rights.includes("manage_access") ? '<button class="small" id="permissions">' + esc(t("Permissions")) + "</button>" : "") +
    '<a class="small" href="#">' + esc(t("All projects")) + "</a></div>";
  if (info.length) html += '<dl class="info">' + info.map(([k, v]) => "<dt>" + esc(t(k)) + "</dt><dd>" + esc(v) + "</dd>").join("") + "</dl>";
  html += '<div id="grants"></div><div class="viewer"><aside class="nav" id="navigator"></aside><section class="sheet" id="sheet"></section></div></div>';
  content(html);
  const perm = document.getElementById("permissions");
  if (perm) perm.onclick = () => showGrants(p.name);
  drawNavigator();
  drawSheet();
}

/// The locks on a page (or on the project data): who holds them.
function lockOf(target) { return project.locks.find(l => target === "project" ? l.target === "project" : l.target && l.target.page === target); }

function pageRow(pg, name) {
  const lock = lockOf(pg.id);
  return '<a class="row' + (pg.id === shown.page ? " sel" : "") + '" data-page="' + pg.id + '" href="#' + esc(new URLSearchParams({ project: project.name, page: pg.id }).toString()) + '">' +
    "<span>" + esc(pg.icon) + "</span><span>" + esc(name) + '</span><span class="desc">' + esc(pg.description) + "</span>" +
    (lock ? '<span class="lock" title="' + esc(tp("Locked by {}", lock.user)) + '">🔒 ' + esc(lock.user) + "</span>" : "") + "</a>";
}

function drawNavigator() {
  const box = document.getElementById("navigator");
  if (!box) return;
  const p = project;
  let html = '<div class="tabs"><button data-nav="pages"' + (shown.tab === "pages" ? ' class="active"' : "") + ">" + esc(t("Pages")) + '</button><button data-nav="devices"' +
    (shown.tab === "devices" ? ' class="active"' : "") + ">" + esc(t("Devices")) + "</button></div>";
  if (shown.tab === "pages") {
    html += '<label class="check"><input type="checkbox" id="by-structure"' + (byStructure ? " checked" : "") + "> " + esc(t("Group the pages by structure")) + "</label>";
    const byId = Object.fromEntries(p.pages.map(x => [x.id, x]));
    if (byStructure) {
      for (const g of p.structures) {
        html += "<details open><summary>" + esc(g.structure ? g.title : t("(no structure)")) + "</summary>" + g.pages.map(id => pageRow(byId[id], byId[id].name)).join("") + "</details>";
      }
    } else {
      const section = id => {
        const s = p.sections.find(x => x.id === id);
        return "<details open><summary>" + esc(s.title) + "</summary>" + p.pages.filter(x => x.section === id).map(x => pageRow(x, x.full_name)).join("") +
          p.sections.filter(x => x.parent === id).map(x => section(x.id)).join("") + "</details>";
      };
      html += p.pages.filter(x => x.section === 0).map(x => pageRow(x, x.full_name)).join("") + p.sections.filter(x => x.parent === 0).map(x => section(x.id)).join("");
    }
    if (!p.pages.length) html += '<p class="muted">' + esc(t("The project has no pages")) + "</p>";
  } else {
    html += '<input type="search" id="device-filter" placeholder="' + esc(t("Filter devices…")) + '" aria-label="' + esc(t("Filter devices…")) + '" value="' + esc(deviceFilter) + '"><div id="devices"></div>';
  }
  box.innerHTML = html;
  box.querySelectorAll("[data-nav]").forEach(b => b.onclick = () => { shown.tab = b.dataset.nav; go(); });
  const toggle = document.getElementById("by-structure");
  if (toggle) toggle.onchange = () => { byStructure = toggle.checked; save("byStructure", byStructure); drawNavigator(); };
  const devices = document.getElementById("device-filter");
  if (devices) {
    devices.oninput = () => { deviceFilter = devices.value; drawDevices(); };
    drawDevices();
  }
  const sel = box.querySelector(".row.sel");
  if (sel) sel.scrollIntoView({ block: "nearest" });
}

function drawDevices() {
  const f = deviceFilter.trim().toLowerCase();
  const structured = project.devices.some(g => g.structure);
  let html = "";
  for (const g of project.devices) {
    const list = g.devices.filter(d => !f || d.tag.toLowerCase().includes(f));
    if (!list.length) continue;
    if (structured) html += '<div class="group">' + esc(g.structure || t("(no structure)")) + "</div>";
    html += list.map(d => "<details><summary>" + esc(d.title) + '</summary><ul class="places">' +
      d.places.map(pl => '<li><a href="#' + esc(new URLSearchParams({ project: project.name, page: pl.page, tab: "devices" }).toString()) + '">' + esc(pl.text) + "</a></li>").join("") + "</ul></details>").join("");
  }
  document.getElementById("devices").innerHTML = html || '<p class="muted">' + esc(project.devices.length ? t("No devices found") : t("The project has no devices")) + "</p>";
}

function drawSheet() {
  const box = document.getElementById("sheet");
  if (!box) return;
  const pages = project.pages;
  const i = pages.findIndex(x => x.id === shown.page);
  if (i < 0) { box.innerHTML = '<div class="status">' + esc(t("The project has no pages")) + "</div>"; return; }
  const pg = pages[i];
  const lock = lockOf(pg.id);
  box.innerHTML = '<div class="toolbar"><div class="title"><strong>' + esc(pg.icon + " " + pg.full_name) + '</strong><span class="muted">' + esc(pg.description) + "</span>" +
    (lock ? '<span class="badge locked">' + tf("Locked by {}", lock.user) + "</span>" : "") + '</div><div class="buttons" style="margin:0">' +
    '<button class="small" id="prev" title="' + esc(t("Previous page")) + '"' + (i === 0 ? " disabled" : "") + ">◀</button>" +
    '<span class="muted">' + tf("Page {} of {}", i + 1, pages.length) + "</span>" +
    '<button class="small" id="next" title="' + esc(t("Next page")) + '"' + (i === pages.length - 1 ? " disabled" : "") + ">▶</button>" +
    '<button class="small" id="zoom-out" title="' + esc(t("Zoom out")) + '">−</button><button class="small" id="zoom-fit">' + esc(t("Fit")) + '</button><button class="small" id="zoom-in" title="' +
    esc(t("Zoom in")) + '">+</button></div></div><div class="paper" id="paper"><div class="status">' + esc(t("Loading…")) + "</div></div>";
  const step = d => { const next = pages[i + d]; if (next) { shown.page = next.id; go(); } };
  document.getElementById("prev").onclick = () => step(-1);
  document.getElementById("next").onclick = () => step(1);
  document.getElementById("zoom-out").onclick = () => setZoom((zoom || 1) / 1.25);
  document.getElementById("zoom-in").onclick = () => setZoom((zoom || 1) * 1.25);
  document.getElementById("zoom-fit").onclick = () => setZoom(0);
  drawPage(pg);
  pan(document.getElementById("paper"));
}

async function drawPage(pg) {
  const k = project.name + "|" + project.server_revision + "|" + pg.id;
  const paper = document.getElementById("paper");
  try {
    if (!drawn[k]) drawn[k] = URL.createObjectURL(await api(projectPath(project.name) + "/pages/" + pg.id + "/svg", { raw: true }));
  } catch (e) { if (paper) paper.innerHTML = '<div class="status">' + esc(e.message) + "</div>"; return; }
  if (!paper || shown.page !== pg.id) return;
  paper.innerHTML = '<img id="page-image" alt="' + esc(pg.full_name + " " + pg.description) + '" src="' + drawn[k] + '">';
  setZoom(zoom);
}

function setZoom(z) {
  zoom = z > 0 ? Math.min(8, Math.max(0.25, z)) : 0;
  const img = document.getElementById("page-image");
  if (img) img.style.width = zoom ? zoom * 100 + "%" : "100%";
}

/// Moves the paper by dragging it; Ctrl and the wheel zoom.
function pan(paper) {
  let from = null;
  paper.onpointerdown = e => { from = { x: e.clientX, y: e.clientY, left: paper.scrollLeft, top: paper.scrollTop }; paper.setPointerCapture(e.pointerId); paper.classList.add("dragging"); };
  paper.onpointermove = e => { if (from) { paper.scrollLeft = from.left - (e.clientX - from.x); paper.scrollTop = from.top - (e.clientY - from.y); } };
  paper.onpointerup = paper.onpointercancel = () => { from = null; paper.classList.remove("dragging"); };
  paper.onwheel = e => { if (!e.ctrlKey) return; e.preventDefault(); setZoom((zoom || 1) * (e.deltaY < 0 ? 1.15 : 1 / 1.15)); };
}

async function showGrants(name) {
  const box = document.getElementById("grants");
  let r;
  try { r = await api(projectPath(name) + "/access"); }
  catch (e) { box.innerHTML = '<div class="status error">' + esc(e.message) + "</div>"; return; }
  scopeGrants(box, r.grants, {
    title: tf("Permissions of project {}", name),
    help: t("The roles given here apply to this project; deny takes the rights of the role away. Only administrators lift a denial."),
    empty: t("No grants: the roles of the users apply"),
    roles: r.roles,
    role: r.roles[0] || "",
    scope: name,
    save: async list => (await api(projectPath(name) + "/access", { method: "PUT", json: list })).grants,
  });
}

/// Looks at the open project now and then: new locks, users and revisions.
async function refresh() {
  if (document.hidden || view !== "project" || !project) return;
  const before = project.server_revision;
  const locks = JSON.stringify(project.locks);
  try { await loadProject(); } catch (e) { return; }
  if (view !== "project") return;
  if (project.server_revision !== before) { showProject(); return; }
  // Opened entries of the navigator stay open while nothing changed.
  if (JSON.stringify(project.locks) === locks) return;
  drawNavigator();
  const sheet = document.getElementById("sheet");
  if (sheet && !sheet.querySelector(".dragging")) {
    const lock = shown.page !== null && lockOf(shown.page);
    const title = sheet.querySelector(".title .badge.locked");
    if (!!lock !== !!title || (lock && title && !title.textContent.includes(lock.user))) drawSheet();
  }
}

document.addEventListener("keydown", e => {
  if (view !== "project" || !project || e.target.closest("input, select, textarea")) return;
  const i = project.pages.findIndex(x => x.id === shown.page);
  const next = e.key === "PageDown" || e.key === "ArrowRight" ? project.pages[i + 1] : e.key === "PageUp" || e.key === "ArrowLeft" ? project.pages[i - 1] : null;
  if (next) { e.preventDefault(); shown.page = next.id; go(); }
});

async function footer() {
  try {
    serverInfo = await api("/info");
    guest = session ? null : serverInfo.access;
    document.getElementById("footer").textContent = serverInfo.name + " " + serverInfo.version;
  } catch (e) {}
}

async function start() {
  try { await setLanguage(startLanguage()); } catch (e) {}
  await refreshSession();
  await footer();
  renderAccount();
  document.getElementById("lang").onchange = async e => { await setLanguage(e.target.value); project = null; route(); };
  document.getElementById("home").onclick = e => { e.preventDefault(); view = "projects"; go(); };
  document.getElementById("nav-projects").onclick = () => { view = "projects"; go(); };
  document.getElementById("nav-admin").onclick = () => { view = "admin"; go(); };
  window.onhashchange = route;
  setInterval(refresh, REFRESH * 1000);
  route();
}
start();
