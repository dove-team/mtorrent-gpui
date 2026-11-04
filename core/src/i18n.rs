use crate::host::Host;
use std::borrow::Cow;
use std::io::{self, ErrorKind};

const PREFERENCE_FILE: &str = "lang";

pub fn available() -> Vec<Cow<'static, str>> {
    rust_i18n::available_locales!()
}

pub fn current() -> String {
    rust_i18n::locale().to_string()
}

pub fn is_available(locale: &str) -> bool {
    available().iter().any(|candidate| candidate.as_ref() == locale)
}

pub fn label(locale: &str) -> Cow<'static, str> {
    match locale {
        "en" => Cow::Borrowed("English"),
        "ru" => Cow::Borrowed("Русский"),
        "zh-CN" => Cow::Borrowed("简体中文"),
        "zh-Hant" => Cow::Borrowed("繁體中文"),
        other => Cow::Owned(other.to_owned()),
    }
}

pub fn torrent_filter_label() -> String {
    rust_i18n::t!("task.torrent_filter").into_owned()
}

pub fn apply(host: &dyn Host) {
    if let Some(saved) = read_preference(host) {
        rust_i18n::set_locale(&saved);
        return;
    }

    if let Some(hint) = host.system_locale() {
        for candidate in available() {
            let name = candidate.as_ref();
            let hint_prefix = hint.split(['-', '_', '.']).next().unwrap_or("");
            if hint.eq_ignore_ascii_case(name)
                || hint.starts_with(name)
                || name.starts_with(hint_prefix)
            {
                rust_i18n::set_locale(name);
                return;
            }
        }
    }
}

pub fn select(host: &dyn Host, locale: &str) -> io::Result<()> {
    if !is_available(locale) {
        return Err(io::Error::new(ErrorKind::InvalidInput, format!("unknown locale: {locale}")));
    }
    rust_i18n::set_locale(locale);
    save_preference(host, locale)
}

pub fn cycle_next(host: &dyn Host) -> io::Result<String> {
    let locales = available();
    if locales.is_empty() {
        return Ok(current());
    }

    let position = locales
        .iter()
        .position(|locale| locale.as_ref() == current())
        .unwrap_or(0);
    let next = locales[(position + 1) % locales.len()].to_string();
    select(host, &next)?;
    Ok(next)
}

fn read_preference(host: &dyn Host) -> Option<String> {
    let raw = std::fs::read_to_string(host.pref_dir().join(PREFERENCE_FILE)).ok()?;
    let saved = raw.trim();
    is_available(saved).then(|| saved.to_owned())
}

fn save_preference(host: &dyn Host, locale: &str) -> io::Result<()> {
    let dir = host.pref_dir();
    std::fs::create_dir_all(&dir)?;
    let target = dir.join(PREFERENCE_FILE);
    let tmp = dir.join(format!("{PREFERENCE_FILE}.tmp"));
    std::fs::write(&tmp, locale)?;
    std::fs::rename(&tmp, target)
}
