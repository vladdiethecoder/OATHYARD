use std::fmt::Write as _;

pub(crate) fn write_json_field(
    out: &mut String,
    indent: usize,
    key: &str,
    value: &str,
    trailing: bool,
) {
    let spaces = "  ".repeat(indent);
    writeln!(
        out,
        "{}{}: {}{}",
        spaces,
        json_quote(key),
        json_quote(value),
        if trailing { "," } else { "" }
    )
    .unwrap();
}

pub(crate) fn json_quote(value: &str) -> String {
    let mut out = String::new();
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => write!(&mut out, "\\u{:04x}", ch as u32).unwrap(),
            ch => out.push(ch),
        }
    }
    out.push('"');
    out
}

pub(crate) fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub(crate) fn comma(index: usize, len: usize) -> &'static str {
    if index < len {
        ","
    } else {
        ""
    }
}

pub(crate) fn hash_hex(bytes: &[u8]) -> String {
    format!("{:016x}", fnv1a64(bytes))
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

pub(crate) fn json_string_value(input: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\":");
    let start = input.find(&needle)? + needle.len();
    let after_colon = input[start..].trim_start();
    if !after_colon.starts_with('"') {
        return None;
    }
    parse_json_string(after_colon).map(|(value, _)| value)
}

pub(crate) fn json_string_array(input: &str, key: &str) -> Option<Vec<String>> {
    let needle = format!("\"{key}\":");
    let start = input.find(&needle)? + needle.len();
    let mut rest = input[start..].trim_start();
    if !rest.starts_with('[') {
        return None;
    }
    rest = &rest[1..];
    let mut values = Vec::new();
    loop {
        rest = rest.trim_start();
        if rest.starts_with(']') {
            return Some(values);
        }
        let (value, consumed) = parse_json_string(rest)?;
        values.push(value);
        rest = &rest[consumed..];
        rest = rest.trim_start();
        if rest.starts_with(',') {
            rest = &rest[1..];
        } else if rest.starts_with(']') {
            return Some(values);
        } else {
            return None;
        }
    }
}

pub(crate) fn json_u32_value(input: &str, key: &str) -> Option<u32> {
    let needle = format!("\"{key}\":");
    let start = input.find(&needle)? + needle.len();
    let after_colon = input[start..].trim_start();
    let digits_end = after_colon
        .char_indices()
        .take_while(|(_, ch)| ch.is_ascii_digit())
        .map(|(index, ch)| index + ch.len_utf8())
        .last()?;
    let after_digits = after_colon[digits_end..].trim_start();
    if !(after_digits.is_empty()
        || after_digits.starts_with(',')
        || after_digits.starts_with('}')
        || after_digits.starts_with(']'))
    {
        return None;
    }
    after_colon[..digits_end].parse().ok()
}

fn parse_json_string(input: &str) -> Option<(String, usize)> {
    let bytes = input.as_bytes();
    if bytes.first().copied()? != b'"' {
        return None;
    }
    let mut out = String::new();
    let mut index = 1;
    while index < bytes.len() {
        match bytes[index] {
            b'"' => return Some((out, index + 1)),
            b'\\' => {
                index += 1;
                if index >= bytes.len() {
                    return None;
                }
                match bytes[index] {
                    b'"' => out.push('"'),
                    b'\\' => out.push('\\'),
                    b'n' => out.push('\n'),
                    b'r' => out.push('\r'),
                    b't' => out.push('\t'),
                    _ => return None,
                }
            }
            other => out.push(other as char),
        }
        index += 1;
    }
    None
}
