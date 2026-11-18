//! Web page of the project server (`--web`): the projects the user may see, read only
//! (pages by section or structure, devices, the pages as SVG) with their inspection and
//! test report, whose results reviewers and editors enter there (on a tablet or a
//! phone on site), and the administration
//! of the users, the access rights and the directories. Plain HTML / JavaScript on top
//! of the API and of the scripts the servers share (`openecraft_web`); its texts are
//! written in English and translated through [`texts`].

use axum::extract::Query;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use openecraft_core::collab::{ProjectRight, ProjectRole};
use openecraft_i18n::{tr_const, Lang};
use openecraft_web::{TextQuery, WebTexts};

const PAGE: &str = include_str!("web/project.html");
const SCRIPT: &str = include_str!("web/project.js");
const ADMIN_SCRIPT: &str = include_str!("web/admin.js");
const INSPECTION_SCRIPT: &str = include_str!("web/inspection.js");

/// Texts of the page besides those of the shared scripts (`{}`: placeholders filled in
/// by the page).
const TEXTS: &[&str] = &[
    tr_const!("Sign in to see the projects"),
    tr_const!("Project"),
    tr_const!("Pages"),
    tr_const!("Revision"),
    tr_const!("In use by"),
    tr_const!("Last changed by"),
    tr_const!("Last changed"),
    tr_const!("Unknown"),
    tr_const!("Your rights"),
    tr_const!("No projects found"),
    tr_const!("You may not see any project"),
    tr_const!("{} projects"),
    tr_const!("Filter projects…"),
    tr_const!("Project {} not found"),
    tr_const!("Read only"),
    tr_const!("Revision {} on the server"),
    tr_const!("In use by {}"),
    tr_const!("Permissions"),
    tr_const!("All projects"),
    tr_const!("Locked by {}"),
    tr_const!("Devices"),
    tr_const!("Group the pages by structure"),
    tr_const!("(no structure)"),
    tr_const!("The project has no pages"),
    tr_const!("Filter devices…"),
    tr_const!("No devices found"),
    tr_const!("The project has no devices"),
    tr_const!("Previous page"),
    tr_const!("Page {} of {}"),
    tr_const!("Next page"),
    tr_const!("Zoom out"),
    tr_const!("Fit"),
    tr_const!("Zoom in"),
    tr_const!("Permissions of project {}"),
    tr_const!("The roles given here apply to this project; deny takes the rights of the role away. Only administrators lift a denial."),
    tr_const!("No grants: the roles of the users apply"),
    tr_const!("A role is a set of rights. Grants give a user or a group a role on projects (* and ? as wildcards, * for all); a deny grant takes the rights of the role away, whatever other grants give."),
    tr_const!("Managers of a project give rights on it themselves (Permissions on the project)."),
    tr_const!("Sign-in needed to view"),
    tr_const!("Sign in as an administrator to manage the server"),
    tr_const!("None (only grants)"),
    tr_const!("Projects in use"),
    tr_const!("Nobody works on a project"),
    tr_const!("{} users"),
    tr_const!("New user"),
    tr_const!("Projects"),
    tr_const!("New password"),
    tr_const!("Sessions"),
    tr_const!("Open sessions"),
    tr_const!("No open sessions"),
    tr_const!("History"),
    tr_const!("No ended sessions"),
    tr_const!("Started"),
    tr_const!("Ended"),
    tr_const!("Duration"),
    tr_const!("Last activity"),
    tr_const!("Saves"),
    tr_const!("Locked"),
    tr_const!("Project data"),
    tr_const!("Ended by"),
    tr_const!("Closed"),
    tr_const!("Expired"),
    tr_const!("Signed out"),
    tr_const!("User deleted"),
    tr_const!("Filter sessions…"),
    tr_const!("Refresh"),
    tr_const!("The server keeps the last {} ended sessions."),
    tr_const!("Directory"),
    tr_const!("you"),
    tr_const!("Local"),
    tr_const!("No users"),
    tr_const!("Delete user {}? Their open projects are closed."),
    tr_const!("The last administrator cannot lose the right to manage users"),
    tr_const!("Edit user {}"),
    tr_const!("Viewers open projects read only, reviewers also save reviews, editors change and create projects, managers also delete them and give rights on them, administrators manage the server."),
    tr_const!("The projects the role applies to (* and ? as wildcards), separated by commas; empty: all"),
    tr_const!("The directory checks the password and sets the groups at every sign-in"),
    tr_const!("Grants of the server's access settings can refer to them"),
    tr_const!("At least {} characters"),
    tr_const!("User {} exists already. Change it?"),
    tr_const!("Saved user {}"),
    tr_const!("OpenECraft project server"),
    tr_const!("Administration"),
    tr_const!("Company"),
    tr_const!("Created by"),
    tr_const!("Date"),
    tr_const!("Server"),
    tr_const!("Version"),
    tr_const!("Users"),
    tr_const!("Access rights"),
    tr_const!("Defined roles"),
    tr_const!("Overview"),
    tr_const!("Viewer"),
    tr_const!("Reviewer"),
    tr_const!("Editor"),
    tr_const!("Project manager"),
    tr_const!("Administrator"),
    tr_const!("The server records who works on which project and when: the open sessions, the locked pages and the latest ended sessions with their saves. The other users of a project see who has locked a page; administrators see the sessions."),
    tr_const!("A project names who changed it last."),
    tr_const!("Drawings"),
    tr_const!("Inspection and test report"),
    tr_const!("{} circuits"),
    tr_const!("Filter circuits…"),
    tr_const!("Download the report as an Excel workbook"),
    tr_const!("Excel"),
    tr_const!("You may read the report; entering results needs the right to save reviews."),
    tr_const!("No circuits found"),
    tr_const!("The project has no circuits fed from the mains"),
    tr_const!("missing"),
    tr_const!("Phases"),
    tr_const!("Cable"),
    tr_const!("Cross-section"),
    tr_const!("Protective device"),
    tr_const!("RCD"),
    tr_const!("Zs max (Ω)"),
    tr_const!("Protective conductor (Ω)"),
    tr_const!("Insulation resistance (MΩ)"),
    tr_const!("Loop impedance Zs (Ω)"),
    tr_const!("RCD trip current (mA)"),
    tr_const!("RCD trip time (ms)"),
    tr_const!("Function test"),
    tr_const!("Tested by"),
    tr_const!("Test instrument"),
    tr_const!("Remark"),
    tr_const!("OK"),
    tr_const!("Not OK"),
    tr_const!("Passed"),
    tr_const!("Not passed"),
    tr_const!("Incomplete"),
    tr_const!("Not tested"),
    tr_const!("Not saved"),
    tr_const!("Save"),
    tr_const!("Saved"),
    tr_const!("Next circuit"),
    tr_const!("Nothing to save"),
    tr_const!("Someone saved this meanwhile. Check the values and save again."),
    tr_const!("Insulation test voltage (V)"),
    tr_const!("RCD trip time at 5 × IΔn (ms)"),
    tr_const!("Contact voltage UB (V)"),
    tr_const!("Client"),
    tr_const!("Installation site"),
    tr_const!("Installer"),
    tr_const!("Reason for the test"),
    tr_const!("Inspector"),
    tr_const!("Next inspection"),
    tr_const!("Test instruments (IEC 61557)"),
    tr_const!("External loop impedance Ze (Ω)"),
    tr_const!("Prospective fault current at the origin (kA)"),
    tr_const!("Earth electrode resistance RA (Ω)"),
    tr_const!("Main equipotential bonding (Ω)"),
    tr_const!("New installation"),
    tr_const!("Extension"),
    tr_const!("Change"),
    tr_const!("Repair"),
    tr_const!("Not applicable"),
    tr_const!("Supply system"),
    tr_const!("Signature"),
    tr_const!("Signed by {} on {}"),
    tr_const!("Not signed"),
    tr_const!("Changing the data or the results afterwards takes the signature away."),
    tr_const!("Sign the report"),
    tr_const!("Signed"),
    tr_const!("Report data and visual inspection"),
    tr_const!("Measured at the origin"),
    tr_const!("Visual inspection"),
    tr_const!("Voltage drop (%)"),
    tr_const!("Polarity"),
    tr_const!("Phase sequence"),
    tr_const!("Insulating floors and walls (kΩ)"),
    tr_const!("Insulation monitoring device"),
    tr_const!("First fault current Id (A)"),
    tr_const!("IT system"),
    tr_const!("Protective measure"),
    tr_const!("SELV"),
    tr_const!("PELV"),
    tr_const!("Electrical separation"),
    tr_const!("No protective measure of its own"),
    tr_const!("SELV, PELV and separation"),
    tr_const!("Circuits"),
    tr_const!("Kind"),
    tr_const!("Transformer"),
    tr_const!("Power supply unit"),
    tr_const!("Output voltage (V)"),
    tr_const!("Above {} V: no SELV or PELV"),
    tr_const!("Sign in or name the inspector in the report data first."),
    tr_const!("Insulation L–PE (MΩ)"),
    tr_const!("Insulation N–PE (MΩ)"),
    tr_const!("Insulation L–N (MΩ)"),
    tr_const!("Breaking capacity of {} below the fault current at the origin"),
    tr_const!("To check"),
    tr_const!("Add an item"),
    tr_const!("Own item of the visual inspection"),
    tr_const!("Limits"),
    tr_const!("Empty: the default. Set other values where the edition of the standard the installation is verified against differs."),
];

pub async fn index() -> impl IntoResponse {
    openecraft_web::page(PAGE)
}

/// The scripts and styles of the page.
pub fn routes<S: Clone + Send + Sync + 'static>() -> Router<S> {
    openecraft_web::routes()
        .route("/project.js", get(|| async { openecraft_web::script(SCRIPT) }))
        .route("/admin.js", get(|| async { openecraft_web::script(ADMIN_SCRIPT) }))
        .route("/inspection.js", get(|| async { openecraft_web::script(INSPECTION_SCRIPT) }))
}

pub fn web_texts(lang: Lang) -> WebTexts {
    let role_title = |name: &str| ProjectRole::ALL.iter().find(|r| r.name() == name).map_or("", |r| r.title());
    // The items of the visual inspection and the limits come with the report, in English.
    let checks = openecraft_core::inspection::CHECKS.iter().map(|c| c.1).chain(openecraft_core::inspection::LIMITS.iter().map(|l| l.1));
    WebTexts::new::<ProjectRight>(lang, TEXTS.iter().copied().chain(checks), role_title)
}

/// `GET /api/v1/texts?lang=de`: the page's texts in a language (English if unknown).
pub async fn texts(Query(q): Query<TextQuery>) -> Json<WebTexts> {
    Json(web_texts(q.lang()))
}

#[cfg(test)]
mod tests;
