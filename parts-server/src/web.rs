//! Web page of the data portal (dashboard, search with facets, part details and the
//! part form, files, favorites, administration of the server) and its translations. The page is plain HTML / JavaScript on top of the API and of the
//! scripts the servers share (`openecraft_web`); its texts are written in English and
//! translated through [`texts`].

use axum::extract::Query;
use axum::response::IntoResponse;
use axum::Json;
use openecraft_i18n::{tr_const, Lang};
use openecraft_parts::portal::{AttachmentKind, ChangeKind, Lifecycle, PartsRight, Role, Sort};
use openecraft_parts::{productgroups, AccessoryMount, FnKind, TopGroup, PRODUCT_GROUPS};
use openecraft_web::{TextQuery, WebTexts};

const PAGE: &str = include_str!("web/portal.html");
const SCRIPT: &str = include_str!("web/portal.js");
const ADMIN_SCRIPT: &str = include_str!("web/admin.js");
const EDIT_SCRIPT: &str = include_str!("web/edit.js");
const DASHBOARD_SCRIPT: &str = include_str!("web/dashboard.js");

/// Texts of the page besides those of the shared scripts (`{}`: placeholders filled in
/// by the page).
const TEXTS: &[&str] = &[
    tr_const!("Graphics"),
    tr_const!("Front view (2D)"),
    tr_const!("3D model"),
    tr_const!("Drag to turn the model"),
    tr_const!("OpenECraft data portal"),
    tr_const!("Search parts"),
    tr_const!("Search"),
    tr_const!("Favorites"),
    tr_const!("All parts"),
    tr_const!("Catalogs"),
    tr_const!("Product top group"),
    tr_const!("Product group"),
    tr_const!("Product subgroup"),
    tr_const!("Manufacturer"),
    tr_const!("Catalog"),
    tr_const!("Lifecycle"),
    tr_const!("Technical data"),
    tr_const!("Clear filters"),
    tr_const!("{} parts"),
    tr_const!("No parts found"),
    tr_const!("Part number"),
    tr_const!("Price"),
    tr_const!("Sort by"),
    tr_const!("Descending"),
    tr_const!("Previous"),
    tr_const!("Next"),
    tr_const!("Page {} of {}"),
    tr_const!("Back to the results"),
    tr_const!("Type number"),
    tr_const!("Order number"),
    tr_const!("Supplier"),
    tr_const!("Note"),
    tr_const!("Voltage"),
    tr_const!("Current"),
    tr_const!("Power"),
    tr_const!("Part {} not found"),
    tr_const!("Breaking capacity"),
    tr_const!("Poles"),
    tr_const!("Cross-section"),
    tr_const!("Power loss"),
    tr_const!("Rated current"),
    tr_const!("Characteristic"),
    tr_const!("Coil voltage"),
    tr_const!("Certificates"),
    tr_const!("Datasheet"),
    tr_const!("Width"),
    tr_const!("Height"),
    tr_const!("Depth"),
    tr_const!("Weight"),
    tr_const!("Mounting"),
    tr_const!("Functions"),
    tr_const!("Accessories"),
    tr_const!("Accessory"),
    tr_const!("Fits"),
    tr_const!("Yes"),
    tr_const!("Successor"),
    tr_const!("Replaced by {}"),
    tr_const!("Files"),
    tr_const!("History"),
    tr_const!("Revision {}"),
    tr_const!("Last change"),
    tr_const!("Add to favorites"),
    tr_const!("Remove from favorites"),
    tr_const!("Sign in to keep favorites"),
    tr_const!("Version"),
    tr_const!("Parts"),
    tr_const!("Show parts"),
    tr_const!("Export (CSV)"),
    tr_const!("The server cannot be reached"),
    tr_const!("General"),
    tr_const!("Commercial data"),
    tr_const!("Mounting data"),
    tr_const!("Power loss"),
    tr_const!("Clearance top"),
    tr_const!("Clearance bottom"),
    tr_const!("Mounting holes"),
    tr_const!("Door cutout"),
    tr_const!("Cutout"),
    tr_const!("{} parts in {} catalogs, revision {}"),
    // Administration (admin.js)
    tr_const!("No"),
    tr_const!("Awaiting approval: {}"),
    tr_const!("Awaiting approval"),
    tr_const!("Approve"),
    tr_const!("Reject"),
    tr_const!("Approved user {}"),
    tr_const!("Reject the registration of {}? The account is deleted."),
    tr_const!("New accounts need the approval of an administrator"),
    tr_const!("Waiting accounts cannot sign in; approve or reject them in Users."),
    tr_const!("Approval of new accounts"),
    tr_const!("Registration"),
    tr_const!("Users may register themselves (as readers)"),
    tr_const!("Set by the start options of the server (--registration or OPENECRAFT_REGISTRATION)."),
    tr_const!("Registered users become readers: they keep favorites and read the parts even when reading needs a sign-in. Give them more rights in Users."),
    tr_const!("The registration is open"),
    tr_const!("Export the parts, catalogs and settings, or download a backup of the whole database."),
    tr_const!("All parts with their data, for spreadsheets or to import them into another server. Their files are only in the database backup."),
    tr_const!("The catalogs with their manufacturer, version and description."),
    tr_const!("The users with their role, manufacturer, groups and last sign-in, without passwords."),
    tr_const!("Restore from a file"),
    tr_const!("Database backup"),
    tr_const!("One SQLite file with all parts, their history and files, the catalogs, the users and their favorites. It holds the password hashes of the users: keep it safe."),
    tr_const!("Download"),
    tr_const!("To restore it, start the server with --data <file>, or copy it into a new database with --migrate-from <file>."),
    tr_const!("The file holds no access settings"),
    tr_const!("Replace the roles and grants with those of the file?"),
    tr_const!("Export and backup"),
    tr_const!("Sign in as an administrator to manage the server"),
    tr_const!("Server"),
    tr_const!("Changes allowed"),
    tr_const!("Read only"),
    tr_const!("Users"),
    tr_const!("Access rights"),
    tr_const!("Sign-in needed to read"),
    tr_const!("All manufacturers"),
    tr_const!("Search users"),
    tr_const!("New user"),
    tr_const!("Directory"),
    tr_const!("Last sign-in"),
    tr_const!("Open sessions: {}"),
    tr_const!("Never"),
    tr_const!("you"),
    tr_const!("Local"),
    tr_const!("Sign out everywhere"),
    tr_const!("No users"),
    tr_const!("Sign out {} on all devices?"),
    tr_const!("Delete user {}? Their sessions and favorites are removed."),
    tr_const!("Edit user {}"),
    tr_const!("Readers keep favorites, manufacturers maintain the parts and catalogs of their manufacturer, editors all parts, administrators also the users."),
    tr_const!("At least {} characters"),
    tr_const!("Grants of the server's access settings can refer to them"),
    tr_const!("The directory checks the password and sets the groups at every sign-in"),
    tr_const!("A manufacturer user needs a manufacturer"),
    tr_const!("Saved user {}"),
    tr_const!("The last administrator cannot lose the right to manage users"),
    tr_const!("User {} exists already. Change it?"),
    tr_const!("A role is a set of rights. Grants give a user or a group a role on the parts of a manufacturer (* and ? as wildcards, * for all); a deny grant takes the rights of the role away, whatever other grants give."),
    tr_const!("New catalog"),
    tr_const!("Only empty catalogs can be deleted"),
    tr_const!("Bundled catalogs"),
    tr_const!("Catalogs shipped with the server. Installing adds their parts and updates those nobody changed on this server."),
    tr_const!("On this server"),
    tr_const!("Delete catalog {}?"),
    tr_const!("{}: {} parts added, {} updated, {} kept"),
    tr_const!("Edit catalog {}"),
    tr_const!("A catalog id must not be empty or contain blanks"),
    tr_const!("Catalog {} exists already. Change it?"),
    tr_const!("Saved catalog {}"),
    tr_const!("Import parts"),
    tr_const!("CSV (with the column names of the export), JSON, BMEcat or EPLAN XML. Parts with the same number are replaced; their files are kept."),
    tr_const!("Import"),
    tr_const!("Imported {} parts (revision {})"),
    tr_const!("Administration"),
    tr_const!("Overview"),
    tr_const!("Storage"),
    tr_const!("Status"),
    tr_const!("Revision"),
    tr_const!("Signed in"),
    tr_const!("Defined roles"),
    tr_const!("New password"),
    tr_const!("Install"),
    tr_const!("Install again"),
    tr_const!("Update"),
    // Groups, directories, catalog permissions, part form, dashboard
    tr_const!("Per page"),
    tr_const!("New part"),
    tr_const!("Delete part {}?"),
    tr_const!("Dashboard"),
    tr_const!("Show the dashboard with graphs of the parts"),
    tr_const!("Set by the start options of the server (--dashboard or OPENECRAFT_DASHBOARD)."),
    tr_const!("Everybody who may read the parts sees it."),
    tr_const!("The dashboard is shown"),
    tr_const!("The dashboard is hidden"),
    tr_const!("Directory groups"),
    tr_const!("Groups of this server, next to those of the directory"),
    tr_const!("Grants on catalog:<id> apply to one catalog and its parts (Catalogs › Permissions)."),
    tr_const!("Groups gather users, local ones and those of a directory. Grants to a group give its members the rights of a role, on the whole server, a manufacturer or a catalog."),
    tr_const!("New group"),
    tr_const!("Members"),
    tr_const!("No groups"),
    tr_const!("Delete group {}? The grants to it are removed."),
    tr_const!("The groups would leave nobody who manages users"),
    tr_const!("Edit group {}"),
    tr_const!("Add"),
    tr_const!("Users of a directory too, by the name they sign in with"),
    tr_const!("A group name must not be empty or contain ':'"),
    tr_const!("Group {} exists already. Change it?"),
    tr_const!("Saved group {}"),
    tr_const!("Permissions"),
    tr_const!("Permissions of catalog {}"),
    tr_const!("The roles given here apply to this catalog and its parts, whatever their manufacturer; deny takes the rights of the role away. Reading is a right of the whole server."),
    tr_const!("No grants: the rights by manufacturer apply"),
    tr_const!("Without catalog"),
    tr_const!("Edit part {}"),
    tr_const!("A part number must not be empty or start or end with blanks"),
    tr_const!("You may not change parts of manufacturer {}"),
    tr_const!("Part {} exists already"),
    tr_const!("Currency"),
    tr_const!("Price unit"),
    tr_const!("Quantity unit"),
    tr_const!("One line per value, e.g. Coil voltage: 24 V DC"),
    tr_const!("Back to the part"),
    tr_const!("Others"),
    tr_const!("Parts per manufacturer"),
    tr_const!("Parts per product group"),
    tr_const!("Parts per catalog"),
    tr_const!("Last changes per month"),
    tr_const!("Parts with …"),
    tr_const!("Users per role"),
    tr_const!("Changed last"),
    tr_const!("Dimensions"),
    tr_const!("Manufacturers"),
    tr_const!("Size of the files"),
    tr_const!("Logo"),
    tr_const!("Remove the logo"),
    tr_const!("The logo of the company: PNG, JPEG, GIF, WebP or SVG, at most 512 kB"),
    tr_const!("The roles, the grants, the groups and the access without sign-in, with the version of the server. Restoring replaces the roles and grants and adds the groups; the directories stay as set up on the server."),
];

pub async fn index() -> impl IntoResponse {
    openecraft_web::page(PAGE)
}

pub async fn script() -> impl IntoResponse {
    openecraft_web::script(SCRIPT)
}

pub async fn admin_script() -> impl IntoResponse {
    openecraft_web::script(ADMIN_SCRIPT)
}

pub async fn edit_script() -> impl IntoResponse {
    openecraft_web::script(EDIT_SCRIPT)
}

pub async fn dashboard_script() -> impl IntoResponse {
    openecraft_web::script(DASHBOARD_SCRIPT)
}

pub fn web_texts(lang: Lang) -> WebTexts {
    let own_groups = PRODUCT_GROUPS.iter().flat_map(|(g, subs)| std::iter::once(*g).chain(subs.iter().copied()));
    let group_names = productgroups::GROUPS.iter().chain(productgroups::SUBGROUPS).map(|(_, n)| *n).chain(own_groups);
    let role_title = |name: &str| Role::ALL.iter().find(|r| r.name() == name).map_or("", |r| r.title());
    let mut texts = WebTexts::new::<PartsRight>(lang, TEXTS.iter().copied().chain(group_names), role_title);
    for t in TopGroup::ALL {
        texts.key("top_group", format!("{t:?}"), t.title());
    }
    for l in Lifecycle::ALL {
        texts.key("lifecycle", format!("{l:?}"), l.title());
    }
    for k in AttachmentKind::ALL {
        texts.key("kind", format!("{k:?}"), k.title());
    }
    for s in Sort::ALL {
        texts.key("sort", format!("{s:?}"), s.title());
    }
    for r in Role::ALL {
        texts.key("role", format!("{r:?}"), r.title());
    }
    for c in [ChangeKind::Created, ChangeKind::Changed, ChangeKind::Deleted] {
        texts.key("change", format!("{c:?}"), c.title());
    }
    for f in FnKind::ALL {
        texts.key("fn", format!("{f:?}"), f.title());
    }
    for m in AccessoryMount::ALL {
        texts.key("mount", format!("{m:?}"), m.title());
    }
    texts
}

/// `GET /api/v1/texts?lang=de`: the page's texts in a language (English if unknown).
pub async fn texts(Query(q): Query<TextQuery>) -> Json<WebTexts> {
    Json(web_texts(q.lang()))
}

#[cfg(test)]
mod tests;
