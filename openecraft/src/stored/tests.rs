use super::*;
use eframe::Storage;
use std::collections::{BTreeMap, HashMap};

#[derive(Default)]
struct Memory(HashMap<String, String>);

impl eframe::Storage for Memory {
    fn get_string(&self, key: &str) -> Option<String> {
        self.0.get(key).cloned()
    }

    fn set_string(&mut self, key: &str, value: String) {
        self.0.insert(key.to_string(), value);
    }

    fn remove_string(&mut self, key: &str) {
        self.0.remove(key);
    }

    fn flush(&mut self) {}
}

#[test]
fn settings_are_stored_as_json_and_read_back() {
    let mut storage = Memory::default();
    let keys: BTreeMap<String, Vec<String>> = [("save".to_string(), vec!["Ctrl+S".to_string()])].into();
    set(&mut storage, "keys", &keys);
    set(&mut storage, "file", &Some("C:\\plans\\a \"b\".oec".to_string()));
    set(&mut storage, "none", &None::<String>);
    assert_eq!(storage.0["keys"], r#"{"save":["Ctrl+S"]}"#);
    assert_eq!(get::<BTreeMap<String, Vec<String>>>(&storage, "keys"), Some(keys));
    assert_eq!(get::<Option<String>>(&storage, "file").flatten().as_deref(), Some("C:\\plans\\a \"b\".oec"));
    assert_eq!(get::<Option<String>>(&storage, "none"), Some(None));
    assert_eq!(get::<String>(&storage, "missing"), None);
    storage.set_string("empty", String::new());
    assert_eq!(get::<String>(&storage, "empty"), None);
}

#[test]
fn settings_stored_before_as_ron_are_read() {
    let mut storage = Memory::default();
    let recent = vec!["a.oec".to_string(), "b.oec".to_string()];
    eframe::set_value(&mut storage, "recent", &recent);
    eframe::set_value(&mut storage, "file", &Some("a.oec".to_string()));
    eframe::set_value(&mut storage, "language", &"de");
    eframe::set_value(&mut storage, "parts", &openecraft_parts::PartsDb { parts: Vec::new() });
    assert_eq!(get::<Vec<String>>(&storage, "recent"), Some(recent));
    assert_eq!(get::<Option<String>>(&storage, "file").flatten().as_deref(), Some("a.oec"));
    assert_eq!(get::<String>(&storage, "language").as_deref(), Some("de"));
    // The library reads the JSON, RON is read as before.
    let parts = get_as(&storage, "parts", openecraft_parts::PartsDb::from_json);
    assert_eq!(parts, Some(openecraft_parts::PartsDb { parts: Vec::new() }));
    let json = openecraft_parts::PartsDb { parts: Vec::new() }.to_json();
    storage.set_string("parts", json);
    assert_eq!(get_as(&storage, "parts", openecraft_parts::PartsDb::from_json), Some(openecraft_parts::PartsDb { parts: Vec::new() }));
}
