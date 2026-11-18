"use strict";
/// The inspection and test report of the open project (project.js), made for a tablet
/// or a phone on site: a card with the data of the report, the visual inspection, the
/// verdict and the signature, a card per source of SELV, PELV or separated circuits and
/// a card per circuit with what the schematic gives (missing data marked as such) and
/// the fields of the results. Each card is saved on its own without opening the
/// project; values typed but not saved stay in the browser.

/// The answer of `/inspection` for the open project.
let sheet = null;
let circuitFilter = "";
/// The card that is open (see `sourceOf`), `null` for none.
let openCard = null;
/// Values typed but not saved by `project|card`: `{ field: value }`.
let drafts = load("inspectionDrafts") || {};

const FIELDS = [
  { key: "continuity", title: "Protective conductor (Ω)", measure: true },
  { key: "insulation_voltage", title: "Insulation test voltage (V)", measure: true },
  { key: "insulation", title: "Insulation L–PE (MΩ)", measure: true },
  { key: "insulation_n_pe", title: "Insulation N–PE (MΩ)", measure: true },
  { key: "insulation_l_n", title: "Insulation L–N (MΩ)", measure: true },
  { key: "loop_impedance", title: "Loop impedance Zs (Ω)", measure: true },
  { key: "voltage_drop", title: "Voltage drop (%)", measure: true },
  { key: "rcd_current", title: "RCD trip current (mA)", measure: true, rcd: true },
  { key: "rcd_time", title: "RCD trip time (ms)", measure: true, rcd: true },
  { key: "rcd_time_5", title: "RCD trip time at 5 × IΔn (ms)", measure: true, rcd: true },
  { key: "rcd_voltage", title: "Contact voltage UB (V)", measure: true, rcd: true },
  { key: "polarity", title: "Polarity", answer: true },
  { key: "phase_sequence", title: "Phase sequence", answer: true, phases: true },
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
  { key: "floor_insulation", title: "Insulating floors and walls (kΩ)", measure: true },
];
const IT = [
  { key: "imd", title: "Insulation monitoring device", check: true },
  { key: "first_fault", title: "First fault current Id (A)", measure: true },
];
const SOURCE = [
  { key: "measure", title: "Protective measure", measures: true },
  { key: "insulation_voltage", title: "Insulation test voltage (V)", measure: true },
  { key: "insulation", title: "Insulation resistance (MΩ)", measure: true },
  { key: "remark", title: "Remark", wide: true, area: true },
];
const MEASURES = [["", "–"], ["selv", "SELV"], ["pelv", "PELV"], ["separation", "Electrical separation"], ["n/a", "No protective measure of its own"]];
const REASONS = ["New installation", "Extension", "Change", "Repair"];
const VERDICTS = { failed: "Not passed", incomplete: "Incomplete", not_tested: "Not tested", passed: "Passed" };

/// Reviewers and editors enter results.
function mayRecord() { return !!sheet && ["review", "edit_pages", "edit_project"].some(r => sheet.rights.includes(r)); }
/// The key of a card is a circuit's device tag, `source:<tag>` or `""` for the report.
function sourceOf(card) { return card.startsWith("source:") ? sheet.sources.find(s => s.device === card.slice(7)) : null; }
function draftKey(card) { return sheet.name + "|" + card; }
function keepDrafts() { save("inspectionDrafts", drafts); }
/// The saved values of a card: the report with the answers of the visual inspection
/// (`check.<item>`, `own.<n>` and `own.<n>.title`) and its own limits (`limit.<name>`).
function storedValues(card) {
  if (card === "") {
    const h = Object.assign({}, sheet.header);
    for (const c of sheet.checks) h[c.key] = c.answer;
    (h.own_checks || []).forEach((c, i) => h["own." + (i + 1) + ".title"] = c.title);
    for (const [k, v] of Object.entries(h.limits || {})) h["limit." + k] = String(v);
    return h;
  }
  const s = sourceOf(card);
  return s ? s.result : sheet.circuits.find(c => c.circuit === card).result;
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

/// The range of a field of a card, as the server checks it (a source: by the measure shown).
function limitOf(card, key) {
  if (card === "") return sheet.limits.find(l => l.key === key) || null;
  const s = sourceOf(card);
  if (s) return (s.limits[shownValue(card, "measure")] || []).find(l => l.key === key) || null;
  return sheet.circuits.find(c => c.circuit === card).limits.find(l => l.field === key) || null;
}
function limitText(l) {
  if (!l) return "";
  const lo = l.min ?? null, hi = l.max ?? null;
  return lo !== null && hi !== null ? num(lo) + " – " + num(hi) : lo !== null ? "≥ " + num(lo) : hi !== null ? "≤ " + num(hi) : "";
}
function bad(card, key, value) {
  const s = sourceOf(card);
  if (s && key === "measure") return s.failed.includes("measure") && value === s.result.measure;
  if (FIELDS.some(f => f.key === key && f.answer) || key === "imd" || key.startsWith("check.") || /^own\.\d+$/.test(key)) return value === "failed";
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
  html += datalist("instruments", instruments) + datalist("reasons", REASONS.map(t)) + '<div id="report-card"></div><div id="sources"></div><div id="circuits"></div>';
  box.innerHTML = html;
  document.getElementById("circuit-filter").oninput = e => { circuitFilter = e.target.value; drawCircuits(); };
  document.getElementById("excel").onclick = downloadExcel;
  drawReportCard();
  drawSources();
  drawCircuits();
}

/// The value a field shows: typed, saved, otherwise for a new circuit who tests, today,
/// the instrument used last and the test voltage of the standard.
function shownValue(card, key) {
  const draft = drafts[draftKey(card)] || {};
  if (key in draft) return draft[key];
  const v = stored(card, key);
  if (v || !mayRecord() || card === "" || sourceOf(card) || storedRev(card) > 0) return v;
  if (key === "tester") return session ? session.user.name : sheet.header.inspector || "";
  if (key === "date") return today();
  if (key === "instrument") return load("inspectionInstrument") || "";
  if (key === "insulation_voltage") return String(sheet.insulation_voltage);
  return "";
}

/// An input of a card with its label, its range and its state.
function input(card, fd) {
  const v = shownValue(card, fd.key);
  const select = options => '<select data-field="' + esc(fd.key) + '">' + options.map(([k, l]) => '<option value="' + k + '"' + (k === v ? " selected" : "") + ">" + esc(l) + "</option>").join("") + "</select>";
  let html;
  if (fd.measures) {
    html = select(MEASURES.map(([k, l]) => [k, k ? t(l) : l]));
  } else if (fd.answer || fd.check) {
    html = select([["", "–"], ["ok", t("OK")], ["failed", t("Not OK")]].concat(fd.check ? [["n/a", t("Not applicable")]] : []));
  } else if (fd.area) {
    html = '<textarea rows="2" data-field="' + fd.key + '">' + esc(v) + "</textarea>";
  } else if (fd.date) {
    html = '<input type="date" data-field="' + fd.key + '" value="' + esc(v) + '">';
  } else {
    const extra = fd.measure ? ' inputmode="decimal" autocomplete="off" enterkeyhint="next"' : fd.list ? ' list="' + fd.list + '"' : "";
    html = '<input data-field="' + fd.key + '"' + extra + ' value="' + esc(v) + '">';
  }
  const c = card !== "" && !sourceOf(card) ? sheet.circuits.find(x => x.circuit === card) : null;
  // The protective conductor: its calculated resistance as a guide, a hint far above it.
  const calculated = c && fd.key === "continuity" && c.continuity_calculated != null ? "≈ " + num(c.continuity_calculated) : "";
  const limit = fd.measure ? limitText(limitOf(card, fd.key)) || calculated : "";
  const note = c && c.notes.includes(fd.key) && v === stored(card, fd.key);
  const cls = [fd.wide ? "wide" : "", fd.long ? "check-item" : "", bad(card, fd.key, v) ? "bad" : note ? "note" : ""].filter(Boolean).join(" ");
  return '<label class="' + cls + '"><span>' + esc(fd.own ? fd.title : t(fd.title)) + ' <span class="limit">' + esc(limit) + "</span></span>" + html + "</label>";
}

function buttons(extra) {
  return mayRecord() ? '<div class="buttons"><button class="primary" data-save>' + esc(t("Save")) + "</button>" + extra + '<span class="msg"></span></div>' : "";
}

/// A card: its summary line, the facts of the schematic and the fields.
function cardHtml(key, title, desc, badges, facts, fields, extra) {
  const draft = drafts[draftKey(key)];
  return '<details class="circuit' + (key === "" ? " report" : "") + '" data-card="' + esc(key) + '"' + (openCard === key ? " open" : "") + "><summary><strong>" + esc(title) + '</strong><span class="desc">' + desc + "</span>" +
    (draft ? '<span class="badge incomplete">' + esc(t("Not saved")) + "</span>" : "") + badges + '</summary><div class="body"><dl>' +
    facts.map(([k, v]) => "<dt>" + esc(t(k)) + "</dt><dd>" + v + "</dd>").join("") + '</dl><fieldset class="measure"' + (mayRecord() ? "" : " disabled") + ">" + fields + "</fieldset>" + buttons(extra) + "</div></details>";
}

function drawReportCard() {
  const box = document.getElementById("report-card");
  if (!box) return;
  const h = sheet.header;
  const titles = Object.fromEntries(HEADER.concat(ORIGIN, IT).map(f => [f.key, f.title]));
  const text = f => {
    if (f.startsWith("breaking.")) return tf("Breaking capacity of {} below the fault current at the origin", f.slice(9));
    const check = sheet.checks.find(c => c.key === f);
    return esc(check && f.startsWith("own.") ? check.title : t(titles[f] || (check || {}).title || f));
  };
  const fails = (sheet.failed || []).map(text);
  const notes = (sheet.notes || []).map(text);
  const facts = [
    ["Supply system", esc("U0 " + Math.round(sheet.u0) + " V · " + sheet.earthing)],
    ["Insulation test voltage (V)", esc(num(sheet.insulation_voltage))],
    ["Signature", h.signed_by ? tf("Signed by {} on {}", h.signed_by, h.signed_on) : esc(t("Not signed"))],
  ];
  if (fails.length) facts.push(["Not passed", '<span class="error">' + fails.join(", ") + "</span>"]);
  if (notes.length) facts.push(["To check", '<span class="warn">' + notes.join(", ") + "</span>"]);
  const section = title => '<h4 class="wide">' + esc(t(title)) + "</h4>";
  const checks = sheet.checks.filter(c => c.key.startsWith("check.")).map(c => input("", { key: c.key, title: c.title, check: true, long: true })).join("");
  const own = (h.own_checks || []).map((c, i) => ownItem(i + 1)).join("");
  const limits = sheet.settings.map(l => '<label class="limit-item" title="' + esc(l.source) + '"><span>' + esc(t(l.title)) + ' <span class="limit">' + esc(l.source) + "</span></span>" +
    '<input data-field="limit.' + esc(l.key) + '" inputmode="decimal" autocomplete="off" placeholder="' + esc(num(l.default)) + '" value="' + esc(shownValue("", "limit." + l.key)) + '"></label>').join("");
  const fields = HEADER.map(fd => input("", fd)).join("") + section("Measured at the origin") + ORIGIN.map(fd => input("", fd)).join("") +
    (sheet.it ? section("IT system") + IT.map(fd => input("", fd)).join("") : "") +
    section("Visual inspection") + checks + '<div class="wide own-items">' + own + "</div>" +
    (mayRecord() ? '<div class="wide"><button type="button" data-add-own>' + esc(t("Add an item")) + "</button></div>" : "") +
    '<details class="wide limits"><summary>' + esc(t("Limits")) + '</summary><p class="muted">' + esc(t("Empty: the default. Set other values where the edition of the standard the installation is verified against differs.")) + '</p><div class="limit-grid">' + limits + "</div></details>";
  const sign = mayRecord() ? '<button data-sign title="' + esc(t("Changing the data or the results afterwards takes the signature away.")) + '">' + esc(t("Sign the report")) + "</button>" : "";
  const badges = (h.signed_by ? '<span class="badge passed">' + esc(t("Signed")) + "</span>" : "") + verdictBadge(sheet.verdict);
  box.innerHTML = cardHtml("", t("Report data and visual inspection"), esc([h.client, h.site].filter(Boolean).join(" · ")), badges, facts, fields, sign);
  wire(box.querySelector("details"));
}

/// An own item of the visual inspection: its text and its answer.
function ownItem(n) {
  const key = "own." + n;
  const title = shownValue("", key + ".title");
  return '<div class="own-item' + (bad("", key, shownValue("", key)) ? " bad" : "") + '"><input data-field="' + key + '.title" placeholder="' + esc(t("Own item of the visual inspection")) + '" value="' + esc(title) + '">' +
    input("", { key, title: "", check: true }).replace(/^<label class="[^"]*"><span>.*?<\/span><\/span>/, "").replace(/<\/label>$/, "") + "</div>";
}

function drawSources() {
  const box = document.getElementById("sources");
  if (!box) return;
  if (!sheet.sources.length) { box.innerHTML = ""; return; }
  box.innerHTML = '<h3 class="group-title">' + esc(t("SELV, PELV and separation")) + "</h3>" + sheet.sources.map(s => {
    const voltage = s.voltage != null ? esc(num(s.voltage) + " V " + (s.transformer ? "AC" : "DC")) : missing();
    const facts = [["Kind", esc(s.transformer ? t("Transformer") : t("Power supply unit"))], ["Output voltage (V)", voltage]];
    if (s.failed.includes("measure")) facts.push(["Not passed", '<span class="error">' + tf("Above {} V: no SELV or PELV", s.transformer ? 50 : 120) + "</span>"]);
    const key = "source:" + s.device;
    return cardHtml(key, s.device, s.description ? esc(s.description) : missing(), verdictBadge(s.verdict), facts, SOURCE.map(fd => input(key, fd)).join(""), "");
  }).join("");
  box.querySelectorAll("details.circuit").forEach(wire);
}

function drawCircuits() {
  const box = document.getElementById("circuits");
  if (!box) return;
  const f = circuitFilter.trim().toLowerCase();
  const list = sheet.circuits.filter(c => !f || c.circuit.toLowerCase().includes(f) || (c.description || "").toLowerCase().includes(f));
  box.innerHTML = (sheet.sources.length ? '<h3 class="group-title">' + esc(t("Circuits")) + "</h3>" : "") +
    (list.map(circuitCard).join("") || '<p class="muted">' + esc(sheet.circuits.length ? t("No circuits found") : t("The project has no circuits fed from the mains")) + "</p>");
  box.querySelectorAll("details.circuit").forEach(wire);
}

function circuitCard(c) {
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
  const fields = FIELDS.filter(fd => (!fd.rcd || c.rcd) && (!fd.phases || c.phases.length >= 3)).map(fd => input(c.circuit, fd)).join("");
  return cardHtml(c.circuit, c.circuit, c.description ? esc(c.description) : missing(), verdictBadge(c.verdict), facts, fields, "<button data-next>" + esc(t("Next circuit")) + "</button>");
}

function wire(d) {
  const key = d.dataset.card;
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
      // The measure of a source sets the ranges of its other fields.
      const fields = field.dataset.field === "measure" ? [...d.querySelectorAll("[data-field]")] : [field];
      for (const f of fields) {
        const label = f.closest("label, .own-item");
        if (!label) continue;
        label.classList.toggle("bad", bad(key, f.dataset.field, f.value));
        if (field.dataset.field === "measure" && label.querySelector(".limit")) label.querySelector(".limit").textContent = limitText(limitOf(key, f.dataset.field));
      }
    };
  });
  const saveButton = d.querySelector("[data-save]");
  if (saveButton) saveButton.onclick = () => saveCard(key, d);
  const add = d.querySelector("[data-add-own]");
  if (add) add.onclick = () => {
    const box = d.querySelector(".own-items");
    const n = box.querySelectorAll(".own-item").length + 1;
    box.insertAdjacentHTML("beforeend", ownItem(n));
    wire(d);
    box.lastElementChild.querySelector("input").focus();
  };
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
  if (!user && !session) {
    d.querySelectorAll("button").forEach(b => b.disabled = false);
    msg(t("Sign in or name the inspector in the report data first."));
    return;
  }
  const source = sourceOf(key);
  const target = source ? { circuit: "", source: source.device } : { circuit: key };
  try {
    sheet = await api(projectPath(sheet.name) + "/inspection?lang=" + encodeURIComponent(T.lang), { method: "PUT", json: Object.assign({ user, rev: storedRev(key), values }, target) });
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
