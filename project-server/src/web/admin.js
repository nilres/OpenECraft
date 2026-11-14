"use strict";
/// Administration of the project server (overview, sessions, users, access rights, directories)
/// for administrators. The page only offers what the caller may do; the server checks
/// every change. Loaded after common.js and access.js, before project.js.
const ADMIN_TABS = [
  ["overview", "Overview", () => true, adminOverview],
  ["sessions", "Sessions", () => true, adminSessions],
  ["users", "Users", () => true, adminUsers],
  ["access", "Access rights", () => true, adminAccess],
  ["directories", "Directories", () => true, adminDirectories],
];
let adminTab = "overview";

/// The scopes of the access settings (access.js): project names.
const ACCESS = {
  help: () => [t("A role is a set of rights. Grants give a user or a group a role on projects (* and ? as wildcards, * for all); a deny grant takes the rights of the role away, whatever other grants give."),
    t("Managers of a project give rights on it themselves (Permissions on the project).")],
  closed: () => t("Sign-in needed to view"),
  scope: () => t("Project"),
  all: () => t("All projects"),
  scopes: async () => (await api("/projects")).map(p => p.name),
};

async function showAdmin() {
  if (!adminAllowed()) { content('<div class="status">' + esc(t("Sign in as an administrator to manage the server")) + "</div>"); return; }
  if (!ADMIN_TABS.some(([id]) => id === adminTab)) adminTab = "overview";
  await adminTabs(ADMIN_TABS, adminTab, id => { adminTab = id; go(); });
}

async function adminOverview() {
  const [info, projects, users, s, sessions] = await Promise.all([api("/info"), api("/projects"), api("/users"), api("/access"), api("/sessions?limit=0")]);
  const card = (title, pairs) => '<div class="box"><h3>' + esc(t(title)) + "</h3><dl>" + pairs.map(([k, v]) => "<dt>" + esc(t(k)) + "</dt><dd>" + v + "</dd>").join("") + "</dl></div>";
  const open = projects.filter(p => p.users.length);
  const byRole = {};
  for (const u of users) { const r = userRole(u) || t("None (only grants)"); byRole[r] = (byRole[r] || 0) + 1; }
  let html = '<div class="cards">';
  html += card("Server", [["Version", esc(info.version)], ["Projects", String(projects.length)], ["Open sessions", String(sessions.active.length)]]);
  html += card("Users", [["Users", String(users.length)]].concat(Object.entries(byRole).map(([r, n]) => [r, String(n)])).concat([["Directories", String((s.providers || []).length)]]));
  html += card("Access rights", [["Without sign-in", esc(s.policy.anonymous ? roleTitle(s.policy.anonymous) : t("Sign-in needed to view"))], ["Defined roles", String((s.policy.roles || []).length)], ["Grants", String((s.policy.grants || []).length)]]);
  html += "</div>";
  html += '<h3 class="section">' + esc(t("Projects in use")) + "</h3>";
  html += open.length ? '<div class="table-wrap"><table class="plain"><thead><tr><th>' + esc(t("Project")) + "</th><th>" + esc(t("In use by")) + "</th></tr></thead><tbody>" +
    open.map(p => '<tr><td><a href="#' + esc(new URLSearchParams({ project: p.name }).toString()) + '">' + esc(p.name) + "</a></td><td>" + esc(p.users.join(", ")) + "</td></tr>").join("") + "</tbody></table></div>"
    : '<p class="muted">' + esc(t("Nobody works on a project")) + "</p>";
  adminBody(html);
}

// Sessions

/// How a session ended, as shown.
const SESSION_ENDS = { closed: "Closed", expired: "Expired", signed_out: "Signed out", user_deleted: "User deleted" };
let sessionFilter = "";

/// `1:05:09` (hours, minutes, seconds).
function duration(secs) {
  secs = Math.max(0, Math.round(secs));
  const two = n => String(n).padStart(2, "0");
  return Math.floor(secs / 3600) + ":" + two(Math.floor(secs / 60) % 60) + ":" + two(secs % 60);
}

/// The open sessions with what they hold locked, and the ended ones the server keeps.
async function adminSessions() {
  const list = await api("/sessions");
  const now = Date.now() / 1000;
  const projectLink = name => '<a href="#' + esc(new URLSearchParams({ project: name }).toString()) + '">' + esc(name) + "</a>";
  let html = '<div class="toolbar"><strong>' + esc(t("Open sessions")) + " (" + list.active.length + ')</strong><button id="sessions-refresh">' + esc(t("Refresh")) + "</button></div>";
  html += '<div class="table-wrap"><table class="plain"><thead><tr><th class="num">#</th><th>' + esc(t("User")) + "</th><th>" + esc(t("Project")) + "</th><th>" + esc(t("Started")) + '</th><th class="hide-narrow">' +
    esc(t("Last activity")) + '</th><th class="num">' + esc(t("Saves")) + "</th><th>" + esc(t("Locked")) + "</th></tr></thead><tbody>";
  for (const r of list.active) {
    const locked = ((r.locks || []).includes("project") ? [t("Project data")] : []).concat(r.pages || []);
    html += '<tr><td class="num">' + r.number + "</td><td>" + esc(r.user) + "</td><td>" + projectLink(r.project) + "</td><td>" + esc(dateTime(r.started)) + " (" + duration(now - r.started) + ')</td><td class="hide-narrow">' +
      esc(dateTime(r.last_seen)) + '</td><td class="num">' + r.saves + "</td><td>" + esc(locked.join(", ")) + "</td></tr>";
  }
  if (!list.active.length) html += '<tr><td colspan="7" class="muted">' + esc(t("No open sessions")) + "</td></tr>";
  html += '</tbody></table></div><div class="toolbar" style="margin-top:18px"><strong>' + esc(t("History")) + '</strong><input id="session-filter" type="search" placeholder="' + esc(t("Filter sessions…")) +
    '" aria-label="' + esc(t("Filter sessions…")) + '" value="' + esc(sessionFilter) + '"></div><div class="table-wrap" id="session-history"></div><p class="muted">' + tf("The server keeps the last {} ended sessions.", list.kept) + "</p>";
  adminBody(html);
  const drawHistory = () => {
    const f = sessionFilter.trim().toLowerCase();
    const rows = list.history.filter(r => !f || r.user.toLowerCase().includes(f) || r.project.toLowerCase().includes(f));
    let h = '<table class="plain"><thead><tr><th>' + esc(t("User")) + "</th><th>" + esc(t("Project")) + "</th><th>" + esc(t("Started")) + '</th><th class="hide-narrow">' + esc(t("Ended")) + '</th><th class="num">' +
      esc(t("Duration")) + '</th><th class="num">' + esc(t("Saves")) + "</th><th>" + esc(t("Ended by")) + "</th></tr></thead><tbody>";
    for (const r of rows) {
      h += "<tr><td>" + esc(r.user) + "</td><td>" + projectLink(r.project) + "</td><td>" + esc(dateTime(r.started)) + '</td><td class="hide-narrow">' + esc(dateTime(r.ended)) + '</td><td class="num">' +
        duration((r.ended || r.last_seen) - r.started) + '</td><td class="num">' + r.saves + "</td><td>" + esc(t(SESSION_ENDS[r.end] || r.end || "")) + "</td></tr>";
    }
    if (!rows.length) h += '<tr><td colspan="7" class="muted">' + esc(t("No ended sessions")) + "</td></tr>";
    document.getElementById("session-history").innerHTML = h + "</tbody></table>";
  };
  document.getElementById("session-filter").oninput = e => { sessionFilter = e.target.value; drawHistory(); };
  document.getElementById("sessions-refresh").onclick = () => adminSessions();
  drawHistory();
}

// Users

const PROJECT_ROLES = ["viewer", "reviewer", "editor", "manager", "admin"];

async function adminUsers() {
  const users = await api("/users");
  users.sort((a, b) => a.name.localeCompare(b.name));
  const me = session && session.user ? session.user.name.toLowerCase() : "";
  let html = '<div id="user-form"></div><div class="toolbar"><strong>' + tf("{} users", users.length) + '</strong><button class="primary" id="new-user">' + esc(t("New user")) + "</button></div>";
  html += '<div class="table-wrap"><table class="plain"><thead><tr><th>' + esc(t("User name")) + "</th><th>" + esc(t("Role")) + '</th><th class="hide-narrow">' + esc(t("Projects")) + "</th><th>" + esc(t("Groups")) +
    '</th><th class="hide-narrow">' + esc(t("Directory")) + "</th><th></th></tr></thead><tbody>";
  users.forEach((u, i) => {
    html += "<tr><td><strong>" + esc(u.name) + "</strong>" + (u.name.toLowerCase() === me ? ' <span class="muted">(' + esc(t("you")) + ")</span>" : "") + "</td><td>" + esc(userRole(u) || "–") +
      '</td><td class="hide-narrow">' + esc((u.projects || []).join(", ") || (u.role ? t("All projects") : "")) + "</td><td>" + esc((u.groups || []).join(", ")) + '</td><td class="hide-narrow">' + esc(u.provider || t("Local")) +
      '</td><td class="actions"><button class="small" data-edit="' + i + '">' + esc(t("Edit")) + '</button><button class="small danger" data-del="' + i + '">' + esc(t("Delete")) + "</button></td></tr>";
  });
  if (!users.length) html += '<tr><td colspan="6" class="muted">' + esc(t("No users")) + "</td></tr>";
  adminBody(html + '</tbody></table></div><p id="users-msg"></p>');
  document.getElementById("new-user").onclick = () => userForm(null, users);
  document.querySelectorAll("[data-edit]").forEach(b => b.onclick = () => userForm(users[+b.dataset.edit], users));
  document.querySelectorAll("[data-del]").forEach(b => b.onclick = async () => {
    const u = users[+b.dataset.del];
    if (!confirm(tp("Delete user {}? Their open projects are closed.", u.name))) return;
    try { await api("/users/" + encodeURIComponent(u.name), { method: "DELETE" }); await adminUsers(); refreshSession(); }
    catch (e) { message("users-msg", e.status === 409 ? t("The last administrator cannot lose the right to manage users") : e.message); }
  });
}

/// The form of a new (`u` null) or an existing user.
function userForm(u, users) {
  const directory = u && u.provider;
  const list = values => values.split(",").map(v => v.trim()).filter(Boolean);
  let html = '<form id="uf" class="box section"><h3>' + (u ? tf("Edit user {}", u.name) : esc(t("New user"))) + '</h3><div class="form">';
  html += field("User name", '<input id="uf-name" required autocomplete="off"' + (u ? ' value="' + esc(u.name) + '" disabled' : "") + ">");
  html += field("Role", '<select id="uf-role"><option value="">' + esc(t("None (only grants)")) + "</option>" + options(PROJECT_ROLES, u ? u.role || "" : "viewer", roleTitle) + "</select>",
    t("Viewers open projects read only, reviewers also save reviews, editors change and create projects, managers also delete them and give rights on them, administrators manage the server."));
  html += field("Projects", '<input id="uf-projects" autocomplete="off" value="' + esc(u ? (u.projects || []).join(", ") : "") + '" placeholder="' + esc(t("All projects")) + '">', t("The projects the role applies to (* and ? as wildcards), separated by commas; empty: all"));
  html += field("Groups", '<input id="uf-groups" autocomplete="off" value="' + esc(u ? (u.groups || []).join(", ") : "") + '" placeholder="' + esc(t("separated by commas")) + '"' + (directory ? " disabled" : "") + ">",
    directory ? t("The directory checks the password and sets the groups at every sign-in") : t("Grants of the server's access settings can refer to them"));
  if (!directory) {
    html += field(u ? "New password" : "Password", '<input id="uf-pw" type="password" autocomplete="new-password"' + (u ? ' placeholder="' + esc(t("unchanged if empty")) + '"' : " required") + ">", tf("At least {} characters", 8));
    html += field("Repeat password", '<input id="uf-pw2" type="password" autocomplete="new-password">');
  }
  html += '</div><div class="buttons"><button class="primary">' + esc(t("Save")) + '</button><button type="button" id="uf-cancel">' + esc(t("Cancel")) + '</button><span id="uf-msg"></span></div></form>';
  const box = document.getElementById("user-form");
  box.innerHTML = html;
  box.scrollIntoView({ block: "nearest" });
  const el = id => document.getElementById(id);
  (u ? el("uf-role") : el("uf-name")).focus();
  el("uf-cancel").onclick = () => { box.innerHTML = ""; };
  el("uf").onsubmit = async e => {
    e.preventDefault();
    const name = u ? u.name : el("uf-name").value.trim();
    if (!name || /\s/.test(name)) { message("uf-msg", t("A user name must not be empty or contain blanks")); return; }
    const password = directory ? "" : el("uf-pw").value;
    if (!directory && password !== el("uf-pw2").value) { message("uf-msg", t("The passwords do not match")); return; }
    if (password && password.length < 8) { message("uf-msg", tp("The password needs at least {} characters", 8)); return; }
    if (!u && users.some(x => x.name.toLowerCase() === name.toLowerCase()) && !confirm(tp("User {} exists already. Change it?", name))) return;
    const update = { role: el("uf-role").value || null, projects: list(el("uf-projects").value) };
    if (!directory) update.groups = list(el("uf-groups").value);
    if (password) update.password = password;
    try {
      await api("/users/" + encodeURIComponent(name), { method: "PUT", json: update });
      await adminUsers();
      message("users-msg", tp("Saved user {}", name), true);
      refreshSession();
    } catch (err) { message("uf-msg", err.status === 409 ? t("The last administrator cannot lose the right to manage users") : err.message); }
  };
}
