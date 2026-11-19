//! The legal texts of the applications (the editor's Help › Legal and privacy, the parts
//! manager's Legal and privacy): the legal notice of who runs a web build
//! ([`crate::io::site_legal`]), the disclaimer of OpenECraft and the privacy notice of
//! the application. Keep the privacy texts true: a change that stores or sends
//! something else changes them too.

use openecraft_i18n::tr;
use openecraft_parts::legal::LegalNotice;

/// The application the privacy notice is about.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Program {
    Editor,
    PartsManager,
}

/// The legal texts as headings with their paragraphs: the legal notice of the operator
/// (a web build whose operator has a name), the disclaimer and the privacy notice of
/// `program` on the `desktop` or in the browser, with what the operator adds to it.
pub fn sections(notice: &LegalNotice, desktop: bool, program: Program) -> Vec<(&'static str, Vec<String>)> {
    let mut out = Vec::new();
    if notice.shown() {
        let mut lines = vec![notice.field("name").to_string(), notice.lines("address").join("\n")];
        for (label, field) in [
            (tr("Represented by"), "representative"),
            (tr("Email"), "email"),
            (tr("Phone"), "phone"),
            (tr("Register entry"), "register"),
            (tr("VAT ID"), "vat"),
            (tr("Responsible for the content"), "responsible"),
        ] {
            if !notice.field(field).is_empty() {
                lines.push(format!("{label}: {}", notice.field(field)));
            }
        }
        lines.extend(notice.lines("text").into_iter().map(String::from));
        lines.retain(|l| !l.is_empty());
        out.push((tr("Legal notice"), lines));
    }
    out.push((
        tr("Disclaimer"),
        vec![
            tr("OpenECraft comes without warranty. Its calculations and checks support the planning; a qualified person has to verify the results against the standards and regulations that apply.").to_string(),
            format!(
                "{} {}",
                tr("EPLAN, TIA Portal and other names are trademarks of their owners. OpenECraft is an independent project."),
                tr("It is not affiliated with, endorsed by or sponsored by the owners of these names.")
            ),
        ],
    ));
    let mut privacy: Vec<&str> = match (program, desktop) {
        (Program::Editor, true) => vec![
            tr("OpenECraft uses no tracking and no analytics. Your projects stay where you save them; nothing is sent to the makers of OpenECraft."),
            tr("The editor connects only to the servers you set up: parts servers, project servers, the EPLAN Data Portal and Autodesk Vault receive your requests, your sign-in and what you save to them. Their operators have privacy notices of their own."),
            tr("Your settings and the data to restore your work are kept in your user folder on this computer."),
            tr("To find updates, OpenECraft asks GitHub for the latest release when it starts, unless you switch that off in the settings; GitHub receives the IP address of your computer."),
        ],
        (Program::Editor, false) => vec![
            tr("OpenECraft uses no tracking and no analytics. Your projects stay where you save them; nothing is sent to the makers of OpenECraft."),
            tr("The editor connects only to the servers you set up: parts servers, project servers, the EPLAN Data Portal and Autodesk Vault receive your requests, your sign-in and what you save to them. Their operators have privacy notices of their own."),
            tr("The web editor sets no cookies. It keeps your settings and the data to restore your work in the storage of your browser; clearing the site data of the browser removes them."),
            tr("As with every website, the server that delivers this page receives the IP address of your device in order to answer."),
        ],
        (Program::PartsManager, true) => vec![
            tr("The parts manager uses no tracking and no analytics; nothing is sent to the makers of OpenECraft."),
            tr("It connects only to the parts server you name, which receives your requests, your sign-in and the parts you save. Its operator has a privacy notice of their own."),
            tr("The address of the server and your settings are kept in your user folder on this computer; the access token and your password are not stored."),
        ],
        (Program::PartsManager, false) => vec![
            tr("The parts manager uses no tracking and no analytics; nothing is sent to the makers of OpenECraft."),
            tr("It connects only to the parts server you name, which receives your requests, your sign-in and the parts you save. Its operator has a privacy notice of their own."),
            tr("The parts manager sets no cookies. It keeps the address of the server and your settings in the storage of your browser; clearing the site data of the browser removes them. The access token and your password are not stored."),
            tr("As with every website, the server that delivers this page receives the IP address of your device in order to answer."),
        ],
    };
    privacy.insert(2, tr("The one exception is the preset parts server: the public parts server of the OpenECraft project, https://parts-server.openecraft.org, receives your requests and the IP address of your device when the program connects to it. Enter the address of another parts server to use that one instead."));
    if program == Program::Editor {
        privacy.push(tr("Reporting a bug opens GitHub in your browser with the details the form shows; nothing is sent before."));
    }
    let mut privacy: Vec<String> = privacy.into_iter().map(String::from).collect();
    privacy.extend(notice.lines("privacy").into_iter().map(String::from));
    out.push((tr("Privacy"), privacy));
    out
}

/// Draws the legal texts of `program` as it runs here.
pub fn show(ui: &mut egui::Ui, notice: &LegalNotice, program: Program) {
    for (title, paragraphs) in sections(notice, crate::io::has_file_system(), program) {
        ui.heading(title);
        for text in paragraphs {
            ui.label(text);
            ui.add_space(4.0);
        }
        ui.add_space(8.0);
    }
}

#[cfg(test)]
mod tests;
