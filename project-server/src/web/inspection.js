"use strict";
/// The inspection and test report of the open project (project.js), made for a tablet
/// or a phone on site: a card with the data of the report, the visual inspection, the
/// verdict and the signature, then a card per circuit with what the schematic gives
/// (missing data marked as such) and the fields of the results. Each card is saved on
/// its own without opening the project; values typed but not saved stay in the browser.

/// The answer of `/inspection` for the open project.
let sheet = null;
let circuitFilter = "";
/// The card that is open: a circuit, `""` for the report card, `null` for none.
let openCard = null;
/// Values typed but not saved by `project|card`: `{ field: value }`.
let drafts = load("inspectionDrafts") || {};

const FIELDS = [
  { key: "continuity", title: "Protective conductor (Ω)", measure: true },
  { key: "insulation_voltage", title: "Insulation test voltage (V)", measure: true },
  { key: "insulation", title: "Insulation resistance (MΩ)", measure: true },
  { key: "loop_impedance", title: "Loop impedance Zs (Ω)", measure: true },
  { key: "rcd_current", title: "RCD trip current (mA)", measure: true, rcd: true },
  { key: "rcd_time", title: "RCD trip time (ms)", measure: true, rcd: true },
  { key: "rcd_time_5", title: "RCD trip time at 5 × IΔn (ms)", measure: true, rcd: true },
  { key: "rcd_voltage", title: "Contact voltage UB (V)", measure: true, rcd: true },
  { key: "function", title: "Function test", answer: true },
  { key: "tester", title: "Tested by" },
  { key: "date", title: "Date", date: true },
  { key: "instrument", title: "Test instrument", list: "instruments" },
  { key: "remark", title: "Remark", wide: true, area: true },
];
const HEADER = [
  { key: "client", title: "Client" },
  { key: "site", title: "Installation site" },
  { key: "installer", title: "Installer" },
  { key: "reason", title: "Reason for the test", list: "reasons" },
  { key: "inspector", title: "Inspector" },
  { key: "next_inspection", title: "Next inspection", date: true },
  { key: "instruments", title: "Test instruments (IEC 61557)", wide: true, area: true },
];
const ORIGIN = [
  { key: "ze", title: "External loop impedance Ze (Ω)", measure: true },
  { key: "ik", title: "Prospective fault current at the origin (kA)", measure: true },
  { key: "earth_electrode", title: "Earth electrode resistance RA (Ω)", measure: true },
  { key: "bonding", title: "Main equipotential bonding (Ω)", measure: true },
];
const REASONS = ["New installation", "Extension", "Change", "Repair"];
const VERDICTS = { failed: "Not passed", incomplete: "Incomplete", not_tested: "Not tested", passed: "Passed" };

/// Reviewers and editors enter results.
function mayRecord() { return !!sheet && ["review", "edit_pages", "edit_project"].some(r => sheet.rights.includes(r)); }
function draftKey(card) { return sheet.name + "|" + card; }
function keepDrafts() { save("inspectionDrafts", drafts); }
/// The saved values of a card: the results of a circuit, or (`""`) the header with
/// the answers of the visual inspection as `check.<item>`.
function storedValues(card) {
  if (card !== "") return sheet.circuits.find(c => c.circuit === card).result;
  const h = Object.assign({}, sheet.header);
  for (const [k, v] of Object.entries(h.checks || {})) h["check." + k] = v;
  return h;
}
function stored(card, key) { return storedValues(card)[key] || ""; }
function storedRev(card) { return storedValues(card).rev || 0; }
/// The number of a measured value, as `inspection::number` reads it (`> 500`, `0,35`).
function measured(text) {
  const m = String(text || "").trim().replace(/^[<>=]+\s*/, "").match(/^[0-9]+(?:[.,][0-9]*)?/);
  return m ? parseFloat(m[0].replace(",", ".")) : null;
}
function num(v) { return Number(v).toLocaleString(T.lang, { maximumFractionDigits: 2 }); }
function missing() { return '<span class="missing">' + esc(t("missing")) + "</span>"; }
function today() { const d = new Date(); return new Date(d.getTime() - d.getTimezoneOffset() * 60000).toISOString().slice(0, 10); }
function verdictBadge(v) { return '<span class="badge ' + v + '">' + esc(t(VERDICTS[v])) + "</span>"; }

/// The range of a field of a card, as the server checks it.
function limitOf(card, key) {
  if (card === "") return key === "earth_electrode" && sheet.earth_electrode_max != null ? { min: null, max: sheet.earth_electrode_max } : null;
  return sheet.circuits.find(c => c.circuit === card).limits.find(l => l.field === key) || null;
}
function limitText(l) {
  if (!l) return "";
  const lo = l.min ?? null, hi = l.max ?? null;
  return lo !== null && hi !== null ? num(lo) + " – " + num(hi) : lo !== null ? "≥ " + num(lo) : hi !== null ? "≤ " + num(hi) : "";
}
function bad(card, key, value) {
  if (key === "function" || key.startsWith("check.")) return value === "failed";
  const v = measured(value), l = limitOf(card, key);
  return v !== null && !!l && ((l.min != null && v < l.min) || (l.max != null && v > l.max));
}

async function showInspection() {
  const box = document.getElementById("inspection");
  if (!box) return;
  if (!sheet || sheet.name.toLowerCase() !== shown.name.toLowerCase()) box.innerHTML = '<div class="status">' + esc(t("Loading…")) + "</div>";
  let answer;
  try { answer = await api(projectPath(shown.name) + "/inspection?lang=" + encodeURIComponent(T.lang)); }
  catch (e) { box.innerHTML = '<div class="status error">' + esc(e.message) + "</div>"; return; }
  sheet = answer;
  if (view === "project" && shown.tab === "inspection") drawInspection();
}

function drawInspection() {
  const box = document.getElementById("inspection");
  if (!box || !sheet) return;
  const count = {};
  for (const c of sheet.circuits) count[c.verdict] = (count[c.verdict] || 0) + 1;
  const instruments = sheet.circuits.map(c => c.result.instrument).concat([load("inspectionInstrument")]);
  let html = '<div class="toolbar"><div class="summary"><strong>' + tf("{} circuits", sheet.circuits.length) + "</strong>" +
    Object.keys(VERDICTS).filter(v => count[v]).map(v => '<span class="badge ' + v + '">' + esc(t(VERDICTS[v])) + " " + count[v] + "</span>").join("") +
    '<span class="muted">U0 ' + esc(Math.round(sheet.u0)) + " V · " + esc(sheet.earthing) + "</span></div>" +
    '<div class="filter"><input type="search" id="circuit-filter" placeholder="' + esc(t("Filter circuits…")) + '" aria-label="' + esc(t("Filter circuits…")) + '" value="' + esc(circuitFilter) + '">' +
    '<button id="excel" title="' + esc(t("Download the report as an Excel workbook")) + '">' + esc(t("Excel")) + '</button><span id="excel-msg"></span></div></div>';
  if (!mayRecord()) html += '<p class="notice">' + esc(t("You may read the report; entering results needs the right to save reviews.")) + "</p>";
  html += datalist("instruments", instruments) + datalist("reasons", REASONS.map(t)) + '<div id="report-card"></div><div id="circuits"></div>';
  box.innerHTML = html;
  document.getElementById("circuit-filter").oninput = e => { circuitFilter = e.target.value; drawCircuits(); };
  document.getElementById("excel").onclick = downloadExcel;
  drawReportCard();
  drawCircuits();
}

/// The value a field shows: typed, saved, otherwise for a new record who tests, today,
/// the instrument used last and the test voltage of the standard.
function shownValue(card, key) {
  const draft = drafts[draftKey(card)] || {};
  if (key in draft) return draft[key];
  const v = stored(card, key);
  if (v || !mayRecord() || card === "" || storedRev(card) > 0) return v;
  if (key === "tester") return session ? session.user.name : sheet.header.inspector || "";
  if (key === "date") return today();
  if (key === "instrument") return load("inspectionInstrument") || "";
  if (key === "insulation_voltage") return String(sheet.insulation_voltage);
  return "";
}

/// An input of a card with its label, its range and its state.
function input(card, fd) {
  const v = shownValue(card, fd.key);
  let html;
  if (fd.answer || fd.check) {
    const answers = [["", "–"], ["ok", t("OK")], ["failed", t("Not OK")]].concat(fd.check ? [["n/a", t("Not applicable")]] : []);
    html = '<select data-field="' + esc(fd.key) + '">' + answers.map(([k, l]) => '<option value="' + k + '"' + (k === v ? " selected" : "") + ">" + esc(l) + "</option>").join("") + "</select>";
  } else if (fd.area) {
    html = '<textarea rows="2" data-field="' + fd.key + '">' + esc(v) + "</textarea>";
  } else if (fd.date) {
    html = '<input type="date" data-field="' + fd.key + '" value="' + esc(v) + '">';
  } else {
    const extra = fd.measure ? ' inputmode="decimal" autocomplete="off" enterkeyhint="next"' : fd.list ? ' list="' + fd.list + '"' : "";
    html = '<input data-field="' + fd.key + '"' + extra + ' value="' + esc(v) + '">';
  }
  const limit = fd.measure ? limitText(limitOf(card, fd.key)) : "";
  const cls = [fd.wide ? "wide" : "", fd.check ? "check-item" : "", bad(card, fd.key, v) ? "bad" : ""].filter(Boolean).join(" ");
  return '<label class="' + cls + '"><span>' + esc(t(fd.title)) + (limit ? ' <span class="limit">' + esc(limit) + "</span>" : "") + "</span>" + html + "</label>";
}

function buttons(extra) {
  return mayRecord() ? '<div class="buttons"><button class="primary" data-save>' + esc(t("Save")) + "</button>" + extra + '<span class="msg"></span></div>' : "";
}

function drawReportCard() {
  const box = document.getElementById("report-card");
  if (!box) return;
  const h = sheet.header;
  const draft = drafts[draftKey("")];
  const fails = (sheet.failed || []).map(f => f === "earth_electrode" ? t("Earth electrode resistance RA (Ω)") : t((sheet.checks.find(c => "check." + c.key === f) || {}).title || f));
  const signature = h.signed_by ? tf("Signed by {} on {}", h.signed_by, h.signed_on) : esc(t("Not signed"));
  const facts = [
    ["Supply system", esc("U0 " + Math.round(sheet.u0) + " V · " + sheet.earthing)],
    ["Insulation test voltage (V)", esc(num(sheet.insulation_voltage))],
    ["Signature", signature],
  ];
  if (fails.length) facts.push(["Not passed", '<span class="error">' + fails.map(esc).join(", ") + "</span>"]);
  const checks = sheet.checks.map(c => input("", { key: "check." + c.key, title: c.title, check: true })).join("");
  const sign = mayRecord() ? '<button data-sign title="' + esc(t("Changing the data or the results afterwards takes the signature away.")) + '">' + esc(t("Sign the report")) + "</button>" : "";
  box.innerHTML = '<details class="circuit report" data-circuit=""' + (openCard === "" ? " open" : "") + "><summary><strong>" + esc(t("Report data and visual inspection")) + '</strong><span class="desc">' +
    esc([h.client, h.site].filter(Boolean).join(" · ")) + "</span>" + (draft ? '<span class="badge incomplete">' + esc(t("Not saved")) + "</span>" : "") +
    (h.signed_by ? '<span class="badge passed">' + esc(t("Signed")) + "</span>" : "") + verdictBadge(sheet.verdict) + '</summary><div class="body"><dl>' +
    facts.map(([k, v]) => "<dt>" + esc(t(k)) + "</dt><dd>" + v + "</dd>").join("") + '</dl><fieldset class="measure"' + (mayRecord() ? "" : " disabled") + ">" +
    HEADER.map(fd => input("", fd)).join("") + '<h4 class="wide">' + esc(t("Measured at the origin")) + "</h4>" + ORIGIN.map(fd => input("", fd)).join("") +
    '<h4 class="wide">' + esc(t("Visual inspection")) + "</h4>" + checks + "</fieldset>" + buttons(sign) + "</div></details>";
  wire(box.querySelector("details"));
}

function drawCircuits() {
  const box = document.getElementById("circuits");
  if (!box) return;
  const f = circuitFilter.trim().toLowerCase();
  const list = sheet.circuits.filter(c => !f || c.circuit.toLowerCase().includes(f) || (c.description || "").toLowerCase().includes(f));
  box.innerHTML = list.map(card).join("") || '<p class="muted">' + esc(sheet.circuits.length ? t("No circuits found") : t("The project has no circuits fed from the mains")) + "</p>";
  box.querySelectorAll("details.circuit").forEach(wire);
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
  const inputs = FIELDS.filter(fd => !fd.rcd || c.rcd).map(fd => input(c.circuit, fd)).join("");
  return '<details class="circuit" data-circuit="' + esc(c.circuit) + '"' + (openCard === c.circuit ? " open" : "") + "><summary><strong>" + esc(c.circuit) + '</strong><span class="desc">' +
    (c.description ? esc(c.description) : missing()) + "</span>" + (draft ? '<span class="badge incomplete">' + esc(t("Not saved")) + "</span>" : "") + verdictBadge(c.verdict) +
    '</summary><div class="body"><dl>' + facts.map(([k, v]) => "<dt>" + esc(t(k)) + "</dt><dd>" + v + "</dd>").join("") +
    '</dl><fieldset class="measure"' + (mayRecord() ? "" : " disabled") + ">" + inputs + "</fieldset>" + buttons("<button data-next>" + esc(t("Next circuit")) + "</button>") + "</div></details>";
}

function wire(d) {
  const key = d.dataset.circuit;
  d.ontoggle = () => {
    if (d.open) {
      openCard = key;
      document.querySelectorAll("#inspection details.circuit[open]").forEach(o => { if (o !== d) o.open = false; });
    } else if (openCard === key) openCard = null;
  };
  d.querySelectorAll("[data-field]").forEach(field => {
    field.oninput = field.onchange = () => {
      const k = draftKey(key);
      const draft = drafts[k] || {};
      if (field.value === stored(key, field.dataset.field)) delete draft[field.dataset.field]; else draft[field.dataset.field] = field.value;
      if (Object.keys(draft).length) drafts[k] = draft; else delete drafts[k];
      keepDrafts();
      field.closest("label").classList.toggle("bad", bad(key, field.dataset.field, field.value));
    };
  });
  const saveButton = d.querySelector("[data-save]");
  if (saveButton) saveButton.onclick = () => saveCard(key, d);
  const sign = d.querySelector("[data-sign]");
  if (sign) sign.onclick = () => {
    const who = (session ? session.user.name : "") || d.querySelector('[data-field="inspector"]').value;
    saveCard(key, d, { signed_by: who, signed_on: today() });
  };
  const next = d.querySelector("[data-next]");
  if (next) next.onclick = () => {
    let n = d.nextElementSibling;
    while (n && !n.matches("details.circuit")) n = n.nextElementSibling;
    if (!n) return;
    n.open = true;
    n.scrollIntoView({ block: "start", behavior: "smooth" });
  };
}

/// Saves the changed values of a card (with `extra` ones, the signature).
async function saveCard(key, d, extra) {
  const values = {};
  d.querySelectorAll("[data-field]").forEach(field => { if (field.value !== stored(key, field.dataset.field)) values[field.dataset.field] = field.value; });
  Object.assign(values, extra || {});
  const msg = (text, ok) => { const m = document.querySelector("#inspection details.circuit[open] .msg"); if (m) { m.className = "msg " + (ok ? "ok" : "error"); m.textContent = text; } };
  if (!Object.keys(values).length) { msg(t("Nothing to save")); return; }
  d.querySelectorAll("button").forEach(b => b.disabled = true);
  const user = values.tester || values.inspector || stored(key, key === "" ? "inspector" : "tester") || (session ? session.user.name : "") || sheet.header.inspector || "";
  try {
    sheet = await api(projectPath(sheet.name) + "/inspection?lang=" + encodeURIComponent(T.lang), { method: "PUT", json: { user, circuit: key, rev: storedRev(key), values } });
  } catch (e) {
    d.querySelectorAll("button").forEach(b => b.disabled = false);
    if (e.status !== 409) { msg(e.message); return; }
    // Saved by someone else meanwhile: the values typed here stay, over theirs.
    for (const [k, v] of Object.entries(values)) if (!(extra && k in extra)) (drafts[draftKey(key)] ||= {})[k] = v;
    keepDrafts();
    await showInspection();
    msg(t("Someone saved this meanwhile. Check the values and save again."));
    return;
  }
  // The new revision is this save: the look at the project every 20 s draws nothing anew.
  if (project && project.name === sheet.name) project.server_revision = sheet.revision;
  delete drafts[draftKey(key)];
  keepDrafts();
  if (values.instrument) save("inspectionInstrument", values.instrument);
  openCard = key;
  drawInspection();
  msg(extra ? t("Signed") : t("Saved"), true);
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
