//! Minimal recursive-descent JSON parser (RFC 8259).
//!
//! Zero external dependencies. Replaces the ad-hoc string-search parsing in
//! freeze_status::json_bool_field and format_utils::json_string_value/array/u32
//! for registry-entry parsing. Proper structural parsing prevents injection
//! attacks where a value string could contain a "frozen":true substring.

use crate::OathError;

/// Parsed JSON value tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(String),
    String(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

impl JsonValue {
    pub fn parse(input: &str) -> Result<JsonValue, OathError> {
        let mut parser = JsonParser {
            bytes: input.as_bytes(),
            pos: 0,
        };
        parser.skip_whitespace();
        let value = parser.parse_value()?;
        parser.skip_whitespace();
        if parser.pos < parser.bytes.len() {
            return Err(OathError::Parse(format!(
                "json: unexpected trailing data at byte {}",
                parser.pos
            )));
        }
        Ok(value)
    }

    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        match self {
            JsonValue::Object(entries) => entries.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            JsonValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            JsonValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[JsonValue]> {
        match self {
            JsonValue::Array(arr) => Some(arr),
            _ => None,
        }
    }
}

struct JsonParser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> JsonParser<'a> {
    fn skip_whitespace(&mut self) {
        while self.pos < self.bytes.len() {
            match self.bytes[self.pos] {
                b' ' | b'\t' | b'\n' | b'\r' => self.pos += 1,
                _ => break,
            }
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn parse_value(&mut self) -> Result<JsonValue, OathError> {
        self.skip_whitespace();
        match self.peek() {
            Some(b'{') => self.parse_object(),
            Some(b'[') => self.parse_array(),
            Some(b'"') => self.parse_string().map(JsonValue::String),
            Some(b't') => self.parse_true(),
            Some(b'f') => self.parse_false(),
            Some(b'n') => self.parse_null(),
            Some(b) if b == b'-' || b.is_ascii_digit() => self.parse_number(),
            Some(b) => Err(OathError::Parse(format!(
                "json: unexpected byte '{}' (0x{:02x}) at byte {}",
                b as char, b, self.pos
            ))),
            None => Err(OathError::Parse(
                "json: unexpected end of input".to_string(),
            )),
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, OathError> {
        self.pos += 1; // consume '{'
        let mut entries = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(JsonValue::Object(entries));
        }
        loop {
            self.skip_whitespace();
            if self.peek() != Some(b'"') {
                return Err(OathError::Parse(format!(
                    "json: expected string key in object at byte {}",
                    self.pos
                )));
            }
            let key = self.parse_string()?;
            self.skip_whitespace();
            if self.peek() != Some(b':') {
                return Err(OathError::Parse(format!(
                    "json: expected ':' after key at byte {}",
                    self.pos
                )));
            }
            self.pos += 1; // consume ':'
            let value = self.parse_value()?;
            entries.push((key, value));
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => {
                    self.pos += 1;
                }
                Some(b'}') => {
                    self.pos += 1;
                    break;
                }
                Some(b) => {
                    return Err(OathError::Parse(format!(
                        "json: expected ',' or '}}' in object at byte {}, got '{}'",
                        self.pos, b as char
                    )));
                }
                None => {
                    return Err(OathError::Parse(
                        "json: unexpected end of input in object".to_string(),
                    ));
                }
            }
        }
        Ok(JsonValue::Object(entries))
    }

    fn parse_array(&mut self) -> Result<JsonValue, OathError> {
        self.pos += 1; // consume '['
        let mut values = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(JsonValue::Array(values));
        }
        loop {
            let value = self.parse_value()?;
            values.push(value);
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => {
                    self.pos += 1;
                }
                Some(b']') => {
                    self.pos += 1;
                    break;
                }
                Some(b) => {
                    return Err(OathError::Parse(format!(
                        "json: expected ',' or ']' in array at byte {}, got '{}'",
                        self.pos, b as char
                    )));
                }
                None => {
                    return Err(OathError::Parse(
                        "json: unexpected end of input in array".to_string(),
                    ));
                }
            }
        }
        Ok(JsonValue::Array(values))
    }

    fn parse_string(&mut self) -> Result<String, OathError> {
        // Assumes peek() == '"'
        self.pos += 1; // consume opening quote
        let mut out = String::new();
        while self.pos < self.bytes.len() {
            let b = self.bytes[self.pos];
            match b {
                b'"' => {
                    self.pos += 1;
                    return Ok(out);
                }
                b'\\' => {
                    self.pos += 1;
                    if self.pos >= self.bytes.len() {
                        return Err(OathError::Parse(
                            "json: unterminated escape in string".to_string(),
                        ));
                    }
                    let esc = self.bytes[self.pos];
                    match esc {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'b' => out.push('\u{0008}'),
                        b'f' => out.push('\u{000C}'),
                        b'u' => {
                            let code = self.parse_unicode_escape()?;
                            if let Some(ch) = char::from_u32(code) {
                                out.push(ch);
                            } else {
                                return Err(OathError::Parse(format!(
                                    "json: invalid unicode escape U+{code:04X}"
                                )));
                            }
                        }
                        _ => {
                            return Err(OathError::Parse(format!(
                                "json: invalid escape '\\{}' at byte {}",
                                esc as char, self.pos
                            )));
                        }
                    }
                    self.pos += 1;
                }
                b if b < 0x20 => {
                    return Err(OathError::Parse(format!(
                        "json: unescaped control char 0x{:02x} at byte {}",
                        b, self.pos
                    )));
                }
                _ => {
                    // UTF-8 multi-byte: consume continuation bytes
                    let len = if b < 0x80 {
                        1
                    } else if b < 0xC0 {
                        return Err(OathError::Parse(format!(
                            "json: invalid UTF-8 continuation byte at {}",
                            self.pos
                        )));
                    } else if b < 0xE0 {
                        2
                    } else if b < 0xF0 {
                        3
                    } else {
                        4
                    };
                    if self.pos + len > self.bytes.len() {
                        return Err(OathError::Parse(
                            "json: truncated UTF-8 sequence".to_string(),
                        ));
                    }
                    match std::str::from_utf8(&self.bytes[self.pos..self.pos + len]) {
                        Ok(s) => out.push_str(s),
                        Err(_) => {
                            return Err(OathError::Parse(format!(
                                "json: invalid UTF-8 at byte {}",
                                self.pos
                            )));
                        }
                    }
                    self.pos += len;
                    continue;
                }
            }
            self.pos += 1;
        }
        Err(OathError::Parse("json: unterminated string".to_string()))
    }

    fn parse_unicode_escape(&mut self) -> Result<u32, OathError> {
        self.pos += 1; // skip 'u'
        let mut code: u32 = 0;
        for _ in 0..4 {
            if self.pos >= self.bytes.len() {
                return Err(OathError::Parse("json: truncated \\u escape".to_string()));
            }
            let h = self.bytes[self.pos];
            let digit = match h {
                b'0'..=b'9' => (h - b'0') as u32,
                b'a'..=b'f' => (h - b'a' + 10) as u32,
                b'A'..=b'F' => (h - b'A' + 10) as u32,
                _ => {
                    return Err(OathError::Parse(format!(
                        "json: invalid hex digit '{}' in \\u escape at byte {}",
                        h as char, self.pos
                    )));
                }
            };
            code = code * 16 + digit;
            self.pos += 1;
        }
        // Handle surrogate pairs
        if (0xD800..=0xDBFF).contains(&code) {
            // High surrogate; expect \uXXXX low surrogate
            if self.pos + 1 < self.bytes.len()
                && self.bytes[self.pos] == b'\\'
                && self.bytes[self.pos + 1] == b'u'
            {
                self.pos += 2;
                let mut low: u32 = 0;
                for _ in 0..4 {
                    if self.pos >= self.bytes.len() {
                        return Err(OathError::Parse(
                            "json: truncated low surrogate".to_string(),
                        ));
                    }
                    let h = self.bytes[self.pos];
                    let digit = match h {
                        b'0'..=b'9' => (h - b'0') as u32,
                        b'a'..=b'f' => (h - b'a' + 10) as u32,
                        b'A'..=b'F' => (h - b'A' + 10) as u32,
                        _ => return Err(OathError::Parse("json: bad surrogate hex".to_string())),
                    };
                    low = low * 16 + digit;
                    self.pos += 1;
                }
                if !(0xDC00..=0xDFFF).contains(&low) {
                    return Err(OathError::Parse(format!(
                        "json: invalid low surrogate U+{low:04X}"
                    )));
                }
                code = 0x10000 + ((code - 0xD800) << 10) + (low - 0xDC00);
            }
        }
        self.pos -= 1; // outer loop will +1
        Ok(code)
    }

    fn parse_true(&mut self) -> Result<JsonValue, OathError> {
        self.expect_keyword(b"true")?;
        Ok(JsonValue::Bool(true))
    }

    fn parse_false(&mut self) -> Result<JsonValue, OathError> {
        self.expect_keyword(b"false")?;
        Ok(JsonValue::Bool(false))
    }

    fn parse_null(&mut self) -> Result<JsonValue, OathError> {
        self.expect_keyword(b"null")?;
        Ok(JsonValue::Null)
    }

    fn expect_keyword(&mut self, keyword: &[u8]) -> Result<(), OathError> {
        if self.pos + keyword.len() > self.bytes.len() {
            return Err(OathError::Parse(format!(
                "json: expected '{}'",
                std::str::from_utf8(keyword).unwrap_or("?")
            )));
        }
        if &self.bytes[self.pos..self.pos + keyword.len()] != keyword {
            return Err(OathError::Parse(format!(
                "json: expected '{}' at byte {}",
                std::str::from_utf8(keyword).unwrap_or("?"),
                self.pos
            )));
        }
        self.pos += keyword.len();
        Ok(())
    }

    fn parse_number(&mut self) -> Result<JsonValue, OathError> {
        let start = self.pos;
        if self.peek() == Some(b'-') {
            self.pos += 1;
        }

        match self.peek() {
            Some(b'0') => self.pos += 1,
            Some(b'1'..=b'9') => {
                self.pos += 1;
                while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
                    self.pos += 1;
                }
            }
            _ => {
                return Err(OathError::Parse(format!(
                    "json: invalid number at byte {start}"
                )));
            }
        }

        if self.peek() == Some(b'.') {
            self.pos += 1;
            let fraction_start = self.pos;
            while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
            if self.pos == fraction_start {
                return Err(OathError::Parse(format!(
                    "json: invalid number fraction at byte {start}"
                )));
            }
        }
        if self.peek() == Some(b'e') || self.peek() == Some(b'E') {
            self.pos += 1;
            if self.peek() == Some(b'+') || self.peek() == Some(b'-') {
                self.pos += 1;
            }
            let exponent_start = self.pos;
            while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
            if self.pos == exponent_start {
                return Err(OathError::Parse(format!(
                    "json: invalid number exponent at byte {start}"
                )));
            }
        }
        let text = std::str::from_utf8(&self.bytes[start..self.pos])
            .map_err(|_| OathError::Parse("json: invalid number".to_string()))?;
        Ok(JsonValue::Number(text.to_string()))
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_object() {
        let val =
            JsonValue::parse(r#"{"name": "test", "value": 42, "flag": true}"#).expect("parse");
        assert_eq!(val.get("name").and_then(|v| v.as_str()), Some("test"));
        assert!(matches!(val.get("value"), Some(JsonValue::Number(n)) if n == "42"));
        assert_eq!(val.get("flag").and_then(|v| v.as_bool()), Some(true));
    }

    #[test]
    fn nested_conditions_object() {
        let val = JsonValue::parse(r#"{"conditions": {"frozen": true, "deterministic": false}}"#)
            .expect("parse");
        let conditions = val.get("conditions").expect("conditions");
        assert_eq!(
            conditions.get("frozen").and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            conditions.get("deterministic").and_then(|v| v.as_bool()),
            Some(false)
        );
    }

    #[test]
    fn string_containing_json_like_substring_is_not_parsed_as_value() {
        // This is the R-HASH-3 injection test: a string value that looks like
        // a boolean field must not be extractable by string search.
        let val =
            JsonValue::parse(r#"{"note": "\"frozen\": true", "frozen": false}"#).expect("parse");
        // The real frozen field is false
        assert_eq!(val.get("frozen").and_then(|v| v.as_bool()), Some(false));
        // The note field is a string, not parsed as an object
        let note = val.get("note").expect("note");
        assert!(matches!(note, JsonValue::String(_)));
    }

    #[test]
    fn reject_trailing_garbage() {
        assert!(JsonValue::parse(r#"{"a": 1} garbage"#).is_err());
    }

    #[test]
    fn reject_malformed() {
        assert!(JsonValue::parse(r#"{"a":}"#).is_err());
        assert!(JsonValue::parse(r#"[1, 2,]"#).is_err());
        assert!(JsonValue::parse(r#""unterminated"#).is_err());
    }
}
