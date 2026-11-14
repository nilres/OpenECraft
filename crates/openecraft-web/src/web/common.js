"use strict";
/// What the web pages of the OpenECraft servers share (parts server, project server):
/// texts, storage, requests to the API, the language, the sign-in and the rights of
/// the session. Loaded first. The page script defines the hooks `adminAllowed()`,
/// `signedIn()` (after a sign-in), `signedOut(left)` (the session ended; `left`: the
/// user signed out) and `route()` (shows the view of the address).
const API = "/api/v1";
let T = { texts: {}, keys: {}, languages: [], lang: "en", rights: [], roles: [] };
function load(k) { try { return JSON.parse(localStorage.getItem(k)); } catch (e) { return null; } }
function save(k, v) { try { v === null ? localStorage.removeItem(k) : localStorage.setItem(k, JSON.stringify(v)); } catch (e) {} }
function loadTemporary(k) { try { return JSON.parse(sessionStorage.getItem(k)); } catch (e) { return null; } }
let session = load("session") || loadTemporary("session");
/// The sign-in form: "user" (name and password), "token" (administrator token) or
/// "register" (a new account, when the server lets users register).
let loginMode = "user";
let registrationOpen = false;

/// Keeps the session for the next visit; one with the access token only for this tab.
function keepSession() {
  const temporary = !!session && session.kind === "token";
  try { temporary ? sessionStorage.setItem("session", JSON.stringify(session)) : sessionStorage.removeItem("session"); } catch (e) {}
  save("session", session && !temporary ? session : null);
}
function esc(s) { return String(s ?? "").replace(/[&<>"']/g, c => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c])); }
function t(s) { return T.texts[s] || s; }
/// Text with `{}` placeholders for HTML (the arguments are escaped).
function tf(s, ...args) { let i = 0; return t(s).replace(/\{(\d*)\}/g, (_, n) => esc(args[n === "" ? i++ : +n])); }
/// Text with `{}` placeholders for dialogs (not HTML: nothing is escaped).
function tp(s, ...args) { let i = 0; return t(s).replace(/\{(\d*)\}/g, (_, n) => String(args[n === "" ? i++ : +n])); }
function key(kind, k) { return T.keys[kind + ":" + k] || k; }
function date(secs) { return secs ? new Date(secs * 1000).toLocaleDateString(T.lang) : ""; }
function dateTime(secs) { return secs ? new Date(secs * 1000).toLocaleString(T.lang, { dateStyle: "short", timeStyle: "short" }) : ""; }
function content(html) { document.getElementById("content").innerHTML = html; }

async function api(path, options = {}) {
  const headers = Object.assign({}, options.headers || {});
  if (session && session.token) headers.Authorization = "Bearer " + session.token;
  if (options.json !== undefined) { headers["Content-Type"] = "application/json"; options.body = JSON.stringify(options.json); }
  const r = await fetch(API + path, Object.assign({}, options, { headers }));
  // A session the server no longer knows: sign out and go on anonymously.
  if (r.status === 401 && session && !options.retried && path !== "/login") {
    session = null; keepSession(); signedOut(false); renderAccount();
    return api(path, Object.assign({}, options, { retried: true }));
  }
  if (!r.ok) {
    let msg = r.status + " " + r.statusText;
    try { msg = (await r.json()).error || msg; } catch (e) {}
    const err = new Error(msg); err.status = r.status; throw err;
  }
  return r.status === 204 ? null : options.raw ? r.blob() : r.json();
}

/// The texts of the page in its language; the title is `data-title` of the body.
function translatePage() {
  document.documentElement.lang = T.lang;
  document.title = t(document.body.dataset.title || document.title);
  document.querySelectorAll("[data-t]").forEach(e => e.textContent = t(e.dataset.t));
  document.querySelectorAll("[data-tp]").forEach(e => e.placeholder = t(e.dataset.tp));
  const sel = document.getElementById("lang");
  sel.setAttribute("aria-label", t("Language"));
  sel.innerHTML = T.languages.map(([c, n]) => '<option value="' + c + '"' + (c === T.lang ? " selected" : "") + ">" + esc(n) + "</option>").join("");
}

async function setLanguage(code) {
  T = await api("/texts?lang=" + encodeURIComponent(code));
  save("lang", T.lang);
  translatePage();
  renderAccount();
}

/// The language of the address (`?lang=de`) wins over the one chosen last.
function startLanguage() { return new URLSearchParams(location.search).get("lang") || load("lang") || navigator.language || "en"; }

// Rights of the session

/// Case-insensitive match with `*` (any text) and `?` (one character).
function wild(pattern, text) {
  const re = pattern.trim().toLowerCase().replace(/[.+^${}()|[\]\\]/g, "\\$&").replace(/\*/g, ".*").replace(/\?/g, ".");
  return new RegExp("^" + re + "$", "s").test(text.trim().toLowerCase());
}
function serverWide(right) { const r = (T.rights || []).find(x => x.name === right); return r ? r.server_wide : right === "read" || right === "administer"; }
/// The rights without sign-in, if the page shows them (from `/info`).
let guest = null;
function entries() { const a = session ? session.access : guest; return (a && a.entries) || []; }
/// The caller's rights on a scope (a project, a manufacturer, `catalog:<id>` or `*`), as `Access::rights` resolves them.
function rightsOn(scope) { return rightsOnAll([scope]); }
/// The rights on several scopes together, as `Access::rights_on` resolves them: a
/// deny on any of them takes the right away.
function rightsOnAll(scopes) {
  scopes = scopes.map(s => String(s || "").trim());
  const server = scopes.includes("*");
  const covering = entries().filter(e => { const p = (e.scope || "").trim() || "*"; return scopes.some(s => s === "*" ? p === "*" : wild(p, s)); });
  const denied = new Set(covering.filter(e => e.deny).flatMap(e => e.rights || []));
  return new Set(covering.filter(e => !e.deny).flatMap(e => e.rights || []).filter(r => !denied.has(r) && (server || !serverWide(r))));
}
function may(right, scope) { return rightsOn(serverWide(right) ? "*" : (scope || "").trim()).has(right); }
function mayAnywhere(right) { return entries().some(e => !e.deny && (e.rights || []).includes(right)); }

function roleTitle(name) {
  const b = (T.roles || []).find(r => r.name.toLowerCase() === String(name || "").trim().toLowerCase());
  return b ? b.title : name;
}
/// The role of a user as shown: the translated built-in role.
function userRole(u) { return u && u.role ? T.keys["role:" + u.role] || roleTitle(u.role) : ""; }

// Sign-in

function renderAccount() {
  const box = document.getElementById("account");
  document.getElementById("nav-admin").hidden = !adminAllowed();
  if (session) {
    const role = userRole(session.user);
    box.innerHTML = '<span class="muted">' + tf("Signed in as {}", session.user.name) + (role ? " (" + esc(role) + ")" : "") + '</span><button id="logout">' + esc(t("Sign out")) + "</button>";
    document.getElementById("logout").onclick = logout;
    return;
  }
  if (loginMode === "register" && !registrationOpen) loginMode = "user";
  const input = (id, type, auto, hint) => '<input id="' + id + '"' + (type ? ' type="' + type + '"' : "") + ' autocomplete="' + auto + '" placeholder="' + esc(hint) + '" aria-label="' + esc(hint) + '">';
  const link = (mode, label, title) => '<button type="button" class="link" data-mode="' + mode + '"' + (title ? ' title="' + esc(title) + '"' : "") + ">" + esc(label) + "</button>";
  let fields, submit, links;
  if (loginMode === "token") {
    fields = input("token", "password", "off", t("Access token"));
    submit = t("Sign in");
    links = link("user", t("User account"), t("Sign in with a user account"));
  } else if (loginMode === "register") {
    fields = input("user", "", "username", t("User name")) + input("pw", "password", "new-password", t("Password")) + input("pw2", "password", "new-password", t("Repeat password"));
    submit = t("Register");
    links = link("user", t("Cancel"));
  } else {
    fields = input("user", "", "username", t("User name")) + input("pw", "password", "current-password", t("Password"));
    submit = t("Sign in");
    links = (registrationOpen ? link("register", t("Register"), t("Create an account on this server")) : "") + link("token", t("Access token"), t("Sign in with the access token of the server"));
  }
  box.innerHTML = '<form id="login" style="display:flex;gap:6px;flex-wrap:wrap">' + fields + '<button class="' + (loginMode === "register" ? "primary" : "") + '">' + esc(submit) + "</button>" + links +
    '</form><span id="login-error" class="error"></span>';
  box.querySelectorAll("[data-mode]").forEach(b => b.onclick = () => { loginMode = b.dataset.mode; renderAccount(); document.querySelector("#login input").focus(); });
  const fail = msg => { document.getElementById("login-error").textContent = msg; };
  document.getElementById("login").onsubmit = async e => {
    e.preventDefault();
    const value = id => document.getElementById(id).value;
    try {
      if (loginMode === "token") {
        const token = value("token").trim();
        const info = await api("/info", { headers: { Authorization: "Bearer " + token } });
        if (!info.user) throw Object.assign(new Error(t("Wrong access token")), { status: 0 });
        session = { token, user: info.user, access: info.access, kind: "token" };
      } else if (loginMode === "register") {
        const name = value("user").trim();
        if (!name || /\s/.test(name)) { fail(t("A user name must not be empty or contain blanks")); return; }
        if (value("pw") !== value("pw2")) { fail(t("The passwords do not match")); return; }
        if (value("pw").trim().length < 8) { fail(tp("The password needs at least {} characters", 8)); return; }
        const created = await api("/register", { method: "POST", json: { name, password: value("pw") } });
        loginMode = "user";
        // Waiting for an administrator: no session yet.
        if (!created.token) {
          renderAccount();
          const note = document.getElementById("login-error");
          note.className = "ok";
          note.textContent = t("Your account was created. An administrator has to approve it before you can sign in.");
          return;
        }
        session = created;
      } else {
        session = await api("/login", { method: "POST", json: { name: value("user"), password: value("pw") } });
      }
      keepSession();
      await signedIn();
      renderAccount();
      route();
    } catch (err) {
      const register = loginMode === "register";
      if (register && err.status === 409) fail(t("This user name is taken"));
      else if (register && err.status === 403) { registrationOpen = false; renderAccount(); fail(t("The registration is closed")); }
      else if (register && err.status === 429) fail(t("Too many registrations from this address, try again later"));
      else if (register && err.status === 400) fail(t("This user name cannot be used"));
      else if (loginMode === "user" && err.status === 403) fail(t("Your account awaits the approval of an administrator"));
      else fail(err.status === 401 ? (loginMode === "token" ? t("Wrong access token") : t("Wrong user name or password")) : err.status === 429 ? t("Too many failed sign-ins, try again later") : err.message);
    }
  };
}

/// The user and the rights of the session as the server sees them now (roles and
/// grants may have changed since the sign-in).
async function refreshSession() {
  if (!session) return;
  try {
    const info = await api("/info");
    if (info.user && session) { session.user = info.user; session.access = info.access; keepSession(); }
  } catch (e) {}
  renderAccount();
}

async function logout() {
  try { await api("/logout", { method: "POST" }); } catch (e) {}
  session = null; keepSession();
  renderAccount();
  signedOut(true);
}

// Administration

/// The access settings being edited (saved together).
let policy = null;
let providers = [];
function roleNames() { return (T.roles || []).map(r => r.name).concat(((policy && policy.roles) || []).map(r => r.name.trim()).filter(Boolean)); }
function options(values, selected, title) { return values.map(v => '<option value="' + esc(v) + '"' + (v === selected ? " selected" : "") + ">" + esc(title ? title(v) : v) + "</option>").join(""); }
function datalist(id, values) { return '<datalist id="' + id + '">' + [...new Set(values.filter(Boolean))].sort().map(v => '<option value="' + esc(v) + '">').join("") + "</datalist>"; }
function message(id, text, ok) { const e = document.getElementById(id); if (e) { e.className = ok ? "ok" : "error"; e.textContent = text; } }
/// A label for the input of `html` (the first `id` in it).
function labelFor(label, html) { const id = (html.match(/id="([^"]+)"/) || [])[1]; return "<label" + (id ? ' for="' + id + '"' : "") + ">" + esc(t(label)) + "</label>"; }
function adminBody(html) { const b = document.getElementById("admin-body"); if (b) b.innerHTML = html; return !!b; }
function focusLast(sel) { const l = document.querySelectorAll(sel); if (l.length) l[l.length - 1].focus(); }
/// A row of the form: label, input and hint.
function field(label, input, hint) { return labelFor(label, input) + "<div>" + input + (hint ? '<div class="hint">' + esc(hint) + "</div>" : "") + "</div>"; }

/// The tabs of the administration (`[id, title, allowed, view]`, the allowed ones) with
/// the body of `current`; `go(id)` shows another tab.
async function adminTabs(tabs, current, go) {
  content('<div class="admin"><div class="tabs">' + tabs.map(([id, title]) => '<button data-tab="' + id + '"' + (id === current ? ' class="active"' : "") + ">" + esc(t(title)) + "</button>").join("") +
    '</div><div id="admin-body"><div class="status">' + esc(t("Loading…")) + "</div></div></div>");
  document.querySelectorAll("[data-tab]").forEach(b => b.onclick = () => go(b.dataset.tab));
  try { await tabs.find(([id]) => id === current)[3](); }
  catch (e) {
    // Signed out meanwhile (e.g. the own sessions ended).
    if (!session) { route(); return; }
    adminBody('<div class="status error">' + esc(e.message) + "</div>");
  }
}
