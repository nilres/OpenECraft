"use strict";
/// The inspection and test report of the open project (project.js), made for a tablet
/// or a phone on site: a card per circuit with what the schematic gives (missing data
/// marked as such) and the fields of the results, saved circuit by circuit without
/// opening the project. Values typed but not saved stay in the browser.

/// The answer of `/inspection` for the open project.
let sheet = null;
let circuitFilter = "";
/// The circuit whose card is open.
let openCircuit = null;
/// Values typed but not saved by `project|circuit`: `{ field: value }`.
let drafts = load("inspectionDrafts") || {};

const FIELDS = [
  { key: "continuity", title: "Protective conductor (Ω)", measure: true },
  { key: "insulation", title: "Insulation resistance (MΩ)", measure: true },
  { key: "loop_impedance", title: "Loop impedance Zs (Ω)", measure: true },
  { key: "rcd_current", title: "RCD trip current (mA)", measure: true, rcd: true },
  { key: "rcd_time", title: "RCD trip time (ms)", measure: true, rcd: true },
  { key: "function", title: "Function test" },
  { key: "tester", title: "Tested by" },
  { key: "date", title: "Date" },
  { key: "instrument", title: "Test instrument" },
  { key: "remark", title: "Remark", wide: true },
];
const VERDICTS = { failed: "Not passed", incomplete: "Incomplete", not_tested: "Not tested", passed: "Passed" };

/// Reviewers and editors enter results.
function mayRecord() { return !!sheet && ["review", "edit_pages", "edit_project"].some(r => sheet.rights.includes(r)); }
function draftKey(circuit) { return sheet.name + "|" + circuit; }
function keepDrafts() { save("inspectionDrafts", drafts); }
function stored(c, key) { return c.result[key] || ""; }
/// The number of a measured value, as `inspection::number` reads it (`> 500`, `0,35`).
function measured(text) {
  const m = String(text || "").trim().replace(/^[<>=]+\s*/, "").match(/^[0-9]+(?:[.,][0-9]*)?/);
  return m ? parseFloat(m[0].replace(",", ".")) : null;
}
function num(v) { return Number(v).toLocaleString(T.lang, { maximumFractionDigits: 2 }); }
function missing() { return '<span class="missing">' + esc(t("missing")) + "</span>"; }
function today() { const d = new Date(); return new Date(d.getTime() - d.getTimezoneOffset() * 60000).toISOString().slice(0, 10); }

/// The limits of a measured value in a circuit, `[lowest, highest]` (null: none), as
/// the server checks them.
function limits(c, key) {
  if (key === "insulation") return [sheet.min_insulation, null];
  if (key === "loop_impedance") return [null, c.zs_max ?? null];
  if (key === "rcd_time") return [null, sheet.max_rcd_time];
  if (key === "rcd_current" && c.rcd && c.rcd.rated != null) return [c.rcd.rated / 2, c.rcd.rated];
  return [null, null];
}
function limitText(c, key) {
  const [lo, hi] = limits(c, key);
  return lo !== null && hi !== null ? num(lo) + " – " + num(hi) : lo !== null ? "≥ " + num(lo) : hi !== null ? "≤ " + num(hi) : "";
}
function bad(c, key, value) {
  if (key === "function") return value === "failed";
  const v = measured(value);
  const [lo, hi] = limits(c, key);
  return v !== null && ((lo !== null && v < lo) || (hi !== null && v > hi));
}

async function showInspection() {
  const box = document.getElementById("inspection");
  if (!box) return;
  if (!sheet || sheet.name.toLowerCase() !== shown.name.toLowerCase()) box.innerHTML = '<div class="status">' + esc(t("Loading…")) + "</div>";
  let answer;
  try { answer = await api(projectPath(shown.name) + "/inspection"); }
  catch (e) { box.innerHTML = '<div class="status error">' + esc(e.message) + "</div>"; return; }
  sheet = answer;
  if (view === "project" && shown.tab === "inspection") drawInspection();
}

function drawInspection() {
  const box = document.getElementById("inspection");
  if (!box || !sheet) return;
  const count = {};
  for (const c of sheet.circuits) count[c.verdict] = (count[c.verdict] || 0) + 1;
  const instruments = sheet.circuits.map(c => stored(c, "instrument")).concat([load("inspectionInstrument")]);
  let html = '<div class="toolbar"><div class="summary"><strong>' + tf("{} circuits", sheet.circuits.length) + "</strong>" +
    Object.keys(VERDICTS).filter(v => count[v]).map(v => '<span class="badge ' + v + '">' + esc(t(VERDICTS[v])) + " " + count[v] + "</span>").join("") +
    '<span class="muted">U0 ' + esc(Math.round(sheet.u0)) + " V · " + esc(sheet.earthing) + "</span></div>" +
    '<div class="filter"><input type="search" id="circuit-filter" placeholder="' + esc(t("Filter circuits…")) + '" aria-label="' + esc(t("Filter circuits…")) + '" value="' + esc(circuitFilter) + '">' +
    '<button id="excel" title="' + esc(t("Download the report as an Excel workbook")) + '">' + esc(t("Excel")) + '</button><span id="excel-msg"></span></div></div>';
  if (!mayRecord()) html += '<p class="notice">' + esc(t("You may read the report; entering results needs the right to save reviews.")) + "</p>";
  html += datalist("instruments", instruments) + '<div id="circuits"></div>';
  box.innerHTML = html;
  document.getElementById("circuit-filter").oninput = e => { circuitFilter = e.target.value; drawCircuits(); };
  document.getElementById("excel").onclick = downloadExcel;
  drawCircuits();
}

function drawCircuits() {
  const box = document.getElementById("circuits");
  if (!box) return;
  const f = circuitFilter.trim().toLowerCase();
  const list = sheet.circuits.filter(c => !f || c.circuit.toLowerCase().includes(f) || (c.description || "").toLowerCase().includes(f));
  box.innerHTML = list.map(card).join("") || '<p class="muted">' + esc(sheet.circuits.length ? t("No circuits found") : t("The project has no circuits fed from the mains")) + "</p>";
  box.querySelectorAll("details.circuit").forEach(d => {
    d.ontoggle = () => {
      if (d.open) {
        openCircuit = d.dataset.circuit;
        box.querySelectorAll("details.circuit[open]").forEach(o => { if (o !== d) o.open = false; });
      } else if (openCircuit === d.dataset.circuit) openCircuit = null;
    };
    wire(d);
  });
}

/// The value a field shows: typed, saved, otherwise for a new record who tests, today
/// and the instrument used last.
function shownValue(c, key) {
  const draft = drafts[draftKey(c.circuit)] || {};
  if (key in draft) return draft[key];
  const v = stored(c, key);
  if (v || !mayRecord()) return v;
  if (key === "tester") return session ? session.user.name : "";
  if (key === "date") return today();
  if (key === "instrument") return load("inspectionInstrument") || "";
  return "";
}

function card(c) {
  const draft = drafts[draftKey(c.circuit)];
  const device = (c.characteristic ? esc(c.characteristic) : missing()) + " " + (c.rated != null ? esc(num(c.rated)) + " A" : missing());
  const rcd = c.rcd ? esc(c.rcd.device) + " · " + (c.rcd.kind ? esc(c.rcd.kind) : missing()) + " · " + (c.rcd.rated != null ? esc(num(c.rcd.rated)) + " mA" : missing()) : "–";
  const facts = [
    ["Phases", esc(c.phases.join(" "))],
    ["Cable", c.cables.length ? esc(c.cables.join(", ")) : "–"],
    ["Cross-section", c.cross_section ? esc(c.cross_section) : missing()],
    ["Protective device", device],
    ["RCD", rcd],
    ["Zs max (Ω)", c.zs_max != null ? esc(num(c.zs_max)) : missing()],
  ];
  let inputs = "";
  for (const fd of FIELDS) {
    if (fd.rcd && !c.rcd) continue;
    const v = shownValue(c, fd.key);
    let input;
    if (fd.key === "function") {
      input = '<select data-field="function">' + [["", "–"], ["ok", t("OK")], ["failed", t("Not OK")]].map(([k, l]) => '<option value="' + k + '"' + (k === v ? " selected" : "") + ">" + esc(l) + "</option>").join("") + "</select>";
    } else if (fd.key === "remark") {
      input = '<textarea rows="2" data-field="remark">' + esc(v) + "</textarea>";
    } else if (fd.key === "date") {
      input = '<input type="date" data-field="date" value="' + esc(v) + '">';
    } else {
      const extra = fd.measure ? ' inputmode="decimal" autocomplete="off" enterkeyhint="next"' : fd.key === "instrument" ? ' list="instruments"' : "";
      input = '<input data-field="' + fd.key + '"' + extra + ' value="' + esc(v) + '">';
    }
    const limit = fd.measure ? limitText(c, fd.key) : "";
    inputs += '<label class="' + (fd.wide ? "wide" : "") + (bad(c, fd.key, v) ? " bad" : "") + '"><span>' + esc(t(fd.title)) + (limit ? ' <span class="limit">' + esc(limit) + "</span>" : "") + "</span>" + input + "</label>";
  }
  const buttons = mayRecord() ? '<div class="buttons"><button class="primary" data-save>' + esc(t("Save")) + "</button><button data-next>" + esc(t("Next circuit")) + '</button><span class="msg"></span></div>' : "";
  return '<details class="circuit" data-circuit="' + esc(c.circuit) + '"' + (openCircuit === c.circuit ? " open" : "") + "><summary><strong>" + esc(c.circuit) + '</strong><span class="desc">' +
    (c.description ? esc(c.description) : missing()) + "</span>" + (draft ? '<span class="badge incomplete">' + esc(t("Not saved")) + "</span>" : "") +
    '<span class="badge ' + c.verdict + '">' + esc(t(VERDICTS[c.verdict])) + '</span></summary><div class="body"><dl>' + facts.map(([k, v]) => "<dt>" + esc(t(k)) + "</dt><dd>" + v + "</dd>").join("") +
    '</dl><fieldset class="measure"' + (mayRecord() ? "" : " disabled") + ">" + inputs + "</fieldset>" + buttons + "</div></details>";
}

function wire(d) {
  const c = sheet.circuits.find(x => x.circuit === d.dataset.circuit);
  d.querySelectorAll("[data-field]").forEach(input => {
    input.oninput = input.onchange = () => {
      const k = draftKey(c.circuit);
      const draft = drafts[k] || {};
      if (input.value === stored(c, input.dataset.field)) delete draft[input.dataset.field]; else draft[input.dataset.field] = input.value;
      if (Object.keys(draft).length) drafts[k] = draft; else delete drafts[k];
      keepDrafts();
      input.closest("label").classList.toggle("bad", bad(c, input.dataset.field, input.value));
    };
  });
  const saveButton = d.querySelector("[data-save]");
  if (saveButton) saveButton.onclick = () => saveCircuit(c, d);
  const next = d.querySelector("[data-next]");
  if (next) next.onclick = () => {
    let n = d.nextElementSibling;
    while (n && !n.matches("details.circuit")) n = n.nextElementSibling;
    if (!n) return;
    n.open = true;
    n.scrollIntoView({ block: "start", behavior: "smooth" });
  };
}

async function saveCircuit(c, d) {
  const values = {};
  d.querySelectorAll("[data-field]").forEach(input => { if (input.value !== stored(c, input.dataset.field)) values[input.dataset.field] = input.value; });
  const msg = text => { const m = document.querySelector('details.circuit[open] .msg'); if (m) { m.className = "msg " + (text === t("Saved") ? "ok" : "error"); m.textContent = text; } };
  if (!Object.keys(values).length) { msg(t("Nothing to save")); return; }
  const button = d.querySelector("[data-save]");
  button.disabled = true;
  const user = values.tester || stored(c, "tester") || (session ? session.user.name : "");
  try {
    sheet = await api(projectPath(sheet.name) + "/inspection", { method: "PUT", json: { user, circuit: c.circuit, rev: c.result.rev || 0, values } });
  } catch (e) {
    button.disabled = false;
    if (e.status !== 409) { msg(e.message); return; }
    // Saved by someone else meanwhile: the values typed here stay, over theirs.
    for (const [k, v] of Object.entries(values)) (drafts[draftKey(c.circuit)] ||= {})[k] = v;
    keepDrafts();
    await showInspection();
    msg(t("Someone saved results of this circuit meanwhile. Check the values and save again."));
    return;
  }
  // The new revision is this save: the look at the project every 20 s draws nothing anew.
  if (project && project.name === sheet.name) project.server_revision = sheet.revision;
  delete drafts[draftKey(c.circuit)];
  keepDrafts();
  if (values.instrument) save("inspectionInstrument", values.instrument);
  openCircuit = c.circuit;
  drawInspection();
  msg(t("Saved"));
}

async function downloadExcel() {
  const m = document.getElementById("excel-msg");
  try {
    const blob = await api(projectPath(sheet.name) + "/inspection.xlsx?lang=" + encodeURIComponent(T.lang), { raw: true });
    const a = document.createElement("a");
    a.href = URL.createObjectURL(blob);
    a.download = sheet.name.replace(/[^A-Za-z0-9_-]/g, "_") + "-inspection.xlsx";
    document.body.appendChild(a);
    a.click();
    a.remove();
    setTimeout(() => URL.revokeObjectURL(a.href), 10000);
    if (m) m.textContent = "";
  } catch (e) { if (m) { m.className = "error"; m.textContent = e.message; } }
}
