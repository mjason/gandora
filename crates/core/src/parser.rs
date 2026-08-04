//! Recursive-descent / Pratt parser producing `Term`s.
//!
//! Follows Elixir's surface grammar (GEP-0001-R005/R006): paren-less calls
//! in command position, `do ... end` blocks carried as trailing `do:`/`else:`
//! keyword arguments, and `->` clause sequences inside blocks.

use crate::ast::{Call, Callee, StrPart, Term};
use crate::diag::{Diagnostic, Result, Span};
use crate::lexer::{LexStrPart, Lexer, Tok};

pub fn parse_file(file: &str, text: &str) -> Result<Term> {
    let toks = Lexer::new(file, text).tokenize()?;
    let mut p = Parser {
        file: file.to_string(),
        toks,
        idx: 0,
    };
    let stmts = p.parse_stmts_until(&[Tok::Eof])?;
    p.expect_tok(&Tok::Eof)?;
    Ok(Term::block(stmts, Span::new(1, 1)))
}

/// Parse a single expression (used by tests and the macro engine).
pub fn parse_expr_str(file: &str, text: &str) -> Result<Term> {
    let toks = Lexer::new(file, text).tokenize()?;
    let mut p = Parser {
        file: file.to_string(),
        toks,
        idx: 0,
    };
    p.skip_newlines();
    let t = p.parse_stmt()?;
    p.skip_newlines();
    p.expect_tok(&Tok::Eof)?;
    Ok(t)
}

struct Parser {
    file: String,
    toks: Vec<(Tok, Span)>,
    idx: usize,
}

const P_MATCH: u8 = 1; // =
const P_WHEN: u8 = 2; // when
const P_ARROW_L: u8 = 3; // <-
const P_OR: u8 = 4;
const P_AND: u8 = 5;
const P_EQ: u8 = 6; // == !=
const P_CMP: u8 = 7; // < > <= >=
const P_PIPE: u8 = 8; // |>
const P_CONCAT: u8 = 9; // ++ <>
const P_RANGE: u8 = 10; // ..
const P_ADD: u8 = 11;
const P_MUL: u8 = 12;

fn infix_prec(op: &str) -> Option<(u8, bool)> {
    // (precedence, right_assoc)
    Some(match op {
        "=" => (P_MATCH, true),
        "::" => (P_MATCH, true),
        "|" => (P_MATCH, false),
        "when" => (P_WHEN, true),
        "\\\\" => (P_WHEN, false),
        "<-" => (P_ARROW_L, false),
        "or" => (P_OR, false),
        "and" => (P_AND, false),
        "==" | "!=" => (P_EQ, false),
        "in" => (P_CMP, false),
        "<" | ">" | "<=" | ">=" => (P_CMP, false),
        "|>" => (P_PIPE, false),
        "++" | "<>" => (P_CONCAT, true),
        ".." => (P_RANGE, false),
        "+" | "-" => (P_ADD, false),
        "*" | "/" | "//" => (P_MUL, false),
        _ => return None,
    })
}

impl Parser {
    fn peek(&self) -> &Tok {
        &self.toks[self.idx.min(self.toks.len() - 1)].0
    }

    fn peek_at(&self, off: usize) -> &Tok {
        &self.toks[(self.idx + off).min(self.toks.len() - 1)].0
    }

    fn span(&self) -> Span {
        self.toks[self.idx.min(self.toks.len() - 1)].1
    }

    fn bump(&mut self) -> Tok {
        let t = self.toks[self.idx.min(self.toks.len() - 1)].0.clone();
        if self.idx < self.toks.len() {
            self.idx += 1;
        }
        t
    }

    fn err(&self, msg: impl Into<String>) -> Diagnostic {
        Diagnostic::new(&self.file, self.span(), msg)
    }

    fn err_at(&self, span: Span, msg: impl Into<String>) -> Diagnostic {
        Diagnostic::new(&self.file, span, msg)
    }

    fn expect_tok(&mut self, t: &Tok) -> Result<()> {
        if self.peek() == t {
            self.bump();
            Ok(())
        } else {
            Err(self.err(format!("expected {t:?}, found {:?}", self.peek())))
        }
    }

    fn skip_newlines(&mut self) {
        while *self.peek() == Tok::Newline {
            self.bump();
        }
    }

    fn at_terminator(&self, terms: &[Tok]) -> bool {
        terms.iter().any(|t| t == self.peek())
    }

    // ---- statements -----------------------------------------------------

    fn parse_stmts_until(&mut self, terms: &[Tok]) -> Result<Vec<Term>> {
        let mut out = Vec::new();
        loop {
            self.skip_newlines();
            if self.at_terminator(terms) || *self.peek() == Tok::Eof {
                break;
            }
            out.push(self.parse_stmt()?);
            match self.peek() {
                Tok::Newline => {
                    self.bump();
                }
                t if terms.contains(t) || *t == Tok::Eof => break,
                _ => return Err(self.err(format!("expected end of statement, found {:?}", self.peek()))),
            }
        }
        Ok(out)
    }

    fn parse_stmt(&mut self) -> Result<Term> {
        self.parse_expr(0, true)
    }

    // ---- expressions ----------------------------------------------------

    fn parse_expr(&mut self, min_prec: u8, command: bool) -> Result<Term> {
        let mut lhs = self.parse_unary(command)?;
        loop {
            // a line ending followed by `|>` continues the expression
            if *self.peek() == Tok::Newline {
                let mut j = self.idx;
                while j < self.toks.len() && self.toks[j].0 == Tok::Newline {
                    j += 1;
                }
                if j < self.toks.len() && self.toks[j].0 == Tok::Op("|>") {
                    self.idx = j;
                } else {
                    break;
                }
            }
            let (op, prec, right): (String, u8, bool) = match self.peek() {
                Tok::Op(op) => match infix_prec(op) {
                    Some((p, r)) => ((*op).to_string(), p, r),
                    None => break,
                },
                Tok::Kw(k @ ("and" | "or" | "when" | "in")) => {
                    let (p, r) = infix_prec(k).unwrap();
                    ((*k).to_string(), p, r)
                }
                _ => break,
            };
            if prec < min_prec {
                break;
            }
            let span = self.span();
            self.bump();
            self.skip_newlines();
            // `x |> .method(args)` pipes into a method call on the piped
            // value itself (GEP-0001-R025)
            if op == "|>" && *self.peek() == Tok::Op(".") {
                lhs = self.parse_postfix(lhs)?;
                continue;
            }
            let next_min = if right { prec } else { prec + 1 };
            let rhs = self.parse_expr(next_min, command && op == "=")?;
            if op == "|>" {
                lhs = pipe_into(lhs, rhs, span, &self.file)?;
            } else {
                lhs = Term::call(&op, vec![lhs, rhs], span);
            }
        }
        Ok(lhs)
    }

    fn parse_unary(&mut self, command: bool) -> Result<Term> {
        let span = self.span();
        match self.peek().clone() {
            Tok::Kw("not") => {
                self.bump();
                let operand = self.parse_unary(false)?;
                return Ok(Term::call("not", vec![operand], span));
            }
            Tok::Op("-") => {
                self.bump();
                let operand = self.parse_unary(false)?;
                return Ok(Term::call("-", vec![operand], span));
            }
            Tok::Op("^") => {
                self.bump();
                let operand = self.parse_unary(false)?;
                return Ok(Term::call("^", vec![operand], span));
            }
            Tok::Op("&") => {
                self.bump();
                if let Tok::Int(n) = self.peek() {
                    let n = *n;
                    self.bump();
                    let base = Term::Var(format!("&{n}"), None);
                    return self.parse_postfix(base);
                }
                let operand = self.parse_expr(P_ADD, false)?;
                return Ok(Term::call("&", vec![operand], span));
            }
            Tok::Op("@") => {
                self.bump();
                let name = match self.bump() {
                    Tok::Ident(n) => n,
                    other => {
                        return Err(self.err_at(
                            span,
                            format!("expected attribute name after '@', found {other:?}"),
                        ))
                    }
                };
                let attr = format!("@{name}");
                if self.starts_expr() {
                    let args = self.parse_command_args()?;
                    return Ok(Term::call(&attr, args, span));
                }
                // an attribute read may continue with postfix access:
                // `@app.route("/")` (GEP-0004-R011)
                return self.parse_postfix(Term::call(&attr, vec![], span));
            }
            _ => {}
        }
        let primary = self.parse_primary(command)?;
        let mut t = self.parse_postfix(primary)?;
        // a do-block after a paren call in command position belongs to it:
        // `foo(x) do ... end`
        if command && *self.peek() == Tok::Kw("do") {
            if let Term::Call(ref mut c) = t {
                if matches!(c.callee, Callee::Name(_)) {
                    let mut block_args = self.parse_do_block()?;
                    c.args.append(&mut block_args);
                }
            }
        }
        Ok(t)
    }

    fn parse_primary(&mut self, command: bool) -> Result<Term> {
        let span = self.span();
        match self.bump() {
            Tok::Int(n) => Ok(Term::Int(n)),
            Tok::Float(f) => Ok(Term::Float(f)),
            Tok::PyRef(name, bounded) => Ok(Term::PyRef(name, bounded)),
            Tok::Kw("true") => Ok(Term::Bool(true)),
            Tok::Kw("false") => Ok(Term::Bool(false)),
            Tok::Kw("nil") => Ok(Term::Nil),
            Tok::Str(parts) => Ok(Term::Str(self.convert_str_parts(parts)?)),
            Tok::Sigil(name, parts) => {
                let body = Term::Str(self.convert_str_parts(parts)?);
                // `$python` / `%json` carry their symbol already; the
                // text family gets the `~` prefix (GEP-0009)
                let callee = if name.starts_with('$') || name.starts_with('%') {
                    name.clone()
                } else {
                    format!("~{name}")
                };
                Ok(Term::call(&callee, vec![body], span))
            }
            Tok::Atom(a) => Ok(Term::Atom(a)),
            Tok::KwKey(k) => {
                // a keyword pair in argument / list position: `k: value`
                self.skip_newlines();
                let value = self.parse_expr(P_WHEN, false)?;
                Ok(Term::Pair(k, Box::new(value)))
            }
            Tok::Ident(name) => {
                let var = Term::Var(name.clone(), None);
                // paren call binds directly: `f(...)`
                if *self.peek() == Tok::Op("(") {
                    return Ok(var);
                }
                // block constructs are expressions everywhere, as in Elixir:
                // `{1, if ok do 2 else 3 end}`, `f(case x do ... end)`
                let block_form = matches!(
                    name.as_str(),
                    "if" | "unless" | "case" | "cond" | "with" | "try" | "for" | "quote"
                );
                if (command || block_form) && self.starts_expr() {
                    let args = self.parse_command_args()?;
                    let mut call = Call {
                        callee: Callee::Name(name),
                        args,
                        span,
                    };
                    if !command && *self.peek() == Tok::Kw("do") {
                        let mut block_args = self.parse_do_block()?;
                        call.args.append(&mut block_args);
                    }
                    return Ok(Term::Call(Box::new(call)));
                }
                Ok(var)
            }
            Tok::UpIdent(seg) => {
                let mut segs = vec![seg];
                while *self.peek() == Tok::Op(".") {
                    if let Tok::UpIdent(next) = self.peek_at(1) {
                        let next = next.clone();
                        self.bump();
                        self.bump();
                        segs.push(next);
                    } else {
                        break;
                    }
                }
                Ok(Term::Alias(segs))
            }
            Tok::Kw("fn") => {
                let clauses = self.parse_clauses(&[Tok::Kw("end")])?;
                self.expect_tok(&Tok::Kw("end"))?;
                Ok(Term::call("fn", clauses, span))
            }
            Tok::Op("(") => {
                self.skip_newlines();
                let inner = self.parse_stmt()?;
                self.skip_newlines();
                self.expect_tok(&Tok::Op(")"))?;
                Ok(inner)
            }
            Tok::Op("[") => {
                let items = self.parse_bracket_items(&Tok::Op("]"))?;
                Ok(Term::List(items))
            }
            Tok::Op("{") => {
                let items = self.parse_bracket_items(&Tok::Op("}"))?;
                Ok(Term::Tuple(items))
            }
            Tok::Op("%") => {
                // struct literal / update: %Mod{...} (GEP-0004-R004/R006)
                let alias = match self.parse_primary(false)? {
                    t @ Term::Alias(_) => t,
                    other => {
                        return Err(self.err_at(
                            span,
                            format!("expected a module name after '%', found {other:?}"),
                        ))
                    }
                };
                self.expect_tok(&Tok::Op("{"))?;
                self.skip_newlines();
                // update form: %Mod{expr | field: v}
                if !matches!(self.peek(), Tok::KwKey(_)) && *self.peek() != Tok::Op("}") {
                    let base = self.parse_expr(P_WHEN, false)?;
                    self.skip_newlines();
                    self.expect_tok(&Tok::Op("|"))?;
                    self.skip_newlines();
                    let pairs = self.parse_struct_pairs()?;
                    return Ok(Term::call(
                        "%struct_update%",
                        vec![alias, base, Term::List(pairs)],
                        span,
                    ));
                }
                let pairs = self.parse_struct_pairs()?;
                Ok(Term::call("%struct%", vec![alias, Term::List(pairs)], span))
            }
            Tok::Op("%{") => {
                // update form: %{expr | key: v} (GEP-0004-R008)
                self.skip_newlines();
                if !matches!(self.peek(), Tok::KwKey(_)) && *self.peek() != Tok::Op("}") {
                    let save = self.idx;
                    let base = self.parse_expr(P_WHEN, false)?;
                    self.skip_newlines();
                    if *self.peek() == Tok::Op("|") {
                        self.bump();
                        self.skip_newlines();
                        let pairs = self.parse_struct_pairs()?;
                        return Ok(Term::call(
                            "%map_update%",
                            vec![base, Term::List(pairs)],
                            span,
                        ));
                    }
                    self.idx = save;
                }
                let mut entries = Vec::new();
                self.skip_newlines();
                while *self.peek() != Tok::Op("}") {
                    self.skip_newlines();
                    if let Tok::KwKey(k) = self.peek().clone() {
                        self.bump();
                        self.skip_newlines();
                        let v = self.parse_expr(P_WHEN, false)?;
                        entries.push((Term::Atom(k), v));
                    } else {
                        let k = self.parse_expr(P_WHEN, false)?;
                        self.skip_newlines();
                        self.expect_tok(&Tok::Op("=>"))?;
                        self.skip_newlines();
                        let v = self.parse_expr(P_WHEN, false)?;
                        entries.push((k, v));
                    }
                    self.skip_newlines();
                    if *self.peek() == Tok::Op(",") {
                        self.bump();
                        self.skip_newlines();
                    } else {
                        break;
                    }
                }
                self.expect_tok(&Tok::Op("}"))?;
                Ok(Term::Map(entries))
            }
            other => Err(self.err_at(span, format!("unexpected token {other:?}"))),
        }
    }

    fn convert_str_parts(&self, parts: Vec<LexStrPart>) -> Result<Vec<StrPart>> {
        let mut out = Vec::new();
        for p in parts {
            match p {
                LexStrPart::Text(t) => out.push(StrPart::Text(t)),
                LexStrPart::Interp(toks) => {
                    let mut sub = Parser {
                        file: self.file.clone(),
                        toks,
                        idx: 0,
                    };
                    sub.skip_newlines();
                    let t = sub.parse_expr(0, false)?;
                    sub.skip_newlines();
                    sub.expect_tok(&Tok::Eof)?;
                    out.push(StrPart::Interp(Box::new(t)));
                }
            }
        }
        Ok(out)
    }

    /// `field: value` pairs up to a closing `}` (struct/map update forms).
    fn parse_struct_pairs(&mut self) -> Result<Vec<Term>> {
        let mut pairs = Vec::new();
        while *self.peek() != Tok::Op("}") {
            self.skip_newlines();
            match self.peek().clone() {
                Tok::KwKey(k) => {
                    self.bump();
                    self.skip_newlines();
                    let v = self.parse_expr(P_WHEN, false)?;
                    pairs.push(Term::Pair(k, Box::new(v)));
                }
                other => {
                    return Err(self.err(format!(
                        "expected a `field: value` pair, found {other:?}"
                    )))
                }
            }
            self.skip_newlines();
            if *self.peek() == Tok::Op(",") {
                self.bump();
                self.skip_newlines();
            } else {
                break;
            }
        }
        self.skip_newlines();
        self.expect_tok(&Tok::Op("}"))?;
        Ok(pairs)
    }

    fn parse_bracket_items(&mut self, close: &Tok) -> Result<Vec<Term>> {
        let mut items = Vec::new();
        self.skip_newlines();
        while self.peek() != close {
            self.skip_newlines();
            let item = self.parse_expr(P_WHEN, false)?;
            // cons cell `head | tail` inside list literals
            if *self.peek() == Tok::Op("|") {
                let span = self.span();
                self.bump();
                self.skip_newlines();
                let tail = self.parse_expr(P_WHEN, false)?;
                items.push(Term::call("|", vec![item, tail], span));
            } else {
                items.push(item);
            }
            self.skip_newlines();
            if *self.peek() == Tok::Op(",") {
                self.bump();
                self.skip_newlines();
            } else {
                break;
            }
        }
        self.skip_newlines();
        self.expect_tok(close)?;
        Ok(items)
    }

    // ---- postfix: dot access, calls ------------------------------------

    fn parse_postfix(&mut self, mut base: Term) -> Result<Term> {
        loop {
            match self.peek().clone() {
                Tok::Op("(") => {
                    let span = self.span();
                    let args = self.parse_paren_args()?;
                    base = match base {
                        Term::Var(name, None) => Term::Call(Box::new(Call {
                            callee: Callee::Name(name),
                            args,
                            span,
                        })),
                        other => Term::Call(Box::new(Call {
                            callee: Callee::Apply(Box::new(other)),
                            args,
                            span,
                        })),
                    };
                }
                Tok::Op(".") => {
                    let span = self.span();
                    match self.peek_at(1).clone() {
                        // Python class names are uppercase: `$collections.OrderedDict()`
                        Tok::UpIdent(name) if !matches!(base, Term::Alias(_)) => {
                            self.bump();
                            self.bump();
                            let is_call = *self.peek() == Tok::Op("(");
                            let args = if is_call { self.parse_paren_args()? } else { vec![] };
                            base = Term::Call(Box::new(Call {
                                callee: Callee::Dot {
                                    base: Box::new(base),
                                    name,
                                    is_call,
                                },
                                args,
                                span,
                            }));
                        }
                        Tok::Ident(name) => {
                            self.bump();
                            self.bump();
                            let is_call = *self.peek() == Tok::Op("(");
                            if is_call {
                                let args = self.parse_paren_args()?;
                                base = Term::Call(Box::new(Call {
                                    callee: Callee::Dot {
                                        base: Box::new(base),
                                        name,
                                        is_call: true,
                                    },
                                    args,
                                    span,
                                }));
                            } else {
                                base = Term::Call(Box::new(Call {
                                    callee: Callee::Dot {
                                        base: Box::new(base),
                                        name,
                                        is_call: false,
                                    },
                                    args: vec![],
                                    span,
                                }));
                            }
                        }
                        Tok::Op("(") => {
                            // anonymous-function application `f.(x)`
                            self.bump();
                            let args = self.parse_paren_args()?;
                            base = Term::Call(Box::new(Call {
                                callee: Callee::Apply(Box::new(base)),
                                args,
                                span,
                            }));
                        }
                        _ => break,
                    }
                }
                _ => break,
            }
        }
        Ok(base)
    }

    fn parse_paren_args(&mut self) -> Result<Vec<Term>> {
        self.expect_tok(&Tok::Op("("))?;
        let mut args = Vec::new();
        self.skip_newlines();
        while *self.peek() != Tok::Op(")") {
            self.skip_newlines();
            args.push(self.parse_expr(0, false)?);
            self.skip_newlines();
            if *self.peek() == Tok::Op(",") {
                self.bump();
                self.skip_newlines();
            } else {
                break;
            }
        }
        self.skip_newlines();
        self.expect_tok(&Tok::Op(")"))?;
        Ok(args)
    }

    /// Whether the current token can begin a paren-less call argument.
    fn starts_expr(&self) -> bool {
        matches!(
            self.peek(),
            Tok::Int(_)
                | Tok::Float(_)
                | Tok::Str(_)
                | Tok::Sigil(_, _)
                | Tok::Atom(_)
                | Tok::PyRef(..)
                | Tok::Ident(_)
                | Tok::UpIdent(_)
                | Tok::KwKey(_)
                | Tok::Kw("fn")
                | Tok::Kw("true")
                | Tok::Kw("false")
                | Tok::Kw("nil")
                | Tok::Kw("not")
                | Tok::Kw("do")
                | Tok::Op("[")
                | Tok::Op("{")
                | Tok::Op("%{")
                | Tok::Op("&")
                | Tok::Op("^")
                | Tok::Op("@")
        )
    }

    fn parse_command_args(&mut self) -> Result<Vec<Term>> {
        let mut args = Vec::new();
        if *self.peek() != Tok::Kw("do") {
            loop {
                args.push(self.parse_expr(0, false)?);
                if *self.peek() == Tok::Op(",") {
                    self.bump();
                    self.skip_newlines();
                } else {
                    break;
                }
            }
        }
        if *self.peek() == Tok::Kw("do") {
            let mut block_args = self.parse_do_block()?;
            args.append(&mut block_args);
        }
        Ok(args)
    }

    // ---- do/end blocks and clauses -------------------------------------

    fn parse_do_block(&mut self) -> Result<Vec<Term>> {
        let span = self.span();
        self.expect_tok(&Tok::Kw("do"))?;
        let sections = [
            Tok::Kw("end"),
            Tok::Kw("else"),
            Tok::Kw("rescue"),
            Tok::Kw("after"),
        ];
        let body = self.parse_block_section(&sections)?;
        let mut out = vec![Term::Pair("do".into(), Box::new(body))];
        while let Tok::Kw(kw @ ("else" | "rescue" | "after")) = self.peek().clone() {
            self.bump();
            let section = self.parse_block_section(&sections)?;
            out.push(Term::Pair(kw.to_string(), Box::new(section)));
        }
        self.expect_tok(&Tok::Kw("end"))
            .map_err(|_| self.err_at(span, "missing 'end' for this 'do'".to_string()))?;
        Ok(out)
    }

    /// A block section is either a statement sequence or a `->` clause list.
    fn parse_block_section(&mut self, terms: &[Tok]) -> Result<Term> {
        let span = self.span();
        self.skip_newlines();
        if self.looks_like_clause_head(terms) {
            let clauses = self.parse_clauses(terms)?;
            return Ok(Term::call("__clauses__", clauses, span));
        }
        let stmts = self.parse_stmts_until(terms)?;
        Ok(Term::block(stmts, span))
    }

    fn looks_like_clause_head(&mut self, terms: &[Tok]) -> bool {
        let save = self.idx;
        let ok = self.try_parse_clause_head(terms).is_some();
        self.idx = save;
        ok
    }

    /// Attempt to parse `pat1, pat2 when guard ->`; returns args on success.
    fn try_parse_clause_head(&mut self, terms: &[Tok]) -> Option<Vec<Term>> {
        self.skip_newlines();
        if self.at_terminator(terms) {
            return None;
        }
        let mut pats = Vec::new();
        if *self.peek() != Tok::Op("->") {
            loop {
                let pat = self.parse_expr(P_ARROW_L, false).ok()?;
                pats.push(pat);
                if *self.peek() == Tok::Op(",") {
                    self.bump();
                    self.skip_newlines();
                } else {
                    break;
                }
            }
            if let Tok::Kw("when") = self.peek() {
                let span = self.span();
                self.bump();
                self.skip_newlines();
                let guard = self.parse_expr(P_OR, false).ok()?;
                let last = pats.pop()?;
                pats.push(Term::call("when", vec![last, guard], span));
            }
        }
        if *self.peek() == Tok::Op("->") {
            self.bump();
            Some(pats)
        } else {
            None
        }
    }

    fn parse_clauses(&mut self, terms: &[Tok]) -> Result<Vec<Term>> {
        let mut clauses = Vec::new();
        loop {
            self.skip_newlines();
            if self.at_terminator(terms) || *self.peek() == Tok::Eof {
                break;
            }
            let span = self.span();
            let head = self
                .try_parse_clause_head(terms)
                .ok_or_else(|| self.err("expected a '->' clause"))?;
            self.skip_newlines();
            // body: statements until the next clause head or terminator
            let mut stmts = Vec::new();
            loop {
                self.skip_newlines();
                if self.at_terminator(terms) || *self.peek() == Tok::Eof {
                    break;
                }
                if self.looks_like_clause_head(terms) {
                    break;
                }
                stmts.push(self.parse_stmt()?);
                match self.peek() {
                    Tok::Newline => {
                        self.bump();
                    }
                    t if terms.contains(t) || *t == Tok::Eof => break,
                    _ => {
                        return Err(
                            self.err(format!("expected end of statement, found {:?}", self.peek()))
                        )
                    }
                }
            }
            if stmts.is_empty() {
                return Err(self.err_at(span, "clause has an empty body"));
            }
            let body = Term::block(stmts, span);
            clauses.push(Term::call("->", vec![Term::List(head), body], span));
        }
        if clauses.is_empty() {
            return Err(self.err("expected at least one '->' clause"));
        }
        Ok(clauses)
    }
}

/// `lhs |> f(a)` becomes `f(lhs, a)` (GEP-0001-R006, Elixir first-arg pipe).
fn pipe_into(lhs: Term, rhs: Term, span: Span, file: &str) -> Result<Term> {
    match rhs {
        Term::Call(mut c) => {
            match &c.callee {
                Callee::Name(_) | Callee::Dot { .. } | Callee::Apply(_) => {
                    c.args.insert(0, lhs);
                    Ok(Term::Call(c))
                }
            }
        }
        Term::Var(name, _) => Ok(Term::Call(Box::new(Call {
            callee: Callee::Name(name),
            args: vec![lhs],
            span,
        }))),
        other => Err(Diagnostic::new(
            file,
            span,
            format!("the right side of |> must be a call, found {other:?}"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Callee, Term};

    fn parse(src: &str) -> Term {
        parse_expr_str("<test>", src).unwrap()
    }

    #[test]
    fn parses_arithmetic_precedence() {
        let t = parse("1 + 2 * 3");
        match t {
            Term::Call(c) => {
                assert!(matches!(&c.callee, Callee::Name(n) if n == "+"));
                assert_eq!(c.args[0], Term::Int(1));
                assert!(c.args[1].is_call_named("*"));
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn parses_pipe_as_first_arg() {
        let t = parse("xs |> map(f)");
        match t {
            Term::Call(c) => {
                assert!(matches!(&c.callee, Callee::Name(n) if n == "map"));
                assert_eq!(c.args.len(), 2);
                assert_eq!(c.args[0], Term::Var("xs".into(), None));
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn parses_defmodule_shape() {
        let t = parse("defmodule App.Hello do\n  def hi(x) do\n    x\n  end\nend");
        match &t {
            Term::Call(c) => {
                assert!(matches!(&c.callee, Callee::Name(n) if n == "defmodule"));
                assert_eq!(c.args[0], Term::Alias(vec!["App".into(), "Hello".into()]));
                assert!(Term::keyword_arg(&c.args, "do").is_some());
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn parses_case_clauses() {
        let t = parse("case x do\n  {:ok, v} -> v\n  :error -> nil\nend");
        match &t {
            Term::Call(c) => {
                let block = Term::keyword_arg(&c.args, "do").unwrap();
                assert!(block.is_call_named("__clauses__"));
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn parses_keyword_shorthand_def() {
        let t = parse("def inc(x), do: x + 1");
        match &t {
            Term::Call(c) => {
                assert!(matches!(&c.callee, Callee::Name(n) if n == "def"));
                assert!(Term::keyword_arg(&c.args, "do").is_some());
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn parses_remote_reference_call() {
        let t = parse("$math.sqrt(2.0)");
        match &t {
            Term::Call(c) => match &c.callee {
                Callee::Dot { base, name, is_call } => {
                    assert_eq!(**base, Term::PyRef("math".into(), false));
                    assert_eq!(name, "sqrt");
                    assert!(is_call);
                }
                other => panic!("{other:?}"),
            },
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn bare_pyref_and_quoted_pyref() {
        assert_eq!(parse("$math"), Term::PyRef("math".into(), false));
        let t = parse("$(os.path).join(a, b)");
        match &t {
            Term::Call(c) => match &c.callee {
                Callee::Dot { base, .. } => {
                    assert_eq!(**base, Term::PyRef("os.path".into(), true));
                }
                other => panic!("{other:?}"),
            },
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn block_forms_are_expressions_everywhere() {
        // `if`/`case`/... are ordinary expressions, as in Elixir: inside
        // tuples, lists, call arguments, and map values
        let t = parse("{1, if ok do 2 else 3 end}");
        match &t {
            Term::Tuple(items) => {
                assert!(matches!(&items[1], Term::Call(c)
                    if matches!(&c.callee, Callee::Name(n) if n == "if")));
            }
            other => panic!("{other:?}"),
        }
        let t = parse("f(case x do\n 1 -> :a\n end)");
        match &t {
            Term::Call(c) => {
                assert!(matches!(&c.args[0], Term::Call(inner)
                    if matches!(&inner.callee, Callee::Name(n) if n == "case")));
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn parses_method_chain() {
        let t = parse("df.rolling(5).mean()");
        assert!(matches!(&t, Term::Call(c)
            if matches!(&c.callee, Callee::Dot { name, .. } if name == "mean")));
    }

    #[test]
    fn parses_map_and_keyword_list() {
        let t = parse("%{\"k\" => 1, a: 2}");
        match &t {
            Term::Map(entries) => {
                assert_eq!(entries.len(), 2);
                assert_eq!(entries[1].0, Term::Atom("a".into()));
            }
            other => panic!("{other:?}"),
        }
        let t = parse("[a: 1, b: 2]");
        match &t {
            Term::List(items) => assert!(matches!(&items[0], Term::Pair(k, _) if k == "a")),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn parses_fn_and_capture() {
        let t = parse("fn x -> x + 1 end");
        assert!(t.is_call_named("fn"));
        let t = parse("&(&1 + 1)");
        assert!(t.is_call_named("&"));
    }

    #[test]
    fn parses_cons_pattern() {
        let t = parse("[h | t]");
        match &t {
            Term::List(items) => assert!(items[0].is_call_named("|")),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn parses_string_interpolation() {
        let t = parse("\"hi #{name}!\"");
        match &t {
            Term::Str(parts) => assert_eq!(parts.len(), 3),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn parses_pin_and_match() {
        let t = parse("^x = y");
        assert!(t.is_call_named("="));
    }

    #[test]
    fn parses_quote_block() {
        let t = parse("quote do\n  a + b\nend");
        match &t {
            Term::Call(c) => {
                assert!(matches!(&c.callee, Callee::Name(n) if n == "quote"));
                assert!(Term::keyword_arg(&c.args, "do").is_some());
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn parses_attributes() {
        let t = parse("@doc \"adds one\"");
        assert!(t.is_call_named("@doc"));
        let t = parse("@decorate $functools.cache");
        assert!(t.is_call_named("@decorate"));
    }

    #[test]
    fn parses_paren_call_with_do_block() {
        let t = parse("if ok?(x) do\n  1\nelse\n  2\nend");
        match &t {
            Term::Call(c) => {
                assert!(matches!(&c.callee, Callee::Name(n) if n == "if"));
                assert!(Term::keyword_arg(&c.args, "do").is_some());
                assert!(Term::keyword_arg(&c.args, "else").is_some());
            }
            other => panic!("{other:?}"),
        }
    }
}

#[cfg(test)]
mod pipe_method_tests {
    use super::*;
    use crate::ast::{Callee, Term};

    #[test]
    fn pipes_into_method_calls() {
        let t = parse_expr_str("<test>", "df |> .groupby(\"p\") |> .agg(spec)").unwrap();
        // outermost is .agg on (.groupby on df)
        match &t {
            Term::Call(c) => match &c.callee {
                Callee::Dot { base, name, is_call } => {
                    assert_eq!(name, "agg");
                    assert!(is_call);
                    assert!(matches!(&**base, Term::Call(inner)
                        if matches!(&inner.callee, Callee::Dot { name, .. } if name == "groupby")));
                }
                other => panic!("{other:?}"),
            },
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn pipes_into_methods_across_lines() {
        let t = parse_expr_str("<test>", "df\n|> .head(3)\n|> .describe()").unwrap();
        assert!(matches!(&t, Term::Call(c)
            if matches!(&c.callee, Callee::Dot { name, .. } if name == "describe")));
    }
}
