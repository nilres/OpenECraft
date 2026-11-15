//! Language state and the lookup / substitution functions (see the crate docs).

use crate::catalog;
use std::cell::Cell;
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::OnceLock;

/// A supported UI language.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Lang {
    En,
    De,
    Fr,
    Es,
    Pt,
    It,
    Nl,
    Pl,
    Cs,
    Ru,
    Uk,
    Tr,
    Sv,
    Da,
    Fi,
    Hu,
    Ro,
    Zh,
    Ja,
    Ko,
}

impl Lang {
    pub const ALL: [Lang; 20] = [
        Lang::En,
        Lang::De,
        Lang::Fr,
        Lang::Es,
        Lang::Pt,
        Lang::It,
        Lang::Nl,
        Lang::Pl,
        Lang::Cs,
        Lang::Ru,
        Lang::Uk,
        Lang::Tr,
        Lang::Sv,
        Lang::Da,
        Lang::Fi,
        Lang::Hu,
        Lang::Ro,
        Lang::Zh,
        Lang::Ja,
        Lang::Ko,
    ];

    /// The languages every text must be translated into (see `AGENTS.md`); the others
    /// show texts they lack in English.
    pub const REQUIRED: [Lang; 4] = [Lang::En, Lang::De, Lang::Fr, Lang::Es];

    /// ISO 639-1 code, also used for persisting the choice.
    /// The language writes decimals with a comma.
    pub fn decimal_comma(self) -> bool {
        !matches!(self, Lang::En | Lang::Zh | Lang::Ja | Lang::Ko)
    }

    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::De => "de",
            Lang::Fr => "fr",
            Lang::Es => "es",
            Lang::Pt => "pt",
            Lang::It => "it",
            Lang::Nl => "nl",
            Lang::Pl => "pl",
            Lang::Cs => "cs",
            Lang::Ru => "ru",
            Lang::Uk => "uk",
            Lang::Tr => "tr",
            Lang::Sv => "sv",
            Lang::Da => "da",
            Lang::Fi => "fi",
            Lang::Hu => "hu",
            Lang::Ro => "ro",
            Lang::Zh => "zh",
            Lang::Ja => "ja",
            Lang::Ko => "ko",
        }
    }

    /// Name of the language in the language itself.
    pub fn native_name(self) -> &'static str {
        match self {
            Lang::En => "English",
            Lang::De => "Deutsch",
            Lang::Fr => "Français",
            Lang::Es => "Español",
            Lang::Pt => "Português",
            Lang::It => "Italiano",
            Lang::Nl => "Nederlands",
            Lang::Pl => "Polski",
            Lang::Cs => "Čeština",
            Lang::Ru => "Русский",
            Lang::Uk => "Українська",
            Lang::Tr => "Türkçe",
            Lang::Sv => "Svenska",
            Lang::Da => "Dansk",
            Lang::Fi => "Suomi",
            Lang::Hu => "Magyar",
            Lang::Ro => "Română",
            Lang::Zh => "简体中文",
            Lang::Ja => "日本語",
            Lang::Ko => "한국어",
        }
    }

    /// Whether the language is one of [`Lang::REQUIRED`].
    pub fn is_required(self) -> bool {
        Lang::REQUIRED.contains(&self)
    }

    /// Language for an ISO code or a locale such as `de_DE.UTF-8` / `fr-CA`.
    pub fn from_code(code: &str) -> Option<Lang> {
        let primary = code.trim().split(['_', '-', '.', '@']).next()?.to_ascii_lowercase();
        Lang::ALL.into_iter().find(|l| l.code() == primary)
    }

    fn index(self) -> u8 {
        self as u8
    }
}

static CURRENT: AtomicU8 = AtomicU8::new(0);

thread_local! {
    /// Language of output generated on this thread (see [`with_lang`]).
    static OVERRIDE: Cell<Option<Lang>> = const { Cell::new(None) };
}

pub fn lang() -> Lang {
    OVERRIDE.with(Cell::get).unwrap_or_else(|| Lang::ALL[CURRENT.load(Ordering::Relaxed) as usize % Lang::ALL.len()])
}

/// Runs `f` with `lang` as the language of this thread, e.g. to generate a PDF in
/// another language than the UI's; the UI language stays as it is.
pub fn with_lang<R>(lang: Lang, f: impl FnOnce() -> R) -> R {
    struct Restore(Option<Lang>);
    impl Drop for Restore {
        fn drop(&mut self) {
            OVERRIDE.with(|o| o.set(self.0));
        }
    }
    let _restore = Restore(OVERRIDE.with(|o| o.replace(Some(lang))));
    f()
}

pub fn set_lang(lang: Lang) {
    CURRENT.store(lang.index(), Ordering::Relaxed);
}

/// Language of the operating system (`LC_ALL`, `LC_MESSAGES`, `LANG`), English if unknown.
pub fn system_lang() -> Lang {
    ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .filter_map(|v| std::env::var(v).ok())
        .find_map(|v| Lang::from_code(&v))
        .unwrap_or(Lang::En)
}

/// Start-up language: the stored choice (ISO code), otherwise the language of the
/// operating system if it is supported; a language already set by the platform entry
/// point (browser language) is kept.
pub fn init(stored: Option<&str>) {
    if let Some(l) = stored.and_then(Lang::from_code) {
        set_lang(l);
    } else if system_lang() != Lang::En {
        set_lang(system_lang());
    }
}

/// All catalog entries as `[English, German, French, Spanish]` (for tests of the applications).
pub fn catalog_entries() -> Vec<[&'static str; 4]> {
    catalog::all().map(|(en, de, fr, es)| [en, de, fr, es]).collect()
}

/// The translations of a further language (not [`Lang::REQUIRED`]) as `(English, translation)`.
pub fn catalog_translations(lang: Lang) -> Vec<(&'static str, &'static str)> {
    catalog::further(lang.code()).flatten().collect()
}

fn table() -> &'static HashMap<&'static str, [&'static str; 3]> {
    static TABLE: OnceLock<HashMap<&'static str, [&'static str; 3]>> = OnceLock::new();
    TABLE.get_or_init(|| catalog::all().map(|(en, de, fr, es)| (en, [de, fr, es])).collect())
}

/// The table of a further language, read when the language is first used.
fn further_table(lang: Lang) -> &'static HashMap<&'static str, &'static str> {
    static TABLES: [OnceLock<HashMap<&'static str, &'static str>>; Lang::ALL.len()] = [const { OnceLock::new() }; Lang::ALL.len()];
    TABLES[lang.index() as usize].get_or_init(|| catalog::further(lang.code()).flatten().collect())
}

/// Translation of the English text `en` into the current language; unknown texts are
/// returned unchanged.
pub fn tr(en: &'static str) -> &'static str {
    translate(en, lang())
}

pub fn translate(en: &'static str, lang: Lang) -> &'static str {
    match lang {
        Lang::En => en,
        Lang::De | Lang::Fr | Lang::Es => table().get(en).map(|t| t[lang.index() as usize - 1]).unwrap_or(en),
        _ => further_table(lang).get(en).copied().unwrap_or(en),
    }
}

/// The translation of an English text that is not a literal of the program (read from
/// a file, for example), if the catalog of `lang` has it.
pub fn translation(en: &str, lang: Lang) -> Option<&'static str> {
    match lang {
        Lang::En => None,
        Lang::De | Lang::Fr | Lang::Es => table().get(en).map(|t| t[lang.index() as usize - 1]),
        _ => further_table(lang).get(en).copied(),
    }
}

/// A file error in the current language: the operating system words its errors in
/// its own language, which may differ from the program's.
pub fn io_error(e: &std::io::Error) -> String {
    use std::io::ErrorKind as K;
    match e.kind() {
        K::NotFound => tr("file or folder not found").to_string(),
        K::PermissionDenied => tr("access denied").to_string(),
        K::AlreadyExists => tr("the file exists already").to_string(),
        K::IsADirectory => tr("this is a folder").to_string(),
        K::StorageFull => tr("the disk is full").to_string(),
        _ => e.to_string(),
    }
}

/// A number with `digits` decimals and the decimal separator of the current language
/// (a comma in most European languages).
pub fn decimal(value: f64, digits: usize) -> String {
    let text = format!("{value:.digits$}");
    if lang().decimal_comma() {
        text.replace('.', ",")
    } else {
        text
    }
}

/// Like [`tr`] for texts with placeholders: `{}` is replaced by the arguments in order,
/// `{0}`, `{1}`, … by the argument with that index (translations may reorder them).
pub fn tr_args(en: &'static str, args: &[&dyn Display]) -> String {
    fill(translate(en, lang()), args)
}

/// `tr_fmt!("Saved {}", name)` – translated text with `{}` placeholders filled by the
/// arguments (anything that implements `Display`).
#[macro_export]
macro_rules! tr_fmt {
    ($text:literal $(, $arg:expr)* $(,)?) => {
        $crate::tr_args($text, &[$(&$arg as &dyn ::std::fmt::Display),*])
    };
}

/// Marks an English text in a `const` / `static` table as translatable (the text is
/// translated with [`tr`] where it is shown); keeps it visible to the catalog test.
#[macro_export]
macro_rules! tr_const {
    ($text:literal) => {
        $text
    };
}

fn fill(text: &str, args: &[&dyn Display]) -> String {
    let mut out = String::with_capacity(text.len() + 16);
    let mut next = 0;
    let mut rest = text;
    while let Some(start) = rest.find('{') {
        let Some(end) = rest[start..].find('}') else { break };
        let key = &rest[start + 1..start + end];
        let index = if key.is_empty() {
            next += 1;
            Some(next - 1)
        } else {
            key.parse::<usize>().ok()
        };
        out.push_str(&rest[..start]);
        match index.and_then(|i| args.get(i)) {
            Some(arg) => out.push_str(&arg.to_string()),
            None => out.push_str(&rest[start..start + end + 1]),
        }
        rest = &rest[start + end + 1..];
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests;
