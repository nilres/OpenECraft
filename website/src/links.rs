//! The settings of the page's links (`config.js`): the tests keep the page, the nginx
//! site of the Docker image, the compose file and the documentation to the same names.

/// The settings of `config.js` with the environment variables of the Docker image.
pub const SETTINGS: [(&str, &str); 5] = [
    ("editor", "OPENECRAFT_EDITOR_URL"),
    ("repository", "OPENECRAFT_REPOSITORY_URL"),
    ("downloads", "OPENECRAFT_DOWNLOADS_URL"),
    ("documentation", "OPENECRAFT_DOCUMENTATION_URL"),
    ("issues", "OPENECRAFT_ISSUES_URL"),
];

/// The fields of the legal notice (`legal` of `config.js`) with their variables.
pub const LEGAL: [(&str, &str); 10] = [
    ("name", "OPENECRAFT_LEGAL_NAME"),
    ("address", "OPENECRAFT_LEGAL_ADDRESS"),
    ("email", "OPENECRAFT_LEGAL_EMAIL"),
    ("phone", "OPENECRAFT_LEGAL_PHONE"),
    ("representative", "OPENECRAFT_LEGAL_REPRESENTATIVE"),
    ("register", "OPENECRAFT_LEGAL_REGISTER"),
    ("vat", "OPENECRAFT_LEGAL_VAT_ID"),
    ("responsible", "OPENECRAFT_LEGAL_RESPONSIBLE"),
    ("text", "OPENECRAFT_LEGAL_TEXT"),
    ("privacy", "OPENECRAFT_LEGAL_PRIVACY"),
];

/// The license files the build puts beside the page (`installer/web/licenses.sh`).
pub const LICENSES: [&str; 4] = ["LICENSE-MIT", "LICENSE-APACHE", "OFL-NotoSans.txt", "THIRD-PARTY-LICENSES.html"];

/// The headers of the nginx site of the Docker image, in every block that sets headers.
pub const HEADERS: [&str; 3] = ["X-Content-Type-Options nosniff", "Referrer-Policy strict-origin-when-cross-origin", "X-Frame-Options SAMEORIGIN"];

#[cfg(test)]
mod tests;
