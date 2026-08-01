//! Compile-time macro expansion (GEP-0002).
//!
//! Macros are `defmacro` definitions whose bodies run on a small,
//! deterministic interpreter over `Term`s: values *are* terms, `quote`
//! builds terms with hygiene contexts, and `unquote`/`unquote_splicing`
//! inject argument terms. Expansion repeats until no macro calls remain,
//! with a bounded depth (GEP-0002-R001/R003/R004).

use std::collections::HashMap;

use crate::ast::{Call, Callee, StrPart, Term};
use crate::diag::{Diagnostic, Result, Span};

const MAX_DEPTH: u32 = 128;
const MAX_STEPS: u64 = 1_000_000;

/// Forms the expander must never treat as macro calls.
const SPECIAL_FORMS: &[&str] = &[
    "__block__", "__clauses__", "defmodule", "def", "defp", "defmacro", "alias", "import",
    "require", "pyimport", "quote", "unquote", "unquote_splicing", "var!", "if", "unless",
    "case", "cond", "with", "fn", "->", "when", "=", "|", "|>", "&", "^", "not", "and", "or",
    "+", "-", "*", "/", "//", "==", "!=", "<", ">", "<=", ">=", "++", "<>", "..", "<-",
];

#[derive(Debug, Clone)]
pub struct MacroDef {
    pub params: Vec<String>,
    pub body: Term,
    pub span: Span,
}

pub type MacroTable = HashMap<(String, usize), MacroDef>;

/// Collect `defmacro` definitions from a module body (GEP-0002-R001).
pub fn collect_macros(file: &str, module_body: &Term) -> Result<MacroTable> {
    let mut table = MacroTable::new();
    for stmt in module_body.as_block() {
        let Term::Call(call) = &stmt else { continue };
        if !matches!(&call.callee, Callee::Name(n) if n == "defmacro") {
            continue;
        }
        let head = call.args.first().ok_or_else(|| {
            Diagnostic::new(file, call.span, "defmacro requires a name and parameters")
        })?;
        let (name, params) = match head {
            Term::Call(h) => match &h.callee {
                Callee::Name(n) => {
                    let mut params = Vec::new();
                    for p in &h.args {
                        match p {
                            Term::Var(v, _) => params.push(v.clone()),
                            other => {
                                return Err(Diagnostic::new(
                                    file,
                                    h.span,
                                    format!(
                                        "defmacro parameters must be plain names, found {other:?}"
                                    ),
                                ))
                            }
                        }
                    }
                    (n.clone(), params)
                }
                _ => {
                    return Err(Diagnostic::new(
                        file,
                        h.span,
                        "defmacro head must be a plain call like `name(a, b)`",
                    ))
                }
            },
            other => {
                return Err(Diagnostic::new(
                    file,
                    call.span,
                    format!("defmacro head must be a call, found {other:?}"),
                ))
            }
        };
        let body = Term::keyword_arg(&call.args, "do")
            .ok_or_else(|| Diagnostic::new(file, call.span, "defmacro requires a do block"))?
            .clone();
        let key = (name.clone(), params.len());
        if table.contains_key(&key) {
            return Err(Diagnostic::new(
                file,
                call.span,
                format!("duplicate defmacro {name}/{} in module", params.len()),
            ));
        }
        table.insert(
            key,
            MacroDef {
                params,
                body,
                span: call.span,
            },
        );
    }
    Ok(table)
}

pub struct Expander {
    file: String,
    macros: MacroTable,
    next_ctx: u64,
    steps: u64,
}

impl Expander {
    pub fn new(file: &str, macros: MacroTable) -> Self {
        Expander {
            file: file.to_string(),
            macros,
            next_ctx: 1,
            steps: 0,
        }
    }

    /// Expand every macro call in a module body.
    pub fn expand_module(&mut self, body: &Term) -> Result<Term> {
        self.expand_term(body, 0)
    }

    fn err(&self, span: Span, msg: impl Into<String>) -> Diagnostic {
        Diagnostic::new(&self.file, span, msg)
    }

    fn expand_term(&mut self, term: &Term, depth: u32) -> Result<Term> {
        if depth > MAX_DEPTH {
            return Err(self.err(
                term.span(),
                format!("macro expansion exceeded the depth limit of {MAX_DEPTH}"),
            ));
        }
        match term {
            Term::Call(call) => {
                if let Callee::Name(name) = &call.callee {
                    // never expand inside quote: it is data until injected
                    if name == "quote" {
                        return Ok(term.clone());
                    }
                    if name == "defmacro" {
                        // macro definitions are already collected; keep as-is
                        return Ok(term.clone());
                    }
                    let key = (name.clone(), call.args.len());
                    let arity_key = self.macros.contains_key(&key);
                    if arity_key && !SPECIAL_FORMS.contains(&name.as_str()) {
                        let def = self.macros[&key].clone();
                        let expanded = self.invoke_macro(&def, name, call)?;
                        return self.expand_term(&expanded, depth + 1);
                    }
                }
                let mut new_call = call.as_ref().clone();
                if let Callee::Dot { base, .. } = &mut new_call.callee {
                    **base = self.expand_term(base, depth)?;
                }
                if let Callee::Apply(f) = &mut new_call.callee {
                    **f = self.expand_term(f, depth)?;
                }
                for arg in &mut new_call.args {
                    *arg = self.expand_term(arg, depth)?;
                }
                Ok(Term::Call(Box::new(new_call)))
            }
            Term::List(items) => Ok(Term::List(self.expand_all(items, depth)?)),
            Term::Tuple(items) => Ok(Term::Tuple(self.expand_all(items, depth)?)),
            Term::Map(entries) => {
                let mut out = Vec::new();
                for (k, v) in entries {
                    out.push((self.expand_term(k, depth)?, self.expand_term(v, depth)?));
                }
                Ok(Term::Map(out))
            }
            Term::Pair(k, v) => Ok(Term::Pair(
                k.clone(),
                Box::new(self.expand_term(v, depth)?),
            )),
            Term::Str(parts) => {
                let mut out = Vec::new();
                for p in parts {
                    match p {
                        StrPart::Text(t) => out.push(StrPart::Text(t.clone())),
                        StrPart::Interp(e) => {
                            out.push(StrPart::Interp(Box::new(self.expand_term(e, depth)?)))
                        }
                    }
                }
                Ok(Term::Str(out))
            }
            other => Ok(other.clone()),
        }
    }

    fn expand_all(&mut self, items: &[Term], depth: u32) -> Result<Vec<Term>> {
        items.iter().map(|t| self.expand_term(t, depth)).collect()
    }

    fn invoke_macro(&mut self, def: &MacroDef, name: &str, call: &Call) -> Result<Term> {
        let ctx = self.next_ctx;
        self.next_ctx += 1;
        let mut env = Env::new();
        for (param, arg) in def.params.iter().zip(&call.args) {
            env.bind(param, arg.clone());
        }
        let result = self.eval(&def.body, &mut env, ctx).map_err(|mut d| {
            d.origin.push((format!("macro {name}"), call.span));
            d
        })?;
        Ok(result)
    }

    // ---- the compile-time interpreter (GEP-0002-R003) -------------------

    fn eval(&mut self, term: &Term, env: &mut Env, ctx: u64) -> Result<Term> {
        self.steps += 1;
        if self.steps > MAX_STEPS {
            return Err(self.err(
                term.span(),
                format!("macro evaluation exceeded the step limit of {MAX_STEPS}"),
            ));
        }
        match term {
            Term::Int(_) | Term::Float(_) | Term::Bool(_) | Term::Nil | Term::Atom(_) => {
                Ok(term.clone())
            }
            Term::Alias(_) => Ok(term.clone()),
            Term::Str(parts) => {
                let mut text = String::new();
                for p in parts {
                    match p {
                        StrPart::Text(t) => text.push_str(t),
                        StrPart::Interp(e) => {
                            let v = self.eval(e, env, ctx)?;
                            text.push_str(&term_to_display(&v));
                        }
                    }
                }
                Ok(Term::Str(vec![StrPart::Text(text)]))
            }
            Term::Var(name, _) => env.lookup(name).ok_or_else(|| {
                self.err(
                    Span::default(),
                    format!("undefined variable '{name}' in macro body"),
                )
            }),
            Term::List(items) => {
                let vals = self.eval_all(items, env, ctx)?;
                Ok(Term::List(vals))
            }
            Term::Tuple(items) => {
                let vals = self.eval_all(items, env, ctx)?;
                Ok(Term::Tuple(vals))
            }
            Term::Map(entries) => {
                let mut out = Vec::new();
                for (k, v) in entries {
                    out.push((self.eval(k, env, ctx)?, self.eval(v, env, ctx)?));
                }
                Ok(Term::Map(out))
            }
            Term::Pair(k, v) => Ok(Term::Pair(k.clone(), Box::new(self.eval(v, env, ctx)?))),
            Term::Call(call) => self.eval_call(call, env, ctx),
        }
    }

    fn eval_all(&mut self, items: &[Term], env: &mut Env, ctx: u64) -> Result<Vec<Term>> {
        items.iter().map(|t| self.eval(t, env, ctx)).collect()
    }

    fn eval_call(&mut self, call: &Call, env: &mut Env, ctx: u64) -> Result<Term> {
        let span = call.span;
        let name = match &call.callee {
            Callee::Name(n) => n.as_str(),
            _ => {
                return Err(self.err(
                    span,
                    "remote and anonymous calls are not allowed in macro bodies (GEP-0002-R003)",
                ))
            }
        };
        match name {
            "__block__" => {
                let mut last = Term::Nil;
                for stmt in &call.args {
                    last = self.eval(stmt, env, ctx)?;
                }
                Ok(last)
            }
            "quote" => {
                let body = Term::keyword_arg(&call.args, "do")
                    .cloned()
                    .or_else(|| call.args.first().cloned())
                    .ok_or_else(|| self.err(span, "quote requires a body"))?;
                self.quasiquote(&body, env, ctx)
            }
            "unquote" | "unquote_splicing" => Err(self.err(
                span,
                format!("{name} is only meaningful inside quote"),
            )),
            "=" => {
                let value = self.eval(&call.args[1], env, ctx)?;
                if !self.match_pattern(&call.args[0], &value, env) {
                    return Err(self.err(span, "no match of right-hand side value in macro body"));
                }
                Ok(value)
            }
            "if" | "unless" => {
                let cond = self.eval(&call.args[0], env, ctx)?;
                let truthy = is_truthy(&cond) != (name == "unless");
                let branch = if truthy {
                    Term::keyword_arg(&call.args, "do")
                } else {
                    Term::keyword_arg(&call.args, "else")
                };
                match branch {
                    Some(b) => self.eval(&b.clone(), env, ctx),
                    None => Ok(Term::Nil),
                }
            }
            "case" => {
                let subject = self.eval(&call.args[0], env, ctx)?;
                let clauses = Term::keyword_arg(&call.args, "do")
                    .ok_or_else(|| self.err(span, "case requires a do block"))?
                    .clone();
                let Term::Call(cl) = &clauses else {
                    return Err(self.err(span, "case requires -> clauses"));
                };
                for clause in &cl.args {
                    let Term::Call(c) = clause else { continue };
                    let Term::List(pats) = &c.args[0] else { continue };
                    let (pat, guard) = split_guard(&pats[0]);
                    let mut trial = env.clone();
                    if self.match_pattern(pat, &subject, &mut trial) {
                        let guard_ok = match guard {
                            Some(g) => {
                                let v = self.eval(g, &mut trial, ctx)?;
                                is_truthy(&v)
                            }
                            None => true,
                        };
                        if guard_ok {
                            *env = trial;
                            return self.eval(&c.args[1].clone(), env, ctx);
                        }
                    }
                }
                Err(self.err(span, "no case clause matched in macro body"))
            }
            "cond" => {
                let clauses = Term::keyword_arg(&call.args, "do")
                    .ok_or_else(|| self.err(span, "cond requires a do block"))?
                    .clone();
                let Term::Call(cl) = &clauses else {
                    return Err(self.err(span, "cond requires -> clauses"));
                };
                for clause in &cl.args {
                    let Term::Call(c) = clause else { continue };
                    let Term::List(pats) = &c.args[0] else { continue };
                    let v = self.eval(&pats[0], env, ctx)?;
                    if is_truthy(&v) {
                        return self.eval(&c.args[1].clone(), env, ctx);
                    }
                }
                Err(self.err(span, "no cond clause was truthy in macro body"))
            }
            "raise" => {
                let msg = self
                    .eval(call.args.first().unwrap_or(&Term::Nil), env, ctx)
                    .map(|t| term_to_display(&t))
                    .unwrap_or_default();
                Err(self.err(span, format!("macro raised: {msg}")))
            }
            "length" => {
                let v = self.eval(&call.args[0], env, ctx)?;
                match v {
                    Term::List(items) => Ok(Term::Int(items.len() as i64)),
                    other => Err(self.err(span, format!("length expects a list, got {other:?}"))),
                }
            }
            "hd" => match self.eval(&call.args[0], env, ctx)? {
                Term::List(items) if !items.is_empty() => Ok(items[0].clone()),
                _ => Err(self.err(span, "hd expects a non-empty list")),
            },
            "tl" => match self.eval(&call.args[0], env, ctx)? {
                Term::List(items) if !items.is_empty() => Ok(Term::List(items[1..].to_vec())),
                _ => Err(self.err(span, "tl expects a non-empty list")),
            },
            "elem" => {
                let t = self.eval(&call.args[0], env, ctx)?;
                let i = self.eval(&call.args[1], env, ctx)?;
                match (t, i) {
                    (Term::Tuple(items), Term::Int(n)) if (n as usize) < items.len() => {
                        Ok(items[n as usize].clone())
                    }
                    _ => Err(self.err(span, "elem expects a tuple and an index in range")),
                }
            }
            "reverse" => match self.eval(&call.args[0], env, ctx)? {
                Term::List(mut items) => {
                    items.reverse();
                    Ok(Term::List(items))
                }
                _ => Err(self.err(span, "reverse expects a list")),
            },
            op @ ("+" | "-" | "*" | "/" | "//") if call.args.len() == 2 => {
                let a = self.eval(&call.args[0], env, ctx)?;
                let b = self.eval(&call.args[1], env, ctx)?;
                arith(op, &a, &b).ok_or_else(|| {
                    self.err(span, format!("invalid operands for {op} in macro body"))
                })
            }
            "-" if call.args.len() == 1 => {
                match self.eval(&call.args[0], env, ctx)? {
                    Term::Int(n) => Ok(Term::Int(-n)),
                    Term::Float(f) => Ok(Term::Float(-f)),
                    _ => Err(self.err(span, "unary '-' expects a number")),
                }
            }
            "++" => {
                let a = self.eval(&call.args[0], env, ctx)?;
                let b = self.eval(&call.args[1], env, ctx)?;
                match (a, b) {
                    (Term::List(mut x), Term::List(y)) => {
                        x.extend(y);
                        Ok(Term::List(x))
                    }
                    _ => Err(self.err(span, "++ expects two lists")),
                }
            }
            "<>" => {
                let a = self.eval(&call.args[0], env, ctx)?;
                let b = self.eval(&call.args[1], env, ctx)?;
                match (a.as_plain_str(), b.as_plain_str()) {
                    (Some(x), Some(y)) => Ok(Term::Str(vec![StrPart::Text(format!("{x}{y}"))])),
                    _ => Err(self.err(span, "<> expects two strings")),
                }
            }
            op @ ("==" | "!=" | "<" | ">" | "<=" | ">=") => {
                let a = self.eval(&call.args[0], env, ctx)?;
                let b = self.eval(&call.args[1], env, ctx)?;
                compare(op, &a, &b)
                    .map(Term::Bool)
                    .ok_or_else(|| self.err(span, format!("cannot compare with {op}")))
            }
            "and" => {
                let a = self.eval(&call.args[0], env, ctx)?;
                if is_truthy(&a) {
                    self.eval(&call.args[1], env, ctx)
                } else {
                    Ok(a)
                }
            }
            "or" => {
                let a = self.eval(&call.args[0], env, ctx)?;
                if is_truthy(&a) {
                    Ok(a)
                } else {
                    self.eval(&call.args[1], env, ctx)
                }
            }
            "not" => {
                let a = self.eval(&call.args[0], env, ctx)?;
                Ok(Term::Bool(!is_truthy(&a)))
            }
            other => {
                // calling another macro from a macro body: expand then eval
                let key = (other.to_string(), call.args.len());
                if self.macros.contains_key(&key) && !SPECIAL_FORMS.contains(&other) {
                    let def = self.macros[&key].clone();
                    let mut arg_env = Env::new();
                    let mut evaled_args = Vec::new();
                    for a in &call.args {
                        evaled_args.push(self.eval(a, env, ctx)?);
                    }
                    for (p, a) in def.params.iter().zip(&evaled_args) {
                        arg_env.bind(p, a.clone());
                    }
                    let inner_ctx = self.next_ctx;
                    self.next_ctx += 1;
                    return self.eval(&def.body.clone(), &mut arg_env, inner_ctx);
                }
                Err(self.err(
                    span,
                    format!("'{other}' is not available in macro bodies (GEP-0002-R003)"),
                ))
            }
        }
    }

    /// Build a term from a quote body: rename template variables into the
    /// hygiene context, inject unquotes, splice splicings (GEP-0002-R002/R004).
    fn quasiquote(&mut self, term: &Term, env: &mut Env, ctx: u64) -> Result<Term> {
        match term {
            Term::Var(name, None) => Ok(Term::Var(name.clone(), Some(ctx))),
            Term::Var(_, Some(_)) => Ok(term.clone()),
            Term::List(items) => Ok(Term::List(self.quasi_seq(items, env, ctx)?)),
            Term::Tuple(items) => Ok(Term::Tuple(self.quasi_seq(items, env, ctx)?)),
            Term::Map(entries) => {
                let mut out = Vec::new();
                for (k, v) in entries {
                    out.push((self.quasiquote(k, env, ctx)?, self.quasiquote(v, env, ctx)?));
                }
                Ok(Term::Map(out))
            }
            Term::Pair(k, v) => Ok(Term::Pair(
                k.clone(),
                Box::new(self.quasiquote(v, env, ctx)?),
            )),
            Term::Str(parts) => {
                let mut out = Vec::new();
                for p in parts {
                    match p {
                        StrPart::Text(t) => out.push(StrPart::Text(t.clone())),
                        StrPart::Interp(e) => {
                            out.push(StrPart::Interp(Box::new(self.quasiquote(e, env, ctx)?)))
                        }
                    }
                }
                Ok(Term::Str(out))
            }
            Term::Call(call) => {
                if let Callee::Name(n) = &call.callee {
                    match n.as_str() {
                        "unquote" => {
                            let arg = call.args.first().ok_or_else(|| {
                                self.err(call.span, "unquote requires an argument")
                            })?;
                            return self.eval(&arg.clone(), env, ctx);
                        }
                        "var!" => {
                            // hygiene escape: keep the call-site name
                            match call.args.first() {
                                Some(Term::Var(v, _)) => return Ok(Term::Var(v.clone(), None)),
                                _ => {
                                    return Err(self
                                        .err(call.span, "var! expects a variable name"))
                                }
                            }
                        }
                        "quote" => {
                            // nested quote stays as data
                            return Ok(term.clone());
                        }
                        _ => {}
                    }
                }
                let mut new_call = call.as_ref().clone();
                match &mut new_call.callee {
                    Callee::Dot { base, .. } => **base = self.quasiquote(base, env, ctx)?,
                    Callee::Apply(f) => **f = self.quasiquote(f, env, ctx)?,
                    Callee::Name(_) => {}
                }
                new_call.args = self.quasi_seq(&new_call.args, env, ctx)?;
                Ok(Term::Call(Box::new(new_call)))
            }
            other => Ok(other.clone()),
        }
    }

    fn quasi_seq(&mut self, items: &[Term], env: &mut Env, ctx: u64) -> Result<Vec<Term>> {
        let mut out = Vec::new();
        for item in items {
            if let Term::Call(c) = item {
                if matches!(&c.callee, Callee::Name(n) if n == "unquote_splicing") {
                    let arg = c.args.first().ok_or_else(|| {
                        self.err(c.span, "unquote_splicing requires an argument")
                    })?;
                    match self.eval(&arg.clone(), env, ctx)? {
                        Term::List(spliced) => out.extend(spliced),
                        other => {
                            return Err(self.err(
                                c.span,
                                format!("unquote_splicing expects a list, got {other:?}"),
                            ))
                        }
                    }
                    continue;
                }
            }
            out.push(self.quasiquote(item, env, ctx)?);
        }
        Ok(out)
    }

    fn match_pattern(&mut self, pat: &Term, value: &Term, env: &mut Env) -> bool {
        match (pat, value) {
            (Term::Var(name, _), _) if name == "_" => true,
            (Term::Var(name, _), v) => {
                env.bind(name, v.clone());
                true
            }
            (Term::Call(c), v) if matches!(&c.callee, Callee::Name(n) if n == "^") => {
                if let Some(Term::Var(name, _)) = c.args.first() {
                    match env.lookup(name) {
                        Some(bound) => bound == *v,
                        None => false,
                    }
                } else {
                    false
                }
            }
            (Term::List(pats), Term::List(vals)) => {
                // handle a trailing cons pattern [a, b | rest]
                if let Some(Term::Call(c)) = pats.last() {
                    if matches!(&c.callee, Callee::Name(n) if n == "|") {
                        let fixed = &pats[..pats.len() - 1];
                        if vals.len() < fixed.len() + 1 {
                            return false;
                        }
                        for (p, v) in fixed.iter().zip(vals) {
                            if !self.match_pattern(p, v, env) {
                                return false;
                            }
                        }
                        let head = &vals[fixed.len()];
                        let tail = Term::List(vals[fixed.len() + 1..].to_vec());
                        return self.match_pattern(&c.args[0], head, env)
                            && self.match_pattern(&c.args[1], &tail, env);
                    }
                }
                pats.len() == vals.len()
                    && pats
                        .iter()
                        .zip(vals)
                        .all(|(p, v)| self.match_pattern(p, v, env))
            }
            (Term::Tuple(pats), Term::Tuple(vals)) => {
                pats.len() == vals.len()
                    && pats
                        .iter()
                        .zip(vals)
                        .all(|(p, v)| self.match_pattern(p, v, env))
            }
            (Term::Map(pat_entries), Term::Map(val_entries)) => {
                pat_entries.iter().all(|(pk, pv)| {
                    val_entries
                        .iter()
                        .any(|(vk, vv)| pk == vk && self.match_pattern(pv, vv, env))
                })
            }
            (a, b) => a == b,
        }
    }
}

fn split_guard(pat: &Term) -> (&Term, Option<&Term>) {
    if let Term::Call(c) = pat {
        if matches!(&c.callee, Callee::Name(n) if n == "when") && c.args.len() == 2 {
            return (&c.args[0], Some(&c.args[1]));
        }
    }
    (pat, None)
}

fn is_truthy(t: &Term) -> bool {
    !matches!(t, Term::Bool(false) | Term::Nil)
}

fn arith(op: &str, a: &Term, b: &Term) -> Option<Term> {
    let (x, y) = match (a, b) {
        (Term::Int(x), Term::Int(y)) => {
            return Some(match op {
                "+" => Term::Int(x + y),
                "-" => Term::Int(x - y),
                "*" => Term::Int(x * y),
                "//" => Term::Int(x.checked_div(*y)?),
                "/" => Term::Float(*x as f64 / *y as f64),
                _ => return None,
            })
        }
        (Term::Float(x), Term::Float(y)) => (*x, *y),
        (Term::Int(x), Term::Float(y)) => (*x as f64, *y),
        (Term::Float(x), Term::Int(y)) => (*x, *y as f64),
        _ => return None,
    };
    Some(match op {
        "+" => Term::Float(x + y),
        "-" => Term::Float(x - y),
        "*" => Term::Float(x * y),
        "/" => Term::Float(x / y),
        _ => return None,
    })
}

fn compare(op: &str, a: &Term, b: &Term) -> Option<bool> {
    if op == "==" {
        return Some(a == b);
    }
    if op == "!=" {
        return Some(a != b);
    }
    let (x, y) = match (a, b) {
        (Term::Int(x), Term::Int(y)) => (*x as f64, *y as f64),
        (Term::Float(x), Term::Float(y)) => (*x, *y),
        (Term::Int(x), Term::Float(y)) => (*x as f64, *y),
        (Term::Float(x), Term::Int(y)) => (*x, *y as f64),
        _ => return None,
    };
    Some(match op {
        "<" => x < y,
        ">" => x > y,
        "<=" => x <= y,
        ">=" => x >= y,
        _ => return None,
    })
}

fn term_to_display(t: &Term) -> String {
    match t.as_plain_str() {
        Some(s) => s,
        None => crate::printer::print_module(t).trim_end().to_string(),
    }
}

#[derive(Debug, Clone)]
struct Env {
    vars: HashMap<String, Term>,
}

impl Env {
    fn new() -> Self {
        Env {
            vars: HashMap::new(),
        }
    }

    fn bind(&mut self, name: &str, value: Term) {
        self.vars.insert(name.to_string(), value);
    }

    fn lookup(&self, name: &str) -> Option<Term> {
        self.vars.get(name).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_file;
    use crate::printer::print_module;

    fn expand(src: &str) -> String {
        let module = parse_file("<test>", src).unwrap();
        let macros = collect_macros("<test>", &module).unwrap();
        let mut ex = Expander::new("<test>", macros);
        let out = ex.expand_module(&module).unwrap();
        print_module(&out)
    }

    #[test]
    fn expands_simple_macro() {
        let out = expand(
            "defmacro double(x) do\n  quote do\n    unquote(x) * 2\n  end\nend\ny = double(3)",
        );
        assert!(out.contains("y = 3 * 2"), "{out}");
    }

    #[test]
    fn expands_unless_macro() {
        let out = expand(
            "defmacro my_unless(cond, body) do\n  quote do\n    if not unquote(cond) do\n      unquote(body)\n    end\n  end\nend\nmy_unless(flag, run())",
        );
        assert!(out.contains("if not flag do"), "{out}");
        assert!(out.contains("run()"), "{out}");
    }

    #[test]
    fn hygiene_renames_template_vars() {
        let out = expand(
            "defmacro swapped(a) do\n  quote do\n    tmp = unquote(a)\n    tmp + 1\n  end\nend\nswapped(9)",
        );
        assert!(out.contains("tmp__gan"), "{out}");
        assert!(!out.contains("tmp =\n"), "{out}");
    }

    #[test]
    fn var_bang_escapes_hygiene() {
        let out = expand(
            "defmacro set_it() do\n  quote do\n    var!(result) = 42\n  end\nend\nset_it()",
        );
        assert!(out.contains("result = 42"), "{out}");
        assert!(!out.contains("result__gan"), "{out}");
    }

    #[test]
    fn splices_lists() {
        let out = expand(
            "defmacro wrap(items) do\n  quote do\n    [0, unquote_splicing(items), 99]\n  end\nend\nwrap([1, 2])",
        );
        assert!(out.contains("[0, 1, 2, 99]"), "{out}");
    }

    #[test]
    fn macro_can_compute() {
        let out = expand(
            "defmacro repeat(x, n) do\n  if n == 0 do\n    quote do\n      nil\n    end\n  else\n    quote do\n      unquote(x)\n      unquote(repeat(x, n - 1))\n    end\n  end\nend\nrepeat(ping(), 2)",
        );
        assert!(out.matches("ping()").count() >= 2, "{out}");
    }

    #[test]
    fn depth_limit_reported() {
        let module = parse_file(
            "<test>",
            "defmacro forever() do\n  quote do\n    forever()\n  end\nend\nforever()",
        )
        .unwrap();
        let macros = collect_macros("<test>", &module).unwrap();
        let mut ex = Expander::new("<test>", macros);
        let err = ex.expand_module(&module).unwrap_err();
        assert!(err.message.contains("depth limit"), "{}", err.message);
    }
}
