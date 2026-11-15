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

#[cfg(test)]
mod tests;
