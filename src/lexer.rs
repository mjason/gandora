//! Lossy-free tokenizer for Gandora's Elixir-flavored surface.

use crate::diag::{Diagnostic, Result, Span};

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    Int(i64),
    Float(f64),
    Str(Vec<LexStrPart>),
    /// `~name(body)` — lowercase names interpolate, uppercase are raw
    Sigil(String, Vec<LexStrPart>),
    Atom(String),
    /// lowercase identifier, possibly ending in `?` or `!`
    Ident(String),
    /// Uppercase-leading module segment
    UpIdent(String),
    /// `name:` keyword key
    KwKey(String),
    /// operators and punctuation, e.g. "+", "|>", "(", "%{"
    Op(&'static str),
    /// structural keywords: do end fn else when and or not true false nil
    Kw(&'static str),
    Newline,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LexStrPart {
    Text(String),
    /// tokens of a `#{...}` interpolation, without the delimiters
    Interp(Vec<(Tok, Span)>),
}

const KEYWORDS: &[&str] = &[
    "do", "end", "fn", "else", "when", "and", "or", "not", "true", "false", "nil",
];

pub struct Lexer<'a> {
    file: &'a str,
    src: Vec<char>,
    pos: usize,
    line: u32,
    col: u32,
}

impl<'a> Lexer<'a> {
    pub fn new(file: &'a str, text: &str) -> Self {
        Lexer {
            file,
            src: text.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    pub fn tokenize(mut self) -> Result<Vec<(Tok, Span)>> {
        let mut out = Vec::new();
        loop {
            let (tok, span) = self.next_token()?;
            let is_eof = tok == Tok::Eof;
            // collapse duplicate newlines
            if tok == Tok::Newline {
                if matches!(out.last(), Some((Tok::Newline, _)) | None) {
                    if is_eof {
                        break;
                    }
                    continue;
                }
            }
            out.push((tok, span));
            if is_eof {
                break;
            }
        }
        Ok(out)
    }

    fn err(&self, msg: impl Into<String>) -> Diagnostic {
        Diagnostic::new(self.file, Span::new(self.line, self.col), msg)
    }

    fn peek(&self) -> Option<char> {
        self.src.get(self.pos).copied()
    }

    fn peek_at(&self, off: usize) -> Option<char> {
        self.src.get(self.pos + off).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.src.get(self.pos).copied()?;
        self.pos += 1;
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(c)
    }

    fn span(&self) -> Span {
        Span::new(self.line, self.col)
    }

    fn next_token(&mut self) -> Result<(Tok, Span)> {
        // skip horizontal whitespace and comments
        loop {
            match self.peek() {
                Some(' ') | Some('\t') | Some('\r') => {
                    self.bump();
                }
                Some('#') => {
                    while let Some(c) = self.peek() {
                        if c == '\n' {
                            break;
                        }
                        self.bump();
                    }
                }
                Some('\\') if self.peek_at(1) == Some('\n') => {
                    // explicit line continuation
                    self.bump();
                    self.bump();
                }
                _ => break,
            }
        }
        let span = self.span();
        let c = match self.peek() {
            None => return Ok((Tok::Eof, span)),
            Some(c) => c,
        };
        if c == '\n' || c == ';' {
            self.bump();
            return Ok((Tok::Newline, span));
        }
        if c.is_ascii_digit() {
            return Ok((self.lex_number()?, span));
        }
        if c == '"' {
            return Ok((self.lex_string()?, span));
        }
        if c == '~' {
            return Ok((self.lex_sigil()?, span));
        }
        if c == ':' {
            // atom, or `::` (unsupported), or lone `:` is an error
            if self.peek_at(1) == Some('"') {
                self.bump();
                let tok = self.lex_string()?;
                if let Tok::Str(parts) = tok {
                    let mut text = String::new();
                    for p in parts {
                        match p {
                            LexStrPart::Text(t) => text.push_str(&t),
                            LexStrPart::Interp(_) => {
                                return Err(self.err("interpolation is not allowed in atoms"))
                            }
                        }
                    }
                    return Ok((Tok::Atom(text), span));
                }
                unreachable!();
            }
            if matches!(self.peek_at(1), Some(c2) if is_ident_start(c2)) {
                self.bump();
                let name = self.lex_ident_chars();
                return Ok((Tok::Atom(name), span));
            }
            return Err(self.err("expected an atom after ':'"));
        }
        if is_ident_start(c) {
            let upper = c.is_uppercase();
            let name = self.lex_ident_chars();
            // keyword key: `name:` not followed by another ':'
            if self.peek() == Some(':') && self.peek_at(1) != Some(':') {
                self.bump();
                return Ok((Tok::KwKey(name), span));
            }
            if !upper {
                if let Some(kw) = KEYWORDS.iter().find(|k| **k == name) {
                    return Ok((Tok::Kw(kw), span));
                }
                return Ok((Tok::Ident(name), span));
            }
            return Ok((Tok::UpIdent(name), span));
        }
        // operators, longest first
        const OPS: &[&str] = &[
            "|>", "++", "<>", "..", "->", "<-", "=>", "==", "!=", "<=", ">=", "//", "%{", "(", ")",
            "[", "]", "{", "}", ",", ".", "+", "-", "*", "/", "<", ">", "=", "|", "^", "&", "@",
            "%", "!",
        ];
        for op in OPS {
            if self.matches(op) {
                for _ in 0..op.chars().count() {
                    self.bump();
                }
                return Ok((Tok::Op(op), span));
            }
        }
        Err(self.err(format!("unexpected character '{c}'")))
    }

    fn matches(&self, s: &str) -> bool {
        s.chars()
            .enumerate()
            .all(|(i, ch)| self.peek_at(i) == Some(ch))
    }

    fn lex_ident_chars(&mut self) -> String {
        let mut name = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                name.push(c);
                self.bump();
            } else {
                break;
            }
        }
        if matches!(self.peek(), Some('?') | Some('!')) {
            // `!` only belongs to the name when not part of `!=`
            if self.peek() == Some('?') || self.peek_at(1) != Some('=') {
                name.push(self.bump().unwrap());
            }
        }
        name
    }

    fn lex_number(&mut self) -> Result<Tok> {
        let mut text = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || c == '_' {
                if c != '_' {
                    text.push(c);
                }
                self.bump();
            } else {
                break;
            }
        }
        let mut is_float = false;
        if self.peek() == Some('.') && matches!(self.peek_at(1), Some(d) if d.is_ascii_digit()) {
            is_float = true;
            text.push('.');
            self.bump();
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() || c == '_' {
                    if c != '_' {
                        text.push(c);
                    }
                    self.bump();
                } else {
                    break;
                }
            }
        }
        if matches!(self.peek(), Some('e') | Some('E'))
            && matches!(self.peek_at(1), Some(d) if d.is_ascii_digit() || d == '+' || d == '-')
        {
            is_float = true;
            text.push('e');
            self.bump();
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() || c == '+' || c == '-' {
                    text.push(c);
                    self.bump();
                } else {
                    break;
                }
            }
        }
        if is_float {
            text.parse::<f64>()
                .map(Tok::Float)
                .map_err(|_| self.err(format!("invalid float literal '{text}'")))
        } else {
            text.parse::<i64>()
                .map(Tok::Int)
                .map_err(|_| self.err(format!("invalid integer literal '{text}'")))
        }
    }

    fn lex_string(&mut self) -> Result<Tok> {
        self.bump(); // opening quote
        // heredoc: """ ... """ (Elixir-style multi-line string)
        let heredoc = self.matches("\"\"");
        if heredoc {
            self.bump();
            self.bump();
        }
        let mut parts: Vec<LexStrPart> = Vec::new();
        let mut text = String::new();
        loop {
            let c = self
                .peek()
                .ok_or_else(|| self.err("unterminated string literal"))?;
            match c {
                '"' if heredoc => {
                    if self.matches("\"\"\"") {
                        self.bump();
                        self.bump();
                        self.bump();
                        break;
                    }
                    text.push(c);
                    self.bump();
                }
                '"' => {
                    self.bump();
                    break;
                }
                '\\' => {
                    self.bump();
                    let esc = self
                        .bump()
                        .ok_or_else(|| self.err("unterminated escape sequence"))?;
                    match esc {
                        'n' => text.push('\n'),
                        't' => text.push('\t'),
                        'r' => text.push('\r'),
                        '0' => text.push('\0'),
                        '\\' => text.push('\\'),
                        '"' => text.push('"'),
                        '#' => text.push('#'),
                        'e' => text.push('\u{1b}'),
                        other => {
                            return Err(self.err(format!("unknown escape sequence '\\{other}'")))
                        }
                    }
                }
                '#' if self.peek_at(1) == Some('{') => {
                    if !text.is_empty() {
                        parts.push(LexStrPart::Text(std::mem::take(&mut text)));
                    }
                    self.bump();
                    self.bump();
                    let mut depth = 1usize;
                    let mut inner = String::new();
                    loop {
                        let ic = self
                            .peek()
                            .ok_or_else(|| self.err("unterminated interpolation"))?;
                        if ic == '{' {
                            depth += 1;
                        } else if ic == '}' {
                            depth -= 1;
                            if depth == 0 {
                                self.bump();
                                break;
                            }
                        } else if ic == '"' {
                            // nested string: copy it verbatim including escapes
                            inner.push(self.bump().unwrap());
                            loop {
                                let sc = self
                                    .peek()
                                    .ok_or_else(|| self.err("unterminated string literal"))?;
                                inner.push(self.bump().unwrap());
                                if sc == '\\' {
                                    if let Some(n) = self.peek() {
                                        inner.push(n);
                                        self.bump();
                                    }
                                } else if sc == '"' {
                                    break;
                                }
                            }
                            continue;
                        }
                        inner.push(self.bump().unwrap());
                    }
                    let toks = Lexer::new(self.file, &inner).tokenize()?;
                    parts.push(LexStrPart::Interp(toks));
                }
                _ => {
                    text.push(c);
                    self.bump();
                }
            }
        }
        if !text.is_empty() || parts.is_empty() {
            parts.push(LexStrPart::Text(text));
        }
        Ok(Tok::Str(parts))
    }

    /// `~name<delim>body<delim>` (GEP-0005-R001/R002).
    fn lex_sigil(&mut self) -> Result<Tok> {
        self.bump(); // '~'
        let mut name = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphabetic() {
                name.push(c);
                self.bump();
            } else {
                break;
            }
        }
        if name.is_empty() {
            return Err(self.err("expected a sigil name after '~'"));
        }
        // built-ins (~w ~s ~r) interpolate with #{}; every other name is an
        // embedded-language sigil with a raw body (GEP-0009-R001)
        let raw = !matches!(name.as_str(), "w" | "s" | "r");
        let open = self
            .peek()
            .ok_or_else(|| self.err("expected a sigil delimiter"))?;
        let (close, nests) = match open {
            '(' => (')', true),
            '[' => (']', true),
            '{' => ('}', true),
            '/' => ('/', false),
            '|' => ('|', false),
            '"' => ('"', false),
            other => {
                return Err(self.err(format!(
                    "'{other}' is not a sigil delimiter (use ( [ {{ / | \")"
                )))
            }
        };
        self.bump();
        let triple = close == '"' && self.matches("\"\"");
        if triple {
            self.bump();
            self.bump();
        }
        let mut depth = 1usize;
        let mut parts: Vec<LexStrPart> = Vec::new();
        let mut text = String::new();
        loop {
            let c = self
                .peek()
                .ok_or_else(|| self.err("unterminated sigil"))?;
            if triple {
                if c == '"' && self.matches("\"\"\"") {
                    self.bump();
                    self.bump();
                    self.bump();
                    break;
                }
            } else if c == '\\' {
                // only the closing delimiter (and a backslash before it)
                // may be escaped; everything else passes through (R002)
                let next = self.peek_at(1);
                if next == Some(close) || (next == Some('\\') && self.peek_at(2) == Some(close)) {
                    self.bump();
                    text.push(self.bump().unwrap());
                    continue;
                }
                text.push(c);
                self.bump();
                continue;
            } else if nests && c == open {
                depth += 1;
                text.push(c);
                self.bump();
                continue;
            } else if c == close {
                depth -= 1;
                if depth == 0 {
                    self.bump();
                    break;
                }
                text.push(c);
                self.bump();
                continue;
            }
            if !raw && c == '#' && self.peek_at(1) == Some('{') {
                if !text.is_empty() {
                    parts.push(LexStrPart::Text(std::mem::take(&mut text)));
                }
                self.bump();
                self.bump();
                let mut idepth = 1usize;
                let mut inner = String::new();
                loop {
                    let ic = self
                        .peek()
                        .ok_or_else(|| self.err("unterminated interpolation"))?;
                    if ic == '{' {
                        idepth += 1;
                    } else if ic == '}' {
                        idepth -= 1;
                        if idepth == 0 {
                            self.bump();
                            break;
                        }
                    }
                    inner.push(self.bump().unwrap());
                }
                let toks = Lexer::new(self.file, &inner).tokenize()?;
                parts.push(LexStrPart::Interp(toks));
                continue;
            }
            text.push(c);
            self.bump();
        }
        if !text.is_empty() || parts.is_empty() {
            parts.push(LexStrPart::Text(text));
        }
        if matches!(self.peek(), Some(m) if m.is_alphabetic()) {
            return Err(self.err("sigil modifiers are not supported (GEP-0005-R003)"));
        }
        Ok(Tok::Sigil(name, parts))
    }
}

fn is_ident_start(c: char) -> bool {
    c == '_' || c.is_alphabetic()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toks(src: &str) -> Vec<Tok> {
        Lexer::new("<test>", src)
            .tokenize()
            .unwrap()
            .into_iter()
            .map(|(t, _)| t)
            .collect()
    }

    #[test]
    fn lexes_basic_tokens() {
        assert_eq!(
            toks("x = 1_000 + 2.5"),
            vec![
                Tok::Ident("x".into()),
                Tok::Op("="),
                Tok::Int(1000),
                Tok::Op("+"),
                Tok::Float(2.5),
                Tok::Eof
            ]
        );
    }

    #[test]
    fn lexes_atoms_and_keyword_keys() {
        assert_eq!(
            toks(":ok :\"os.path\" a: 1"),
            vec![
                Tok::Atom("ok".into()),
                Tok::Atom("os.path".into()),
                Tok::KwKey("a".into()),
                Tok::Int(1),
                Tok::Eof
            ]
        );
    }

    #[test]
    fn lexes_predicate_and_bang_names() {
        assert_eq!(
            toks("empty? save! x != y"),
            vec![
                Tok::Ident("empty?".into()),
                Tok::Ident("save!".into()),
                Tok::Ident("x".into()),
                Tok::Op("!="),
                Tok::Ident("y".into()),
                Tok::Eof
            ]
        );
    }

    #[test]
    fn lexes_interpolated_string() {
        let t = toks("\"a#{x + 1}b\"");
        match &t[0] {
            Tok::Str(parts) => {
                assert_eq!(parts.len(), 3);
                assert_eq!(parts[0], LexStrPart::Text("a".into()));
                assert!(matches!(&parts[1], LexStrPart::Interp(ts) if ts.len() == 4));
                assert_eq!(parts[2], LexStrPart::Text("b".into()));
            }
            other => panic!("expected string, got {other:?}"),
        }
    }

    #[test]
    fn lexes_operators_and_keywords() {
        assert_eq!(
            toks("xs |> f() do end"),
            vec![
                Tok::Ident("xs".into()),
                Tok::Op("|>"),
                Tok::Ident("f".into()),
                Tok::Op("("),
                Tok::Op(")"),
                Tok::Kw("do"),
                Tok::Kw("end"),
                Tok::Eof
            ]
        );
    }

    #[test]
    fn comments_and_newlines() {
        assert_eq!(
            toks("a # comment\n\n\nb"),
            vec![
                Tok::Ident("a".into()),
                Tok::Newline,
                Tok::Ident("b".into()),
                Tok::Eof
            ]
        );
    }
}

#[cfg(test)]
mod sigil_tests {
    use super::*;

    fn toks(src: &str) -> Vec<Tok> {
        Lexer::new("<test>", src)
            .tokenize()
            .unwrap()
            .into_iter()
            .map(|(t, _)| t)
            .collect()
    }

    #[test]
    fn lexes_word_sigil() {
        match &toks("~w(one two three)")[0] {
            Tok::Sigil(name, parts) => {
                assert_eq!(name, "w");
                assert_eq!(parts, &vec![LexStrPart::Text("one two three".into())]);
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn lexes_regex_sigil_with_backslashes() {
        match &toks(r"~r/\d+/")[0] {
            Tok::Sigil(name, parts) => {
                assert_eq!(name, "r");
                assert_eq!(parts, &vec![LexStrPart::Text(r"\d+".into())]);
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn raw_sigil_ignores_interpolation() {
        match &toks("~python(sum(i for i in #{x}))")[0] {
            Tok::Sigil(name, parts) => {
                assert_eq!(name, "python");
                assert_eq!(
                    parts,
                    &vec![LexStrPart::Text("sum(i for i in #{x})".into())]
                );
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn brackets_nest_and_delims_escape() {
        match &toks(r"~s(a (b) c)")[0] {
            Tok::Sigil(_, parts) => {
                assert_eq!(parts, &vec![LexStrPart::Text("a (b) c".into())]);
            }
            other => panic!("{other:?}"),
        }
        match &toks(r"~s/a\/b/")[0] {
            Tok::Sigil(_, parts) => {
                assert_eq!(parts, &vec![LexStrPart::Text("a/b".into())]);
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn interpolating_sigil_splits_parts() {
        match &toks("~w(a #{x} b)")[0] {
            Tok::Sigil(_, parts) => assert_eq!(parts.len(), 3),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn rejects_modifiers() {
        let err = Lexer::new("<test>", "~r/x/i").tokenize().unwrap_err();
        assert!(err.message.contains("GEP-0005-R003"), "{}", err.message);
    }
}

#[cfg(test)]
mod heredoc_tests {
    use super::*;

    #[test]
    fn lexes_heredoc_strings() {
        let toks = Lexer::new("<test>", "\"\"\"\nline \"quoted\" one\nline two\n\"\"\"")
            .tokenize()
            .unwrap();
        match &toks[0].0 {
            Tok::Str(parts) => match &parts[0] {
                LexStrPart::Text(t) => {
                    assert!(t.contains("line \"quoted\" one\nline two"), "{t}")
                }
                other => panic!("{other:?}"),
            },
            other => panic!("{other:?}"),
        }
    }
}
