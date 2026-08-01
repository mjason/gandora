//! Minimal JSONC parser (comments and trailing commas permitted,
//! duplicate keys rejected) for `gandora.jsonc` (GEP-0001-R018).

use crate::diag::{Diagnostic, Result, Span};

#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

pub fn parse_jsonc(file: &str, text: &str) -> Result<JsonValue> {
    let mut p = P {
        file,
        src: text.chars().collect(),
        pos: 0,
    };
    p.skip_ws();
    let v = p.value()?;
    p.skip_ws();
    if p.pos < p.src.len() {
        return Err(p.err("unexpected trailing content"));
    }
    Ok(v)
}

struct P<'a> {
    file: &'a str,
    src: Vec<char>,
    pos: usize,
}

impl<'a> P<'a> {
    fn err(&self, msg: impl Into<String>) -> Diagnostic {
        Diagnostic::new(self.file, Span::default(), msg)
    }

    fn peek(&self) -> Option<char> {
        self.src.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += 1;
        Some(c)
    }

    fn skip_ws(&mut self) {
        loop {
            match self.peek() {
                Some(c) if c.is_whitespace() => {
                    self.bump();
                }
                Some('/') if self.src.get(self.pos + 1) == Some(&'/') => {
                    while let Some(c) = self.peek() {
                        if c == '\n' {
                            break;
                        }
                        self.bump();
                    }
                }
                Some('/') if self.src.get(self.pos + 1) == Some(&'*') => {
                    self.bump();
                    self.bump();
                    while self.pos < self.src.len() {
                        if self.peek() == Some('*') && self.src.get(self.pos + 1) == Some(&'/') {
                            self.bump();
                            self.bump();
                            break;
                        }
                        self.bump();
                    }
                }
                _ => break,
            }
        }
    }

    fn value(&mut self) -> Result<JsonValue> {
        match self.peek() {
            Some('{') => self.object(),
            Some('[') => self.array(),
            Some('"') => Ok(JsonValue::String(self.string()?)),
            Some('t') => self.literal("true", JsonValue::Bool(true)),
            Some('f') => self.literal("false", JsonValue::Bool(false)),
            Some('n') => self.literal("null", JsonValue::Null),
            Some(c) if c == '-' || c.is_ascii_digit() => self.number(),
            _ => Err(self.err("expected a JSON value")),
        }
    }

    fn literal(&mut self, word: &str, value: JsonValue) -> Result<JsonValue> {
        for expected in word.chars() {
            if self.bump() != Some(expected) {
                return Err(self.err(format!("expected '{word}'")));
            }
        }
        Ok(value)
    }

    fn number(&mut self) -> Result<JsonValue> {
        let mut text = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || "+-.eE".contains(c) {
                text.push(c);
                self.bump();
            } else {
                break;
            }
        }
        text.parse::<f64>()
            .map(JsonValue::Number)
            .map_err(|_| self.err(format!("invalid number '{text}'")))
    }

    fn string(&mut self) -> Result<String> {
        self.bump();
        let mut out = String::new();
        loop {
            match self.bump() {
                Some('"') => return Ok(out),
                Some('\\') => match self.bump() {
                    Some('n') => out.push('\n'),
                    Some('t') => out.push('\t'),
                    Some('r') => out.push('\r'),
                    Some('"') => out.push('"'),
                    Some('\\') => out.push('\\'),
                    Some('/') => out.push('/'),
                    Some('u') => {
                        let mut hex = String::new();
                        for _ in 0..4 {
                            hex.push(self.bump().ok_or_else(|| self.err("bad \\u escape"))?);
                        }
                        let code = u32::from_str_radix(&hex, 16)
                            .map_err(|_| self.err("bad \\u escape"))?;
                        out.push(char::from_u32(code).unwrap_or('\u{fffd}'));
                    }
                    _ => return Err(self.err("unknown escape in string")),
                },
                Some(c) => out.push(c),
                None => return Err(self.err("unterminated string")),
            }
        }
    }

    fn array(&mut self) -> Result<JsonValue> {
        self.bump();
        let mut items = Vec::new();
        loop {
            self.skip_ws();
            if self.peek() == Some(']') {
                self.bump();
                return Ok(JsonValue::Array(items));
            }
            items.push(self.value()?);
            self.skip_ws();
            match self.peek() {
                Some(',') => {
                    self.bump();
                }
                Some(']') => {}
                _ => return Err(self.err("expected ',' or ']' in array")),
            }
        }
    }

    fn object(&mut self) -> Result<JsonValue> {
        self.bump();
        let mut entries: Vec<(String, JsonValue)> = Vec::new();
        loop {
            self.skip_ws();
            if self.peek() == Some('}') {
                self.bump();
                return Ok(JsonValue::Object(entries));
            }
            if self.peek() != Some('"') {
                return Err(self.err("expected a string key"));
            }
            let key = self.string()?;
            if entries.iter().any(|(k, _)| *k == key) {
                return Err(self.err(format!("duplicate key '{key}' (GEP-0001-R018)")));
            }
            self.skip_ws();
            if self.bump() != Some(':') {
                return Err(self.err("expected ':' after key"));
            }
            self.skip_ws();
            let value = self.value()?;
            entries.push((key, value));
            self.skip_ws();
            match self.peek() {
                Some(',') => {
                    self.bump();
                }
                Some('}') => {}
                _ => return Err(self.err("expected ',' or '}' in object")),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_jsonc_with_comments_and_trailing_commas() {
        let v = parse_jsonc(
            "<test>",
            "{\n  // comment\n  \"source\": [\"src\",],\n  /* block */ \"outDir\": \"dist\",\n}",
        )
        .unwrap();
        match v {
            JsonValue::Object(entries) => {
                assert_eq!(entries.len(), 2);
                assert_eq!(entries[0].0, "source");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn rejects_duplicate_keys() {
        let err = parse_jsonc("<test>", "{\"a\": 1, \"a\": 2}").unwrap_err();
        assert!(err.message.contains("duplicate key"));
    }
}
