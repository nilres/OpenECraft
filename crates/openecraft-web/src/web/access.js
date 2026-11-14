"use strict";
/// The administration of the access settings the servers share: the access without
/// sign-in, the roles and the grants (`adminAccess`) and the directories users sign in
/// with (`adminDirectories`). The page script describes its scopes in `ACCESS`:
/// `help()` (paragraphs), `closed()` (no access without sign-in), `scope()` (title of
/// the scope column), `all()` (the scope `*`), `scopes()` (suggestions) and,
/// optionally, `before(box)` (settings above).

async function adminAccess() {
  const [s, users, scopes] = await Promise.all([api("/access"), api("/users"), ACCESS.scopes()]);
  policy = Object.assign({ anonymous: "", roles: [], grants: [] }, s.policy);
  providers = s.providers || [];
  const groups = (s.groups || []).map(g => g.name).concat(users.flatMap(u => u.groups || []));
  adminBody('<div id="access-before"></div><div id="access"></div>' + datalist("subjects-user", users.map(u => u.name)) + datalist("subjects-group", groups) +
    datalist("scopes", ["*"].concat(scopes)));
  if (ACCESS.before) await ACCESS.before(document.getElementById("access-before"));
  renderAccess();
}

function renderAccess() {
  const rights = T.rights || [];
  const check = (on, attrs) => '<input type="checkbox"' + (on ? " checked" : "") + " " + attrs + ">";
  let html = '<p class="muted">' + ACCESS.help().map(esc).join(" ") + "</p>";
  html += '<div class="form"><label for="anon">' + esc(t("Without sign-in")) + '</label><div><select id="anon"><option value="">' + esc(ACCESS.closed()) + "</option>" +
    options(roleNames(), policy.anonymous, roleTitle) + "</select></div></div>";

  html += '<h3 class="section">' + esc(t("Roles")) + '</h3><div class="table-wrap"><table class="plain"><thead><tr><th>' + esc(t("Role")) + '</th><th class="hide-narrow">' + esc(t("Description")) + "</th>" +
    rights.map(r => '<th class="check">' + esc(r.title) + "</th>").join("") + "<th></th></tr></thead><tbody>";
  for (const b of T.roles || []) {
    html += "<tr><td>" + esc(b.title) + '</td><td class="hide-narrow muted">' + esc(t("Built-in")) + "</td>" + rights.map(r => '<td class="check">' + check(b.rights.includes(r.name), "disabled") + "</td>").join("") + "<td></td></tr>";
  }
  policy.roles.forEach((r, i) => {
    html += '<tr><td><input data-role="' + i + '" data-f="name" value="' + esc(r.name) + '" placeholder="' + esc(t("Name")) + '"></td><td class="hide-narrow"><input data-role="' + i + '" data-f="description" value="' + esc(r.description || "") + '"></td>' +
      rights.map(x => '<td class="check">' + check((r.rights || []).includes(x.name), 'data-role="' + i + '" data-right="' + x.name + '" aria-label="' + esc(x.title) + '"') + "</td>").join("") +
      '<td><button type="button" class="small danger" data-del-role="' + i + '" title="' + esc(t("Remove")) + '">✕</button></td></tr>';
  });
  html += '</tbody></table></div><div class="buttons"><button type="button" id="add-role">' + esc(t("New role")) + "</button></div>";

  html += '<h3 class="section">' + esc(t("Grants")) + '</h3><div class="table-wrap"><table class="plain"><thead><tr><th>' + esc(t("Type")) + "</th><th>" + esc(t("User or group")) + "</th><th>" + esc(t("Role")) + "</th><th>" +
    esc(ACCESS.scope()) + '</th><th class="check">' + esc(t("Deny")) + "</th><th></th></tr></thead><tbody>";
  const names = roleNames();
  policy.grants.forEach((g, i) => {
    const group = g.subject.group !== undefined;
    const who = group ? g.subject.group : g.subject.user || "";
    const roles = names.includes(g.role) ? names : names.concat([g.role]);
    html += '<tr><td><select data-grant="' + i + '" data-f="kind"><option value="user">' + esc(t("User")) + '</option><option value="group"' + (group ? " selected" : "") + ">" + esc(t("Group")) + "</option></select></td>" +
      '<td><input data-grant="' + i + '" data-f="who" list="subjects-' + (group ? "group" : "user") + '" value="' + esc(who) + '"' + (group ? ' placeholder="' + esc(t("directory:group")) + '"' : "") + "></td>" +
      '<td><select data-grant="' + i + '" data-f="role">' + options(roles, g.role, roleTitle) + "</select></td>" +
      '<td><input data-grant="' + i + '" data-f="scope" list="scopes" value="' + esc(g.scope || "") + '" placeholder="* (' + esc(ACCESS.all()) + ')"></td>' +
      '<td class="check">' + check(g.deny, 'data-grant="' + i + '" data-f="deny" aria-label="' + esc(t("Deny")) + '"') + "</td>" +
      '<td><button type="button" class="small danger" data-del-grant="' + i + '" title="' + esc(t("Remove")) + '">✕</button></td></tr>';
  });
  if (!policy.grants.length) html += '<tr><td colspan="6" class="muted">' + esc(t("No grants: users have the rights of their role")) + "</td></tr>";
  html += '</tbody></table></div><div class="buttons"><button type="button" id="add-grant">' + esc(t("Add grant")) + "</button></div>";

  html += '<div class="buttons sticky"><button class="primary" id="save-access">' + esc(t("Save")) + '</button><button type="button" id="reload-access">' + esc(t("Discard changes")) + '</button><span id="access-msg"></span></div>';
  document.getElementById("access").innerHTML = html;

  const all = sel => document.querySelectorAll(sel);
  document.getElementById("anon").onchange = e => { policy.anonymous = e.target.value; };
  all("[data-role]").forEach(e => e.oninput = e.onchange = ev => {
    const r = policy.roles[+e.dataset.role];
    if (e.dataset.right) r.rights = rights.map(x => x.name).filter(n => n === e.dataset.right ? e.checked : (r.rights || []).includes(n));
    else r[e.dataset.f] = e.value;
    // The roles of the selections follow the names.
    if (e.dataset.f === "name" && ev.type === "change") renderAccess();
  });
  all("[data-grant]").forEach(e => e.onchange = e.oninput = ev => {
    const g = policy.grants[+e.dataset.grant];
    const f = e.dataset.f;
    if (f === "kind") { const who = g.subject.user ?? g.subject.group ?? ""; g.subject = e.value === "group" ? { group: who } : { user: who }; if (ev.type === "change") renderAccess(); }
    else if (f === "who") g.subject = g.subject.group !== undefined ? { group: e.value } : { user: e.value };
    else if (f === "deny") g.deny = e.checked;
    else g[f] = e.value;
  });
  // New roles and grants start with the lowest built-in role.
  const lowest = (T.roles || [])[0] || { name: "", rights: [] };
  all("[data-del-role]").forEach(b => b.onclick = () => { policy.roles.splice(+b.dataset.delRole, 1); renderAccess(); });
  all("[data-del-grant]").forEach(b => b.onclick = () => { policy.grants.splice(+b.dataset.delGrant, 1); renderAccess(); });
  document.getElementById("add-role").onclick = () => { policy.roles.push({ name: "", description: "", rights: lowest.rights.slice() }); renderAccess(); focusLast("[data-f=name][data-role]"); };
  document.getElementById("add-grant").onclick = () => { policy.grants.push({ subject: { user: "" }, role: lowest.name, scope: "", deny: false }); renderAccess(); focusLast("[data-f=who]"); };
  document.getElementById("reload-access").onclick = () => adminAccess();
  document.getElementById("save-access").onclick = async () => {
    const p = { anonymous: policy.anonymous, roles: policy.roles.map(r => ({ name: r.name.trim(), description: (r.description || "").trim(), rights: r.rights || [] })),
      grants: policy.grants.map(g => Object.assign({}, g, { scope: (g.scope || "").trim() })) };
    try {
      const s = await api("/access", { method: "PUT", json: p });
      policy = Object.assign({ anonymous: "", roles: [], grants: [] }, s.policy);
      providers = s.providers || [];
      renderAccess();
      message("access-msg", t("Access settings saved"), true);
      refreshSession();
    } catch (e) { message("access-msg", e.status === 409 ? t("The access settings would leave nobody who manages users") : e.message); }
  };
}

/// The grants on one scope (a catalog, a project) in `box`, `grants` as loaded. `o`:
/// `title` (HTML), `help`, `empty` (without grants), `roles` (to choose from, default
/// all), `role` (of a new grant), `scope`, `users` and `groups` (suggestions) and
/// `save(list)`, which stores the grants and gives them back.
function scopeGrants(box, grants, o) {
  const render = () => {
    const names = o.roles || roleNames();
    let html = '<div class="box section"><h3>' + o.title + '</h3><p class="muted">' + esc(o.help) + "</p>";
    html += '<div class="table-wrap"><table class="plain"><thead><tr><th>' + esc(t("Type")) + "</th><th>" + esc(t("User or group")) + "</th><th>" + esc(t("Role")) + '</th><th class="check">' + esc(t("Deny")) + "</th><th></th></tr></thead><tbody>";
    grants.forEach((g, i) => {
      const group = g.subject.group !== undefined;
      const roles = names.includes(g.role) ? names : names.concat([g.role]);
      html += '<tr><td><select data-cg="' + i + '" data-f="kind"><option value="user">' + esc(t("User")) + '</option><option value="group"' + (group ? " selected" : "") + ">" + esc(t("Group")) + "</option></select></td>" +
        '<td><input data-cg="' + i + '" data-f="who" list="cg-' + (group ? "groups" : "users") + '" value="' + esc(group ? g.subject.group : g.subject.user || "") + '"' + (group ? ' placeholder="' + esc(t("directory:group")) + '"' : "") + "></td>" +
        '<td><select data-cg="' + i + '" data-f="role">' + options(roles, g.role, roleTitle) + "</select></td>" +
        '<td class="check"><input type="checkbox" data-cg="' + i + '" data-f="deny"' + (g.deny ? " checked" : "") + ' aria-label="' + esc(t("Deny")) + '"></td>' +
        '<td><button type="button" class="small danger" data-cg-del="' + i + '" title="' + esc(t("Remove")) + '">✕</button></td></tr>';
    });
    if (!grants.length) html += '<tr><td colspan="5" class="muted">' + esc(o.empty) + "</td></tr>";
    html += '</tbody></table></div><div class="buttons"><button type="button" id="cg-add">' + esc(t("Add grant")) + '</button><button type="button" class="primary" id="cg-save">' + esc(t("Save")) + '</button><button type="button" id="cg-cancel">' + esc(t("Cancel")) + '</button><span id="cg-msg"></span></div></div>' +
      datalist("cg-users", o.users || []) + datalist("cg-groups", o.groups || []);
    box.innerHTML = html;
    box.scrollIntoView({ block: "nearest" });
    box.querySelectorAll("[data-cg]").forEach(e => e.onchange = e.oninput = ev => {
      const g = grants[+e.dataset.cg];
      const f = e.dataset.f;
      if (f === "kind") { const who = g.subject.user ?? g.subject.group ?? ""; g.subject = e.value === "group" ? { group: who } : { user: who }; if (ev.type === "change") render(); }
      else if (f === "who") g.subject = g.subject.group !== undefined ? { group: e.value } : { user: e.value };
      else if (f === "deny") g.deny = e.checked;
      else g[f] = e.value;
    });
    box.querySelectorAll("[data-cg-del]").forEach(b => b.onclick = () => { grants.splice(+b.dataset.cgDel, 1); render(); });
    document.getElementById("cg-add").onclick = () => { grants.push({ subject: { group: "" }, role: o.role, scope: o.scope, deny: false }); render(); focusLast("[data-cg][data-f=who]"); };
    document.getElementById("cg-cancel").onclick = () => { box.innerHTML = ""; };
    document.getElementById("cg-save").onclick = async () => {
      const list = grants.map(g => Object.assign({}, g, { scope: o.scope }));
      if (list.some(g => !(g.subject.user ?? g.subject.group ?? "").trim())) { message("cg-msg", t("A grant needs a user or a group")); return; }
      try {
        grants = await o.save(list);
        render();
        message("cg-msg", t("Access settings saved"), true);
        refreshSession();
      } catch (e) { message("cg-msg", e.message); }
    };
  };
  render();
}

// Directories (Active Directory, LDAP)

const DIRECTORY_KINDS = [["active_directory", "Active Directory"], ["ldap", "LDAP"]];

async function adminDirectories() {
  const list = await api("/directories");
  let html = '<div id="directory-form"></div><p class="muted">' + esc(t("Users sign in with the account of a directory (Active Directory, LDAP) when the server has no local account of their name. Grants refer to the groups of a directory as directory:group.")) + "</p>" +
    '<div class="toolbar"><strong>' + esc(t("Directories")) + '</strong><button class="primary" id="new-directory">' + esc(t("New directory")) + "</button></div>";
  html += '<div class="table-wrap"><table class="plain"><thead><tr><th>' + esc(t("Name")) + "</th><th>" + esc(t("Type")) + '</th><th class="hide-narrow">' + esc(t("Address")) + '</th><th class="hide-narrow">' + esc(t("Search base")) + "</th><th></th></tr></thead><tbody>";
  list.forEach((d, i) => {
    html += "<tr><td><strong>" + esc(d.title || d.id) + '</strong> <span class="muted">' + esc(d.id) + "</span></td><td>" + esc((DIRECTORY_KINDS.find(k => k[0] === d.kind) || [d.kind, d.kind])[1]) +
      '</td><td class="hide-narrow">' + esc(d.url) + '</td><td class="hide-narrow">' + esc(d.base_dn) + '</td><td class="actions"><button class="small" data-edit="' + i + '">' + esc(t("Edit")) +
      '</button><button class="small danger" data-del="' + i + '">' + esc(t("Delete")) + "</button></td></tr>";
  });
  if (!list.length) html += '<tr><td colspan="5" class="muted">' + esc(t("Only local accounts")) + "</td></tr>";
  adminBody(html + '</tbody></table></div><p id="directories-msg"></p>');
  document.getElementById("new-directory").onclick = () => directoryForm(null, list);
  document.querySelectorAll("[data-edit]").forEach(b => b.onclick = () => directoryForm(list[+b.dataset.edit], list));
  document.querySelectorAll("[data-del]").forEach(b => b.onclick = async () => {
    const d = list[+b.dataset.del];
    if (!confirm(tp("Delete directory {}? Its users can no longer sign in.", d.title || d.id))) return;
    try { await api("/directories/" + encodeURIComponent(d.id), { method: "DELETE" }); await adminDirectories(); }
    catch (e) { message("directories-msg", e.message); }
  });
}

/// The settings of a new (`d` null) or an existing directory, with a test of the
/// connection and of a sign-in.
function directoryForm(d, list) {
  const v = k => esc(d ? d[k] ?? "" : "");
  const check = (id, on, label) => '<label class="check"><input type="checkbox" id="' + id + '"' + (on ? " checked" : "") + "> " + esc(t(label)) + "</label>";
  let html = '<form id="df" class="box section"><h3>' + (d ? tf("Edit directory {}", d.title || d.id) : esc(t("New directory"))) + '</h3><div class="form">';
  html += field("Identifier", '<input id="df-id" required autocomplete="off" value="' + v("id") + '"' + (d ? " disabled" : "") + ">", t("Short name without blanks; qualifies the groups (id:group)"));
  html += field("Name", '<input id="df-title" value="' + v("title") + '">');
  html += field("Type", '<select id="df-kind">' + DIRECTORY_KINDS.map(([k, n]) => '<option value="' + k + '"' + ((d ? d.kind : "active_directory") === k ? " selected" : "") + ">" + n + "</option>").join("") + "</select>");
  html += field("Address", '<input id="df-url" required placeholder="ldaps://dc1.corp.example" value="' + v("url") + '">', t("ldap:// or ldaps://, with the port if it is not the usual one"));
  html += labelFor("Encryption", '<span id="df-tls"></span>') + '<div id="df-tls">' + check("df-starttls", d && d.starttls, "StartTLS (for ldap://)") + "<br>" +
    check("df-insecure", d && d.insecure_tls, "Accept any certificate (test installations only)") + "</div>";
  html += field("Service account", '<input id="df-bind-dn" autocomplete="off" placeholder="CN=svc-openecraft,OU=Service,DC=corp,DC=example" value="' + v("bind_dn") + '">', t("Looks the users up; empty: users bind with their own name (below)"));
  html += field("Password", '<input id="df-bind-pw" type="password" autocomplete="new-password" placeholder="' + esc(d && d.password_set ? t("unchanged if empty") : "") + '">',
    t("Better kept in an environment variable of the server"));
  html += field("Password variable", '<input id="df-bind-env" autocomplete="off" placeholder="OPENECRAFT_LDAP_PASSWORD" value="' + v("bind_password_env") + '">', t("Environment variable with the password of the service account"));
  html += field("User bind", '<input id="df-user-bind" autocomplete="off" placeholder="{user}@corp.example" value="' + v("user_bind") + '">', t("Bind name of a user without service account; {user} is replaced"));
  html += field("Search base", '<input id="df-base" required placeholder="DC=corp,DC=example" value="' + v("base_dn") + '">');
  html += field("User filter", '<input id="df-filter" autocomplete="off" placeholder="(&(objectClass=user)(sAMAccountName={user}))" value="' + v("user_filter") + '">', t("Empty: the usual filter of the type"));
  html += field("Name attribute", '<input id="df-name-attr" autocomplete="off" placeholder="sAMAccountName / uid" value="' + v("name_attribute") + '">');
  html += labelFor("Groups", '<span id="df-nested"></span>') + '<div id="df-nested">' + check("df-nested-groups", d && d.nested_groups, "Also groups of groups (Active Directory)") + "</div>";
  html += field("Group filter", '<input id="df-group-filter" autocomplete="off" placeholder="(&(objectClass=groupOfNames)(member={dn}))" value="' + v("group_filter") + '">', t("Empty: the memberOf attribute of the user"));
  html += field("Group search base", '<input id="df-group-base" autocomplete="off" value="' + v("group_base_dn") + '">', t("Empty: the search base"));
  html += field("Allowed groups", '<input id="df-allowed" autocomplete="off" value="' + esc(d ? (d.allowed_groups || []).join(", ") : "") + '" placeholder="' + esc(t("separated by commas")) + '">', t("Only their members sign in; empty: all users"));
  html += field("Timeout (s)", '<input id="df-timeout" type="number" min="1" max="120" value="' + esc(d ? d.timeout_secs || 10 : 10) + '">');
  html += labelFor("Test", '<input id="df-test-user">') + '<div><div class="directory-test"><input id="df-test-user" autocomplete="off" placeholder="' + esc(t("User name")) + '"><input id="df-test-pw" type="password" autocomplete="off" placeholder="' + esc(t("Password")) + '">' +
    '<button type="button" id="df-test">' + esc(t("Test")) + '</button></div><div class="hint">' + esc(t("Without a user: connects and binds with the service account. With a user: signs them in and shows their groups.")) + '</div><div id="df-test-msg"></div></div>';
  html += '</div><div class="buttons"><button class="primary">' + esc(t("Save")) + '</button><button type="button" id="df-cancel">' + esc(t("Cancel")) + '</button><span id="df-msg"></span></div></form>';
  const box = document.getElementById("directory-form");
  box.innerHTML = html;
  box.scrollIntoView({ block: "nearest" });
  const el = id => document.getElementById(id);
  (d ? el("df-title") : el("df-id")).focus();
  el("df-cancel").onclick = () => { box.innerHTML = ""; };
  const settings = () => ({
    id: d ? d.id : el("df-id").value.trim(), title: el("df-title").value.trim(), kind: el("df-kind").value, url: el("df-url").value.trim(),
    starttls: el("df-starttls").checked, insecure_tls: el("df-insecure").checked, bind_dn: el("df-bind-dn").value.trim(), bind_password: el("df-bind-pw").value,
    bind_password_env: el("df-bind-env").value.trim(), user_bind: el("df-user-bind").value.trim(), base_dn: el("df-base").value.trim(), user_filter: el("df-filter").value.trim(),
    name_attribute: el("df-name-attr").value.trim(), nested_groups: el("df-nested-groups").checked, group_filter: el("df-group-filter").value.trim(),
    group_base_dn: el("df-group-base").value.trim(), allowed_groups: el("df-allowed").value.split(",").map(g => g.trim()).filter(Boolean),
    timeout_secs: Math.max(1, parseInt(el("df-timeout").value, 10) || 10),
  });
  const valid = s => {
    if (!s.id || /[\s:]/.test(s.id)) return t("A directory id must not be empty or contain blanks or ':'");
    if (!/^ldaps?:\/\//.test(s.url)) return t("The address must start with ldap:// or ldaps://");
    return "";
  };
  el("df-test").onclick = async () => {
    const s = settings();
    const problem = valid(s);
    const out = el("df-test-msg");
    if (problem) { out.className = "error"; out.textContent = problem; return; }
    el("df-test").disabled = true;
    out.className = "muted";
    out.textContent = t("Loading…");
    try {
      const r = await api("/directories/" + encodeURIComponent(s.id) + "/test", { method: "POST", json: { directory: s, name: el("df-test-user").value.trim(), password: el("df-test-pw").value } });
      out.className = r.ok ? "ok" : "error";
      out.textContent = (r.ok ? t("The test succeeded") : t("The test failed")) + ": " + r.message + (r.identity && (r.identity.groups || []).length ? " · " + r.identity.groups.join(", ") : "");
    } catch (e) { out.className = "error"; out.textContent = e.message; }
    el("df-test").disabled = false;
  };
  el("df").onsubmit = async e => {
    e.preventDefault();
    const s = settings();
    const problem = valid(s);
    if (problem) { message("df-msg", problem); return; }
    if (!d && list.some(x => x.id.toLowerCase() === s.id.toLowerCase()) && !confirm(tp("Directory {} exists already. Change it?", s.id))) return;
    try {
      await api("/directories/" + encodeURIComponent(s.id), { method: "PUT", json: s });
      await adminDirectories();
      message("directories-msg", tp("Saved directory {}", s.title || s.id), true);
    } catch (err) { message("df-msg", err.message); }
  };
}
