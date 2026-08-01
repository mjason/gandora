//! Diagnostics with file/line/column spans (GEP-0001-R023).

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub line: u32,
    pub col: u32,
}

impl Span {
    pub fn new(line: u32, col: u32) -> Self {
        Span { line, col }
    }
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub file: String,
    pub span: Span,
    pub message: String,
    /// Macro-expansion origin chain, outermost call site first (GEP-0002-R005).
    pub origin: Vec<(String, Span)>,
}

impl Diagnostic {
    pub fn new(file: impl Into<String>, span: Span, message: impl Into<String>) -> Self {
        Diagnostic {
            file: file.into(),
            span,
            message: message.into(),
            origin: Vec::new(),
        }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}: error: {}",
            self.file, self.span.line, self.span.col, self.message
        )?;
        for (label, span) in &self.origin {
            write!(f, "\n  in expansion of {} at {}:{}", label, span.line, span.col)?;
        }
        Ok(())
    }
}

pub type Result<T> = std::result::Result<T, Diagnostic>;
