//! The Gandora AST.
//!
//! The shape follows Elixir's quoted form (GEP-0002-R002): every composite
//! construct is a call `{name, meta, args}`; literals represent themselves.
//! Special forms (`defmodule`, `def`, `if`, `case`, `quote`, ...) are calls
//! whose callee is a plain name, with `do`/`else` blocks carried as trailing
//! keyword arguments whose values are `__block__` calls.

use crate::diag::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    Int(i64),
    Float(f64),
    Bool(bool),
    Nil,
    /// A string literal as interpolation parts.
    Str(Vec<StrPart>),
    Atom(String),
    /// `$module` — a Python module reference, a first-class module
    /// object at runtime (GEP-0003-R001/R002). The flag records the
    /// explicit-boundary form `$(...)`, which the dotted-chain
    /// heuristic must never extend (GEP-0003-R010).
    PyRef(String, bool),
    /// A lowercase identifier reference. `ctx` is the hygiene context:
    /// `None` for user-written code, `Some(id)` for macro-template names.
    Var(String, Option<u64>),
    /// A module path such as `App.Hello`, stored as segments.
    Alias(Vec<String>),
    List(Vec<Term>),
    Tuple(Vec<Term>),
    /// `%{k => v, ...}`; `a: 1` sugar arrives as an Atom key.
    Map(Vec<(Term, Term)>),
    /// A keyword pair `name: value` (inside lists / trailing arguments).
    Pair(String, Box<Term>),
    Call(Box<Call>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    pub callee: Callee,
    pub args: Vec<Term>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Callee {
    /// `foo(...)`, operators (`+`), and special forms (`if`, `def`, ...).
    Name(String),
    /// `base.name(...)` — remote atom calls, module calls, method calls.
    /// `base.name` without parens parses as `Dot` with `is_call = false`.
    Dot {
        base: Box<Term>,
        name: String,
        is_call: bool,
    },
    /// `f.(x)` — calling an anonymous function value.
    Apply(Box<Term>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum StrPart {
    Text(String),
    Interp(Box<Term>),
}

impl Term {
    pub fn call(name: &str, args: Vec<Term>, span: Span) -> Term {
        Term::Call(Box::new(Call {
            callee: Callee::Name(name.to_string()),
            args,
            span,
        }))
    }

    pub fn block(stmts: Vec<Term>, span: Span) -> Term {
        Term::call("__block__", stmts, span)
    }

    pub fn span(&self) -> Span {
        match self {
            Term::Call(c) => c.span,
            _ => Span::default(),
        }
    }

    /// The statements of a `__block__`, or the term itself as a single one.
    pub fn as_block(&self) -> Vec<Term> {
        match self {
            Term::Call(c) if matches!(&c.callee, Callee::Name(n) if n == "__block__") => {
                c.args.clone()
            }
            other => vec![other.clone()],
        }
    }

    pub fn is_call_named(&self, name: &str) -> bool {
        matches!(self, Term::Call(c)
            if matches!(&c.callee, Callee::Name(n) if n == name))
    }

    /// Plain string literal contents when the term is an uninterpolated string.
    pub fn as_plain_str(&self) -> Option<String> {
        match self {
            Term::Str(parts) => {
                let mut out = String::new();
                for p in parts {
                    match p {
                        StrPart::Text(t) => out.push_str(t),
                        StrPart::Interp(_) => return None,
                    }
                }
                Some(out)
            }
            _ => None,
        }
    }

    /// Look up `key:` in a trailing keyword segment of `args`.
    pub fn keyword_arg<'a>(args: &'a [Term], key: &str) -> Option<&'a Term> {
        for arg in args.iter().rev() {
            match arg {
                Term::Pair(k, v) if k == key => return Some(v),
                Term::List(items) => {
                    for item in items {
                        if let Term::Pair(k, v) = item {
                            if k == key {
                                return Some(v);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        None
    }
}
