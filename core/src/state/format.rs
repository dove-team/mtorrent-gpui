pub fn decimal_separator() -> char {
    match &*rust_i18n::locale() {
        "ru" => ',',
        _ => '.',
    }
}

pub fn format_bytes(bytes: u64) -> String {
    if bytes == 0 {
        return "0 B".to_string();
    }
    const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB"];
    let k = 1024f64;
    let i = (bytes as f64).log(k) as usize;
    let i = i.min(UNITS.len() - 1);
    let value = bytes as f64 / k.powi(i as i32);
    let value = if value < 10.0 {
        format!("{value:.1}")
    } else {
        format!("{value:.0}")
    };
    let value = if decimal_separator() == ',' {
        value.replace('.', ",")
    } else {
        value
    };
    format!("{value} {}", UNITS[i])
}

pub fn format_bytes_kib(bytes: u64) -> String {
    let kib = (bytes / 1024).to_string();
    let mut grouped = String::with_capacity(kib.len() + 8);
    for (i, ch) in kib.chars().enumerate() {
        if i > 0 && (kib.len() - i).is_multiple_of(3) {
            grouped.push(' ');
        }
        grouped.push(ch);
    }
    grouped.push_str(" KiB");
    grouped
}

pub fn format_percent(ratio: f32) -> String {
    let text = format!("{:.1}%", ratio * 100.0);
    if decimal_separator() == ',' {
        text.replace('.', ",")
    } else {
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_bytes_units() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.0 KiB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MiB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GiB");
        assert_eq!(format_bytes(1024u64 * 1024 * 1024 * 1024), "1.0 TiB");
    }

    #[test]
    fn format_bytes_kib_grouping() {
        assert_eq!(format_bytes_kib(0), "0 KiB");
        assert_eq!(format_bytes_kib(1024 * 1024), "1 024 KiB");
        assert_eq!(format_bytes_kib(1024 * 1024 * 1024), "1 048 576 KiB");
    }

    #[test]
    fn percent_uses_dot_separator_by_default() {
        assert_eq!(format_percent(0.3), "30.0%");
    }
}
