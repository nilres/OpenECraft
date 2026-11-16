//! The legal notice of who runs an installation (in Germany the *Impressum*) and what
//! they add to its privacy notice: set by the operator of the website, of a web editor
//! or of a server, and shown as it is set. The website reads the same fields from its
//! `config.js` (`docs/website.md`).

use serde::{Deserialize, Serialize};

/// The fields with the environment variables that set them, in the order shown.
pub const FIELDS: [(&str, &str); 10] = [
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

/// Who runs an installation. Without a name there is no legal notice; `privacy` is
/// shown below the privacy notice also without one.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LegalNotice {
    /// The person or company running the installation.
    pub name: String,
    /// Its postal address (several lines).
    pub address: String,
    pub email: String,
    pub phone: String,
    /// Who represents a company.
    pub representative: String,
    /// Register court and number.
    pub register: String,
    /// VAT identification number.
    pub vat: String,
    /// Who is responsible for the content, with address.
    pub responsible: String,
    /// Further text: disclaimers, dispute resolution (several lines).
    pub text: String,
    /// What the operator adds to the privacy notice: the hosting, the logs, whom to
    /// ask (several lines).
    pub privacy: String,
}

impl LegalNotice {
    fn field_mut(&mut self, name: &str) -> Option<&mut String> {
        Some(match name {
            "name" => &mut self.name,
            "address" => &mut self.address,
            "email" => &mut self.email,
            "phone" => &mut self.phone,
            "representative" => &mut self.representative,
            "register" => &mut self.register,
            "vat" => &mut self.vat,
            "responsible" => &mut self.responsible,
            "text" => &mut self.text,
            "privacy" => &mut self.privacy,
            _ => return None,
        })
    }

    /// The value of a field of [`FIELDS`] without blanks around it.
    pub fn field(&self, name: &str) -> &str {
        match name {
            "name" => &self.name,
            "address" => &self.address,
            "email" => &self.email,
            "phone" => &self.phone,
            "representative" => &self.representative,
            "register" => &self.register,
            "vat" => &self.vat,
            "responsible" => &self.responsible,
            "text" => &self.text,
            "privacy" => &self.privacy,
            _ => "",
        }
        .trim()
    }

    /// Sets a field of [`FIELDS`]; false for another name.
    pub fn set(&mut self, name: &str, value: impl Into<String>) -> bool {
        match self.field_mut(name) {
            Some(field) => {
                *field = value.into();
                true
            }
            None => false,
        }
    }

    /// The notice with the values of the environment (`variable` gives the value of a
    /// variable of [`FIELDS`]); a variable that is set and not blank wins.
    pub fn with_environment(mut self, variable: impl Fn(&str) -> Option<String>) -> LegalNotice {
        for (name, var) in FIELDS {
            if let Some(value) = variable(var).filter(|v| !v.trim().is_empty()) {
                self.set(name, value);
            }
        }
        self
    }

    /// True if there is a legal notice to show: the operator has a name.
    pub fn shown(&self) -> bool {
        !self.name.trim().is_empty()
    }

    /// The lines of a field: separated by line breaks or `|`, without empty ones.
    pub fn lines(&self, name: &str) -> Vec<&str> {
        self.field(name).split(['\n', '|']).map(str::trim).filter(|l| !l.is_empty()).collect()
    }
}

#[cfg(test)]
mod tests;
