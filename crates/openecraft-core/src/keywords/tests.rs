use super::*;

fn has(text: &str, word: &str) -> bool {
    keywords(text).split(' ').any(|w| w.eq_ignore_ascii_case(word))
}

#[test]
fn abbreviations_of_common_devices() {
    assert!(has("Residual current device 4-pole", "RCD"));
    assert!(has("Residual current device 2-pole", "FI"));
    assert!(has("Circuit breaker 3-pole, thermal-magnetic", "MCB"));
    assert!(has("Circuit breaker 1-pole", "LS"));
    assert!(has("Surge protective device", "SPD"));
    assert!(has("Power supply AC/DC", "PSU"));
    assert!(has("PLC card 8 digital inputs", "PLC"));
    assert!(has("Frequency converter", "VFD"));
}

#[test]
fn translated_names_and_compound_words_find_the_keywords() {
    assert!(has("Fehlerstromschutzschalter 40 A", "RCD"));
    assert!(has("Leitungsschutzschalter B16", "MCB"));
    assert!(has("Interrupteur différentiel 30 mA", "RCD"));
    assert!(has("Drehstrommotor", "M"));
}

#[test]
fn short_terms_only_match_where_a_word_starts() {
    assert_eq!(keywords("Controlled replacement"), "");
    assert!(has("Status LED green", "LED"));
    assert_eq!(keywords(""), "");
}

#[test]
fn keywords_are_listed_once() {
    let words = keywords("PLC digital input");
    let list: Vec<&str> = words.split(' ').collect();
    assert_eq!(list.iter().filter(|w| w.eq_ignore_ascii_case("plc")).count(), 1);
}

#[test]
fn terms_are_lower_case() {
    for (terms, words) in TERMS {
        assert!(!words.is_empty());
        for t in *terms {
            assert_eq!(*t, t.to_lowercase(), "term {t}");
        }
    }
}
