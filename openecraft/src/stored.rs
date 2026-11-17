//! The settings the editor keeps between sessions in the storage of eframe, as JSON
//! (`openecraft_ui::io::stored`: the desktop also reads what was stored before as
//! RON).

use serde::{de::DeserializeOwned, Serialize};

pub fn get<T: DeserializeOwned>(storage: &dyn eframe::Storage, key: &str) -> Option<T> {
    storage.get_string(key).and_then(|text| openecraft_ui::io::stored(&text))
}

/// [`get`] of a big type with the reader of its library (`openecraft_ui::io::stored_as`).
pub fn get_as<T: DeserializeOwned>(storage: &dyn eframe::Storage, key: &str, json: impl FnOnce(&str) -> Option<T>) -> Option<T> {
    storage.get_string(key).and_then(|text| openecraft_ui::io::stored_as(&text, json))
}

pub fn set<T: Serialize>(storage: &mut dyn eframe::Storage, key: &str, value: &T) {
    if let Ok(text) = serde_json::to_string(value) {
        storage.set_string(key, text);
    }
}

#[cfg(test)]
mod tests;
