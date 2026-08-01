//! Python code generation (GEP-0001-R009..R017, GEP-0003).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use crate::ast::{Call, Callee, StrPart, Term};
use crate::diag::{Diagnostic, Result, Span};

/// Map a Gandora identifier to a Python identifier (GEP-0001-R015).
pub fn map_ident(name: &str) -> String {
    let mut out = String::new();
    for c in name.chars() {
        match c {
            '?' => out.push_str("_p"),
            '!' => out.push_str("_bang"),
            c if c.is_alphanumeric() || c == '_' => out.push(c),
            c => {
                let _ = write!(out, "_u{:x}_", c as u32);
            }
        }
    }
    if out.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        out.insert(0, '_');
    }
    const PY_KEYWORDS: &[&str] = &[
        "False", "None", "True", "and", "as", "assert", "async", "await", "break", "class",
        "continue", "def", "del", "elif", "else", "except", "finally", "for", "from", "global",
        "if", "import", "in", "is", "lambda", "nonlocal", "not", "or", "pass", "raise", "return",
        "try", "while", "with", "yield", "match",
    ];
    if PY_KEYWORDS.contains(&out.as_str()) {
        out.push_str("__kw");
    }
    out
}

/// `App.HelloWeb` -> `app.hello_web` (GEP-0001-R013/R014).
pub fn module_py_path(segs: &[String]) -> String {
    segs.iter()
        .map(|s| camel_to_snake(s))
        .collect::<Vec<_>>()
        .join(".")
}

pub fn camel_to_snake(s: &str) -> String {
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                out.push('_');
            }
            for lc in c.to_lowercase() {
                out.push(lc);
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[derive(Clone, Copy, PartialEq)]
enum Dest {
    Return,
    Assign(usize), // index into temp names
    Ignore,
}

struct FnDef {
    name: String,
    private: bool,
    doc: Option<String>,
    decorators: Vec<Term>,
    clauses: Vec<(Vec<Term>, Option<Term>, Term)>, // params, guard, body
    span: Span,
}

pub struct Codegen {
    file: String,
    module_segs: Vec<String>,
    py_imports: BTreeSet<String>,
    gan_imports: BTreeSet<String>,
    star_imports: BTreeSet<String>,
    aliases: BTreeMap<String, Vec<String>>,
    helpers: BTreeSet<&'static str>,
    local_funs: BTreeSet<(String, usize)>,
    private_funs: BTreeSet<(String, usize)>,
    tmp_counter: usize,
    tmp_names: Vec<String>,
    fn_counter: usize,
}

impl Codegen {
    pub fn new(file: &str, module_segs: Vec<String>) -> Self {
        Codegen {
            file: file.to_string(),
            module_segs,
            py_imports: BTreeSet::new(),
            gan_imports: BTreeSet::new(),
            star_imports: BTreeSet::new(),
            aliases: BTreeMap::new(),
            helpers: BTreeSet::new(),
            local_funs: BTreeSet::new(),
            private_funs: BTreeSet::new(),
            tmp_counter: 0,
            tmp_names: Vec::new(),
            fn_counter: 0,
        }
    }

    fn err(&self, span: Span, msg: impl Into<String>) -> Diagnostic {
        Diagnostic::new(&self.file, span, msg)
    }

    fn fresh_tmp(&mut self, prefix: &str) -> usize {
        let name = format!("_gan_{prefix}{}", self.tmp_counter);
        self.tmp_counter += 1;
        self.tmp_names.push(name);
        self.tmp_names.len() - 1
    }

    fn tmp(&self, idx: usize) -> &str {
        &self.tmp_names[idx]
    }

    /// Compile a fully expanded module to Python source.
    pub fn compile(&mut self, module: &Term) -> Result<String> {
        let stmts = module.as_block();
        let defmodule = stmts
            .iter()
            .find(|s| s.is_call_named("defmodule"))
            .ok_or_else(|| {
                self.err(Span::default(), "a source file must contain one defmodule")
            })?;
        if stmts.iter().filter(|s| s.is_call_named("defmodule")).count() > 1 {
            return Err(self.err(
                defmodule.span(),
                "a source file must contain exactly one defmodule (GEP-0001-R013)",
            ));
        }
        let Term::Call(dm) = defmodule else { unreachable!() };
        let declared = match dm.args.first() {
            Some(Term::Alias(segs)) => segs.clone(),
            _ => return Err(self.err(dm.span, "defmodule requires a module name")),
        };
        if !self.module_segs.is_empty() && declared != self.module_segs {
            return Err(self.err(
                dm.span,
                format!(
                    "module name {} does not match its file path (expected {}) (GEP-0001-R013)",
                    declared.join("."),
                    self.module_segs.join(".")
                ),
            ));
        }
        let body = Term::keyword_arg(&dm.args, "do")
            .ok_or_else(|| self.err(dm.span, "defmodule requires a do block"))?
            .clone();

        let mut moduledoc: Option<String> = None;
        let mut pending_doc: Option<String> = None;
        let mut pending_decorators: Vec<Term> = Vec::new();
        let mut funs: Vec<FnDef> = Vec::new();
        let mut order: BTreeMap<String, usize> = BTreeMap::new();

        for stmt in body.as_block() {
            let Term::Call(call) = &stmt else {
                return Err(self.err(
                    stmt.span(),
                    "module-level expressions are not supported; use def/defp",
                ));
            };
            let name = match &call.callee {
                Callee::Name(n) => n.clone(),
                _ => {
                    return Err(self.err(
                        call.span,
                        "module-level expressions are not supported; use def/defp",
                    ))
                }
            };
            match name.as_str() {
                "@moduledoc" => {
                    moduledoc = call.args.first().and_then(|t| t.as_plain_str());
                }
                "@doc" => {
                    pending_doc = call.args.first().and_then(|t| t.as_plain_str());
                }
                "@decorate" => {
                    if let Some(e) = call.args.first() {
                        pending_decorators.push(e.clone());
                    }
                }
                "pyimport" => self.collect_pyimport(call)?,
                "alias" => {
                    let segs = match call.args.first() {
                        Some(Term::Alias(segs)) => segs.clone(),
                        _ => return Err(self.err(call.span, "alias requires a module name")),
                    };
                    let short = match Term::keyword_arg(&call.args, "as") {
                        Some(Term::Alias(s)) if s.len() == 1 => s[0].clone(),
                        None => segs.last().unwrap().clone(),
                        _ => return Err(self.err(call.span, "as: must name a single alias")),
                    };
                    self.gan_imports.insert(module_py_path(&segs));
                    self.aliases.insert(short, segs);
                }
                "import" => {
                    let segs = match call.args.first() {
                        Some(Term::Alias(segs)) => segs.clone(),
                        _ => return Err(self.err(call.span, "import requires a module name")),
                    };
                    self.star_imports.insert(module_py_path(&segs));
                }
                "require" => { /* compile-time only */ }
                "defmacro" => { /* compile-time only */ }
                "def" | "defp" => {
                    let (fname, params, guard, fbody) = self.parse_def(call)?;
                    let key = format!("{fname}/{}", params.len());
                    self.local_funs.insert((fname.clone(), params.len()));
                    if name == "defp" {
                        self.private_funs.insert((fname.clone(), params.len()));
                    }
                    let idx = if let Some(&i) = order.get(&key) {
                        if pending_doc.is_some() || !pending_decorators.is_empty() {
                            return Err(self.err(
                                call.span,
                                "@doc/@decorate must precede the first clause of a function",
                            ));
                        }
                        i
                    } else {
                        funs.push(FnDef {
                            name: fname.clone(),
                            private: name == "defp",
                            doc: pending_doc.take(),
                            decorators: std::mem::take(&mut pending_decorators),
                            clauses: Vec::new(),
                            span: call.span,
                        });
                        order.insert(key, funs.len() - 1);
                        funs.len() - 1
                    };
                    if funs[idx].private != (name == "defp") {
                        return Err(self.err(
                            call.span,
                            format!("clauses of {fname} mix def and defp"),
                        ));
                    }
                    funs[idx].clauses.push((params, guard, fbody));
                }
                other => {
                    return Err(self.err(
                        call.span,
                        format!(
                            "'{other}' is not supported at module level \
                             (supported: def, defp, defmacro, alias, import, require, \
                             pyimport, @doc, @moduledoc, @decorate)"
                        ),
                    ))
                }
            }
        }

        // compile function bodies first so imports/helpers are collected
        let mut fun_code = String::new();
        let mut has_main0 = false;
        for f in &funs {
            if f.name == "main" && f.clauses.iter().any(|(p, _, _)| p.is_empty()) && !f.private {
                has_main0 = true;
            }
            let code = self.compile_fun(f)?;
            fun_code.push('\n');
            fun_code.push_str(&code);
        }

        let mut out = String::new();
        if let Some(doc) = &moduledoc {
            let _ = writeln!(out, "\"\"\"{}\"\"\"", doc.replace("\"\"\"", "\\\"\\\"\\\""));
        }
        let mut import_lines: Vec<String> = Vec::new();
        for m in &self.py_imports {
            import_lines.push(format!("import {m}"));
        }
        for m in &self.gan_imports {
            import_lines.push(format!("import {m}"));
        }
        for m in &self.star_imports {
            import_lines.push(format!("from {m} import *"));
        }
        if !import_lines.is_empty() {
            if !out.is_empty() {
                out.push('\n');
            }
            for l in import_lines {
                out.push_str(&l);
                out.push('\n');
            }
        }
        let helper_code = self.helper_code();
        if !helper_code.is_empty() {
            out.push('\n');
            out.push_str(&helper_code);
        }
        out.push_str(&fun_code);
        if has_main0 {
            out.push_str("\n\nif __name__ == \"__main__\":\n    main()\n");
        }
        if !out.ends_with('\n') {
            out.push('\n');
        }
        Ok(out)
    }

    fn collect_pyimport(&mut self, call: &Call) -> Result<()> {
        let module = match call.args.first() {
            Some(t) => dotted_name(t).ok_or_else(|| {
                self.err(call.span, "pyimport requires a module name like os.path")
            })?,
            None => return Err(self.err(call.span, "pyimport requires a module name")),
        };
        match Term::keyword_arg(&call.args, "as") {
            Some(Term::Var(alias, _)) => {
                self.py_imports
                    .insert(format!("{module} as {}", map_ident(alias)));
            }
            None => {
                self.py_imports.insert(module);
            }
            _ => return Err(self.err(call.span, "as: must name a plain identifier")),
        }
        Ok(())
    }

    fn parse_def(&mut self, call: &Call) -> Result<(String, Vec<Term>, Option<Term>, Term)> {
        let head = call
            .args
            .first()
            .ok_or_else(|| self.err(call.span, "def requires a function head"))?;
        let (head, guard) = match head {
            Term::Call(c) if matches!(&c.callee, Callee::Name(n) if n == "when") => {
                (&c.args[0], Some(c.args[1].clone()))
            }
            other => (other, None),
        };
        let (name, params) = match head {
            Term::Call(c) => match &c.callee {
                Callee::Name(n) => (n.clone(), c.args.clone()),
                _ => return Err(self.err(c.span, "def head must be a plain function call")),
            },
            Term::Var(n, _) => (n.clone(), Vec::new()),
            other => {
                return Err(self.err(
                    call.span,
                    format!("def head must be a function call, found {other:?}"),
                ))
            }
        };
        let body = Term::keyword_arg(&call.args, "do")
            .ok_or_else(|| self.err(call.span, "def requires a do block or do: expression"))?
            .clone();
        Ok((name, params, guard, body))
    }

    fn compile_fun(&mut self, f: &FnDef) -> Result<String> {
        let arity = f.clauses[0].0.len();
        for (params, _, _) in &f.clauses {
            if params.len() != arity {
                return Err(self.err(
                    f.span,
                    format!("clauses of {} have different arities", f.name),
                ));
            }
        }
        let mut py_name = map_ident(&f.name);
        if f.private {
            py_name.insert(0, '_');
        }
        let mut lines: Vec<String> = Vec::new();
        for dec in &f.decorators {
            let mut pre = Vec::new();
            let e = self.emit_expr(dec, &mut pre)?;
            if !pre.is_empty() {
                return Err(self.err(
                    dec.span(),
                    "@decorate expressions must be simple expressions",
                ));
            }
            lines.push(format!("@{e}"));
        }
        let simple = f.clauses.len() == 1
            && f.clauses[0].1.is_none()
            && f.clauses[0]
                .0
                .iter()
                .all(|p| matches!(p, Term::Var(n, _) if n != "_"));
        if simple {
            let (params, _, body) = &f.clauses[0];
            let names: Vec<String> = params
                .iter()
                .map(|p| match p {
                    Term::Var(n, ctx) => hygienic_name(n, *ctx),
                    _ => unreachable!(),
                })
                .collect();
            lines.push(format!("def {py_name}({}):", names.join(", ")));
            let mut body_lines = Vec::new();
            if let Some(doc) = &f.doc {
                body_lines.push(format!(
                    "\"\"\"{}\"\"\"",
                    doc.replace("\"\"\"", "\\\"\\\"\\\"")
                ));
            }
            self.emit_stmt_block(body, Dest::Return, &mut body_lines)?;
            push_indented(&mut lines, &body_lines);
        } else {
            lines.push(format!("def {py_name}(*_gan_args):"));
            let mut body_lines = Vec::new();
            if let Some(doc) = &f.doc {
                body_lines.push(format!(
                    "\"\"\"{}\"\"\"",
                    doc.replace("\"\"\"", "\\\"\\\"\\\"")
                ));
            }
            body_lines.push("match _gan_args:".to_string());
            for (params, guard, body) in &f.clauses {
                let mut pat_guards = Vec::new();
                let pats: Vec<String> = params
                    .iter()
                    .map(|p| self.compile_pattern(p, &mut pat_guards))
                    .collect::<Result<_>>()?;
                if let Some(g) = guard {
                    let mut pre = Vec::new();
                    let ge = self.emit_bool_expr(g, &mut pre)?;
                    if !pre.is_empty() {
                        return Err(
                            self.err(g.span(), "guards must be simple boolean expressions")
                        );
                    }
                    pat_guards.push(ge);
                }
                let mut case_line = format!("case ({},):", pats.join(", "));
                if pats.is_empty() {
                    case_line = "case ():".to_string();
                }
                if !pat_guards.is_empty() {
                    case_line = format!(
                        "case ({},) if {}:",
                        pats.join(", "),
                        pat_guards.join(" and ")
                    );
                    if pats.is_empty() {
                        case_line = format!("case () if {}:", pat_guards.join(" and "));
                    }
                }
                let mut clause_lines = vec![case_line];
                let mut inner = Vec::new();
                self.emit_stmt_block(body, Dest::Return, &mut inner)?;
                push_indented(&mut clause_lines, &inner);
                push_indented(&mut body_lines, &clause_lines);
            }
            self.helpers.insert("match_error");
            let n = &f.name;
            body_lines.push(format!(
                "raise GanMatchError(\"no clause of {n}/{arity} matched \" + repr(_gan_args))"
            ));
            push_indented(&mut lines, &body_lines);
        }
        Ok(format!("\n{}\n", lines.join("\n")))
    }

    // ---- statements ------------------------------------------------------

    fn emit_stmt_block(&mut self, body: &Term, dest: Dest, out: &mut Vec<String>) -> Result<()> {
        let stmts = body.as_block();
        if stmts.is_empty() {
            self.finish_value("None".into(), dest, out);
            return Ok(());
        }
        for (i, stmt) in stmts.iter().enumerate() {
            let d = if i + 1 == stmts.len() { dest } else { Dest::Ignore };
            self.emit_stmt(stmt, d, out)?;
        }
        Ok(())
    }

    fn finish_value(&mut self, expr: String, dest: Dest, out: &mut Vec<String>) {
        match dest {
            Dest::Return => out.push(format!("return {expr}")),
            Dest::Assign(t) => {
                let name = self.tmp(t).to_string();
                out.push(format!("{name} = {expr}"));
            }
            Dest::Ignore => {
                if expr
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '_' || c == '.')
                {
                    // skip pure reads with no effect
                } else {
                    out.push(expr);
                }
            }
        }
    }

    fn emit_stmt(&mut self, term: &Term, dest: Dest, out: &mut Vec<String>) -> Result<()> {
        match term {
            Term::Call(call) => {
                if let Callee::Name(name) = &call.callee {
                    match name.as_str() {
                        "__block__" => return self.emit_stmt_block(term, dest, out),
                        "=" => return self.emit_match(call, dest, out),
                        "if" | "unless" => return self.emit_if(call, name == "unless", dest, out),
                        "case" => return self.emit_case(call, dest, out),
                        "cond" => return self.emit_cond(call, dest, out),
                        "with" => return self.emit_with(call, dest, out),
                        "raise" => {
                            let mut pre = Vec::new();
                            let msg = match call.args.first() {
                                Some(e) => self.emit_expr(e, &mut pre)?,
                                None => "\"raise\"".to_string(),
                            };
                            out.extend(pre);
                            out.push(format!("raise RuntimeError({msg})"));
                            return Ok(());
                        }
                        _ => {}
                    }
                }
                let mut pre = Vec::new();
                let e = self.emit_expr(term, &mut pre)?;
                out.extend(pre);
                self.finish_value(e, dest, out);
                Ok(())
            }
            other => {
                let mut pre = Vec::new();
                let e = self.emit_expr(other, &mut pre)?;
                out.extend(pre);
                self.finish_value(e, dest, out);
                Ok(())
            }
        }
    }

    fn emit_match(&mut self, call: &Call, dest: Dest, out: &mut Vec<String>) -> Result<()> {
        let pat = &call.args[0];
        let value = &call.args[1];
        let mut pre = Vec::new();
        let ve = self.emit_expr(value, &mut pre)?;
        out.extend(pre);
        match pat {
            Term::Var(n, ctx) if n != "_" => {
                let name = hygienic_name(n, *ctx);
                out.push(format!("{name} = {ve}"));
                self.finish_value(name, dest, out);
            }
            Term::Var(_, _) => {
                // `_ = expr`
                let t = self.fresh_tmp("tmp");
                out.push(format!("{} = {ve}", self.tmp(t)));
                self.finish_value(self.tmp(t).to_string(), dest, out);
            }
            _ => {
                let t = self.fresh_tmp("val");
                let tname = self.tmp(t).to_string();
                out.push(format!("{tname} = {ve}"));
                let mut guards = Vec::new();
                let p = self.compile_pattern(pat, &mut guards)?;
                self.helpers.insert("match_error");
                out.push(format!("match {tname}:"));
                let case_line = if guards.is_empty() {
                    format!("case {p}:")
                } else {
                    format!("case {p} if {}:", guards.join(" and "))
                };
                push_indented(out, &[case_line, "    pass".into()]);
                push_indented(
                    out,
                    &[
                        "case _:".to_string(),
                        format!("    raise GanMatchError(\"no match of right-hand side value: \" + repr({tname}))"),
                    ],
                );
                self.finish_value(tname, dest, out);
            }
        }
        Ok(())
    }

    fn emit_if(
        &mut self,
        call: &Call,
        negate: bool,
        dest: Dest,
        out: &mut Vec<String>,
    ) -> Result<()> {
        let mut pre = Vec::new();
        let mut cond = self.emit_bool_expr(&call.args[0], &mut pre)?;
        out.extend(pre);
        if negate {
            cond = format!("not ({cond})");
        }
        out.push(format!("if {cond}:"));
        let mut then_lines = Vec::new();
        match Term::keyword_arg(&call.args, "do") {
            Some(b) => self.emit_stmt_block(&b.clone(), dest, &mut then_lines)?,
            None => self.finish_value("None".into(), dest, &mut then_lines),
        }
        if then_lines.is_empty() {
            then_lines.push("pass".into());
        }
        push_indented(out, &then_lines);
        let mut else_lines = Vec::new();
        match Term::keyword_arg(&call.args, "else") {
            Some(b) => self.emit_stmt_block(&b.clone(), dest, &mut else_lines)?,
            None => self.finish_value("None".into(), dest, &mut else_lines),
        }
        if !else_lines.is_empty() {
            out.push("else:".into());
            push_indented(out, &else_lines);
        }
        Ok(())
    }

    fn clause_list<'t>(&self, block: &'t Term, span: Span) -> Result<Vec<&'t Term>> {
        match block {
            Term::Call(c) if matches!(&c.callee, Callee::Name(n) if n == "__clauses__") => {
                Ok(c.args.iter().collect())
            }
            _ => Err(self.err(span, "expected '->' clauses in this do block")),
        }
    }

    fn emit_case(&mut self, call: &Call, dest: Dest, out: &mut Vec<String>) -> Result<()> {
        let mut pre = Vec::new();
        let subject = self.emit_expr(&call.args[0], &mut pre)?;
        out.extend(pre);
        let t = self.fresh_tmp("case");
        let tname = self.tmp(t).to_string();
        out.push(format!("{tname} = {subject}"));
        let block = Term::keyword_arg(&call.args, "do")
            .ok_or_else(|| self.err(call.span, "case requires a do block"))?
            .clone();
        let clauses = self.clause_list(&block, call.span)?;
        out.push(format!("match {tname}:"));
        let mut any_wild = false;
        let mut arms: Vec<String> = Vec::new();
        for clause in clauses {
            let Term::Call(c) = clause else { continue };
            let Term::List(pats) = &c.args[0] else { continue };
            if pats.len() != 1 {
                return Err(self.err(c.span, "case clauses take exactly one pattern"));
            }
            let (pat, guard) = split_when(&pats[0]);
            let mut guards = Vec::new();
            let p = self.compile_pattern(pat, &mut guards)?;
            if let Some(g) = guard {
                let mut gpre = Vec::new();
                let ge = self.emit_bool_expr(g, &mut gpre)?;
                if !gpre.is_empty() {
                    return Err(self.err(g.span(), "guards must be simple boolean expressions"));
                }
                guards.push(ge);
            }
            // a plain capture (or _) with no guard matches anything
            let is_capture = p == "_"
                || (!matches!(p.as_str(), "True" | "False" | "None")
                    && p.chars().next().is_some_and(|c| c.is_alphabetic() || c == '_')
                    && p.chars().all(|c| c.is_alphanumeric() || c == '_'));
            if is_capture && guards.is_empty() {
                any_wild = true;
            }
            let case_line = if guards.is_empty() {
                format!("case {p}:")
            } else {
                format!("case {p} if {}:", guards.join(" and "))
            };
            let mut clause_lines = vec![case_line];
            let mut inner = Vec::new();
            self.emit_stmt_block(&c.args[1], dest, &mut inner)?;
            if inner.is_empty() {
                inner.push("pass".into());
            }
            push_indented(&mut clause_lines, &inner);
            arms.extend(clause_lines);
        }
        if !any_wild {
            self.helpers.insert("match_error");
            arms.push("case _:".into());
            arms.push(format!(
                "    raise GanMatchError(\"no case clause matched: \" + repr({tname}))"
            ));
        }
        push_indented(out, &arms);
        Ok(())
    }

    fn emit_cond(&mut self, call: &Call, dest: Dest, out: &mut Vec<String>) -> Result<()> {
        let block = Term::keyword_arg(&call.args, "do")
            .ok_or_else(|| self.err(call.span, "cond requires a do block"))?
            .clone();
        let clauses = self.clause_list(&block, call.span)?;
        let mut first = true;
        let mut closed = false;
        for clause in clauses {
            let Term::Call(c) = clause else { continue };
            let Term::List(pats) = &c.args[0] else { continue };
            let mut pre = Vec::new();
            let cond_term = &pats[0];
            let is_true = matches!(cond_term, Term::Bool(true));
            let cond = self.emit_bool_expr(cond_term, &mut pre)?;
            if !pre.is_empty() && !first {
                return Err(self.err(
                    c.span,
                    "cond conditions after the first must be simple expressions",
                ));
            }
            out.extend(pre);
            let mut inner = Vec::new();
            self.emit_stmt_block(&c.args[1], dest, &mut inner)?;
            if inner.is_empty() {
                inner.push("pass".into());
            }
            if is_true {
                if first {
                    self.emit_stmt_block(&c.args[1], dest, out)?;
                } else {
                    out.push("else:".into());
                    push_indented(out, &inner);
                }
                closed = true;
                break;
            }
            out.push(format!("{}if {cond}:", if first { "" } else { "el" }));
            push_indented(out, &inner);
            first = false;
        }
        if !closed {
            self.helpers.insert("match_error");
            out.push("else:".into());
            let mut inner = vec!["raise GanMatchError(\"no cond clause was truthy\")".to_string()];
            if matches!(dest, Dest::Assign(_)) {
                inner.clear();
                inner.push("raise GanMatchError(\"no cond clause was truthy\")".to_string());
            }
            push_indented(out, &inner);
        }
        Ok(())
    }

    fn emit_with(&mut self, call: &Call, dest: Dest, out: &mut Vec<String>) -> Result<()> {
        // desugar: with p1 <- e1, p2 <- e2 do body else clauses end
        // into nested case expressions (GEP-0001-R006)
        let body = Term::keyword_arg(&call.args, "do")
            .ok_or_else(|| self.err(call.span, "with requires a do block"))?
            .clone();
        let else_block = Term::keyword_arg(&call.args, "else").cloned();
        let steps: Vec<&Term> = call
            .args
            .iter()
            .filter(|a| !matches!(a, Term::Pair(_, _)))
            .collect();
        let mut current = body;
        for step in steps.iter().rev() {
            match step {
                Term::Call(c) if matches!(&c.callee, Callee::Name(n) if n == "<-") => {
                    let pat = c.args[0].clone();
                    let expr = c.args[1].clone();
                    let fail_var = Term::Var("_gan_with_fail".into(), Some(0));
                    let fail_body = match &else_block {
                        Some(eb) => Term::Call(Box::new(Call {
                            callee: Callee::Name("case".into()),
                            args: vec![fail_var.clone(), Term::Pair("do".into(), Box::new(eb.clone()))],
                            span: c.span,
                        })),
                        None => fail_var.clone(),
                    };
                    let clauses = Term::call(
                        "__clauses__",
                        vec![
                            Term::call("->", vec![Term::List(vec![pat]), current], c.span),
                            Term::call(
                                "->",
                                vec![Term::List(vec![fail_var]), fail_body],
                                c.span,
                            ),
                        ],
                        c.span,
                    );
                    current = Term::Call(Box::new(Call {
                        callee: Callee::Name("case".into()),
                        args: vec![expr, Term::Pair("do".into(), Box::new(clauses))],
                        span: c.span,
                    }));
                }
                other => {
                    // a bare expression step: evaluate for effect
                    current = Term::block(vec![(*other).clone(), current], other.span());
                }
            }
        }
        self.emit_stmt(&current, dest, out)
    }

    // ---- expressions -----------------------------------------------------

    /// Emit an expression that will be used as a Python condition.
    fn emit_bool_expr(&mut self, term: &Term, pre: &mut Vec<String>) -> Result<String> {
        let e = self.emit_expr(term, pre)?;
        if self.is_boolean_shaped(term) {
            Ok(e)
        } else {
            self.helpers.insert("truthy");
            Ok(format!("_gan_truthy({e})"))
        }
    }

    fn is_boolean_shaped(&self, term: &Term) -> bool {
        match term {
            Term::Bool(_) => true,
            Term::Call(c) => match &c.callee {
                Callee::Name(n) => matches!(
                    n.as_str(),
                    "==" | "!=" | "<" | ">" | "<=" | ">=" | "not" | "and" | "or"
                ) && {
                    if n == "and" || n == "or" {
                        c.args.iter().all(|a| self.is_boolean_shaped(a))
                    } else {
                        true
                    }
                },
                _ => false,
            },
            _ => false,
        }
    }

    fn emit_expr(&mut self, term: &Term, pre: &mut Vec<String>) -> Result<String> {
        match term {
            Term::Int(n) => Ok(n.to_string()),
            Term::Float(f) => {
                let s = format!("{f}");
                if s.contains('.') || s.contains('e') || s.contains("inf") || s.contains("nan") {
                    Ok(s)
                } else {
                    Ok(format!("{s}.0"))
                }
            }
            Term::Bool(b) => Ok(if *b { "True" } else { "False" }.to_string()),
            Term::Nil => Ok("None".to_string()),
            Term::Atom(a) => Ok(py_str_lit(a)),
            Term::Str(parts) => self.emit_string(parts, pre),
            Term::Var(name, ctx) => Ok(hygienic_name(name, *ctx)),
            Term::Alias(segs) => {
                // a bare module reference
                let resolved = self.resolve_alias(segs);
                let path = module_py_path(&resolved);
                self.gan_imports.insert(path.clone());
                Ok(path)
            }
            Term::List(items) => {
                // keyword list -> list of tuples (GEP-0001-R009)
                let rendered: Vec<String> = items
                    .iter()
                    .map(|i| match i {
                        Term::Pair(k, v) => {
                            let ve = self.emit_expr(v, pre)?;
                            Ok(format!("({}, {ve})", py_str_lit(k)))
                        }
                        Term::Call(c) if matches!(&c.callee, Callee::Name(n) if n == "|") => {
                            Err(self.err(c.span, "cons expressions are only valid in patterns; build lists with [head] ++ tail"))
                        }
                        other => self.emit_expr(other, pre),
                    })
                    .collect::<Result<_>>()?;
                Ok(format!("[{}]", rendered.join(", ")))
            }
            Term::Tuple(items) => {
                let rendered: Vec<String> = items
                    .iter()
                    .map(|i| self.emit_expr(i, pre))
                    .collect::<Result<_>>()?;
                if rendered.len() == 1 {
                    Ok(format!("({},)", rendered[0]))
                } else {
                    Ok(format!("({})", rendered.join(", ")))
                }
            }
            Term::Map(entries) => {
                let rendered: Vec<String> = entries
                    .iter()
                    .map(|(k, v)| {
                        let ke = self.emit_expr(k, pre)?;
                        let ve = self.emit_expr(v, pre)?;
                        Ok(format!("{ke}: {ve}"))
                    })
                    .collect::<Result<_>>()?;
                Ok(format!("{{{}}}", rendered.join(", ")))
            }
            Term::Pair(_, _) => Err(self.err(
                Span::default(),
                "a keyword pair is not valid in this position",
            )),
            Term::Call(call) => self.emit_call(call, pre),
        }
    }

    fn emit_string(&mut self, parts: &[StrPart], pre: &mut Vec<String>) -> Result<String> {
        if parts.iter().all(|p| matches!(p, StrPart::Text(_))) {
            let mut text = String::new();
            for p in parts {
                if let StrPart::Text(t) = p {
                    text.push_str(t);
                }
            }
            return Ok(py_str_lit(&text));
        }
        let mut body = String::new();
        for p in parts {
            match p {
                StrPart::Text(t) => body.push_str(&escape_py_str(t, true)),
                StrPart::Interp(e) => {
                    let ee = self.emit_expr(e, pre)?;
                    let _ = write!(body, "{{{ee}}}");
                }
            }
        }
        Ok(format!("f\"{body}\""))
    }

    fn resolve_alias(&self, segs: &[String]) -> Vec<String> {
        if let Some(full) = self.aliases.get(&segs[0]) {
            let mut out = full.clone();
            out.extend(segs[1..].iter().cloned());
            out
        } else {
            segs.to_vec()
        }
    }

    fn emit_args(&mut self, args: &[Term], pre: &mut Vec<String>) -> Result<String> {
        let mut rendered = Vec::new();
        let n = args.len();
        // a trailing run of pairs becomes Python keyword arguments (GEP-0003-R005)
        let mut kw_start = n;
        while kw_start > 0 && matches!(args[kw_start - 1], Term::Pair(_, _)) {
            kw_start -= 1;
        }
        for (i, a) in args.iter().enumerate() {
            if i >= kw_start {
                let Term::Pair(k, v) = a else { unreachable!() };
                let ve = self.emit_expr(v, pre)?;
                rendered.push(format!("{}={ve}", map_ident(k)));
            } else {
                rendered.push(self.emit_expr(a, pre)?);
            }
        }
        Ok(rendered.join(", "))
    }

    fn emit_call(&mut self, call: &Call, pre: &mut Vec<String>) -> Result<String> {
        let span = call.span;
        match &call.callee {
            Callee::Name(name) => self.emit_named_call(name, call, pre),
            Callee::Dot {
                base,
                name,
                is_call,
            } => {
                match base.as_ref() {
                    // :module.fun(...) — remote atom call (GEP-0003-R001/R002)
                    Term::Atom(module) => {
                        self.py_imports.insert(module.clone());
                        let f = map_ident(name);
                        if *is_call {
                            let args = self.emit_args(&call.args, pre)?;
                            Ok(format!("{module}.{f}({args})"))
                        } else {
                            Ok(format!("{module}.{f}"))
                        }
                    }
                    // Mod.fun(...) — Gandora cross-module call (GEP-0001-R017)
                    Term::Alias(segs) => {
                        let resolved = self.resolve_alias(segs);
                        if resolved == vec!["IO".to_string()] {
                            return self.emit_io_call(name, call, pre);
                        }
                        let path = module_py_path(&resolved);
                        self.gan_imports.insert(path.clone());
                        let f = map_ident(name);
                        if *is_call {
                            let args = self.emit_args(&call.args, pre)?;
                            Ok(format!("{path}.{f}({args})"))
                        } else {
                            Ok(format!("{path}.{f}"))
                        }
                    }
                    // expr.name / expr.name(...) — postfix access (GEP-0003-R004)
                    other => {
                        let b = self.emit_expr(other, pre)?;
                        let needs_paren = matches!(other, Term::Int(_) | Term::Float(_));
                        let b = if needs_paren { format!("({b})") } else { b };
                        let f = map_ident(name);
                        if *is_call {
                            let args = self.emit_args(&call.args, pre)?;
                            Ok(format!("{b}.{f}({args})"))
                        } else {
                            Ok(format!("{b}.{f}"))
                        }
                    }
                }
            }
            Callee::Apply(f) => {
                let fe = self.emit_expr(f, pre)?;
                let args = self.emit_args(&call.args, pre)?;
                let fe = if fe.starts_with("lambda") {
                    format!("({fe})")
                } else {
                    fe
                };
                Ok(format!("{fe}({args})"))
            }
        }
        .map_err(|d: Diagnostic| {
            let mut d = d;
            if d.span == Span::default() {
                d.span = span;
            }
            d
        })
    }

    fn emit_io_call(&mut self, name: &str, call: &Call, pre: &mut Vec<String>) -> Result<String> {
        match name {
            "puts" => {
                let args = self.emit_args(&call.args, pre)?;
                Ok(format!("print({args})"))
            }
            "inspect" => {
                self.helpers.insert("inspect");
                let args = self.emit_args(&call.args, pre)?;
                Ok(format!("_gan_inspect({args})"))
            }
            other => Err(self.err(
                call.span,
                format!("IO.{other} is not supported (only IO.puts and IO.inspect)"),
            )),
        }
    }

    fn emit_named_call(&mut self, name: &str, call: &Call, pre: &mut Vec<String>) -> Result<String> {
        let args = &call.args;
        match (name, args.len()) {
            ("+", 2) | ("-", 2) | ("*", 2) => {
                let a = self.emit_operand(&args[0], name, pre)?;
                let b = self.emit_operand(&args[1], name, pre)?;
                Ok(format!("{a} {name} {b}"))
            }
            ("/", 2) => {
                let a = self.emit_operand(&args[0], name, pre)?;
                let b = self.emit_operand(&args[1], name, pre)?;
                Ok(format!("{a} / {b}"))
            }
            ("//", 2) | ("div", 2) => {
                self.helpers.insert("intdiv");
                let a = self.emit_expr(&args[0], pre)?;
                let b = self.emit_expr(&args[1], pre)?;
                Ok(format!("_gan_div({a}, {b})"))
            }
            ("rem", 2) => {
                self.helpers.insert("intdiv");
                let a = self.emit_expr(&args[0], pre)?;
                let b = self.emit_expr(&args[1], pre)?;
                Ok(format!("_gan_rem({a}, {b})"))
            }
            ("==", 2) | ("!=", 2) | ("<", 2) | (">", 2) | ("<=", 2) | (">=", 2) => {
                let a = self.emit_operand(&args[0], name, pre)?;
                let b = self.emit_operand(&args[1], name, pre)?;
                Ok(format!("{a} {name} {b}"))
            }
            ("and", 2) | ("or", 2) => {
                if self.is_boolean_shaped(&args[0]) && self.is_boolean_shaped(&args[1]) {
                    let a = self.emit_operand(&args[0], name, pre)?;
                    let b = self.emit_operand(&args[1], name, pre)?;
                    return Ok(format!("{a} {name} {b}"));
                }
                // Elixir truthiness with single evaluation (GEP-0001-R010)
                self.helpers.insert("truthy");
                let helper = if name == "and" { "_gan_and" } else { "_gan_or" };
                self.helpers.insert(if name == "and" { "and" } else { "or" });
                let a = self.emit_expr(&args[0], pre)?;
                let b = self.emit_expr(&args[1], pre)?;
                Ok(format!("{helper}({a}, lambda: {b})"))
            }
            ("not", 1) => {
                let e = self.emit_bool_expr(&args[0], pre)?;
                Ok(format!("not ({e})"))
            }
            ("-", 1) => {
                let e = self.emit_expr(&args[0], pre)?;
                Ok(format!("-({e})"))
            }
            ("++", 2) => {
                let a = self.emit_operand(&args[0], "+", pre)?;
                let b = self.emit_operand(&args[1], "+", pre)?;
                Ok(format!("{a} + {b}"))
            }
            ("<>", 2) => {
                let a = self.emit_operand(&args[0], "+", pre)?;
                let b = self.emit_operand(&args[1], "+", pre)?;
                Ok(format!("{a} + {b}"))
            }
            ("..", 2) => {
                let a = self.emit_expr(&args[0], pre)?;
                let b = self.emit_expr(&args[1], pre)?;
                Ok(format!("range({a}, ({b}) + 1)"))
            }
            ("=", 2) => {
                // match in expression position
                let t = self.fresh_tmp("m");
                let mut lines = Vec::new();
                self.emit_match(call, Dest::Assign(t), &mut lines)?;
                pre.extend(lines);
                Ok(self.tmp(t).to_string())
            }
            ("__block__", _) => {
                let stmts = Term::Call(Box::new(call.clone())).as_block();
                if stmts.len() == 1 {
                    return self.emit_expr(&stmts[0], pre);
                }
                let t = self.fresh_tmp("tmp");
                let mut lines = Vec::new();
                self.emit_stmt(&Term::Call(Box::new(call.clone())), Dest::Assign(t), &mut lines)?;
                pre.extend(lines);
                Ok(self.tmp(t).to_string())
            }
            ("if", _) | ("unless", _) | ("case", _) | ("cond", _) | ("with", _) => {
                let t = self.fresh_tmp("tmp");
                let mut lines = Vec::new();
                self.emit_stmt(&Term::Call(Box::new(call.clone())), Dest::Assign(t), &mut lines)?;
                pre.extend(lines);
                Ok(self.tmp(t).to_string())
            }
            ("fn", _) => self.emit_fn(call, pre),
            ("&", 1) => self.emit_capture(call, pre),
            ("^", 1) => Err(self.err(call.span, "pin (^) is only valid inside patterns")),
            ("|", 2) => Err(self.err(
                call.span,
                "cons expressions are only valid in patterns; build lists with [head] ++ tail",
            )),
            ("length", 1) => {
                let e = self.emit_expr(&args[0], pre)?;
                Ok(format!("len({e})"))
            }
            ("hd", 1) => {
                let e = self.emit_expr(&args[0], pre)?;
                Ok(format!("{e}[0]"))
            }
            ("tl", 1) => {
                let e = self.emit_expr(&args[0], pre)?;
                Ok(format!("{e}[1:]"))
            }
            ("elem", 2) => {
                let a = self.emit_expr(&args[0], pre)?;
                let b = self.emit_expr(&args[1], pre)?;
                Ok(format!("{a}[{b}]"))
            }
            ("to_string", 1) => {
                let e = self.emit_expr(&args[0], pre)?;
                Ok(format!("str({e})"))
            }
            ("inspect", 1) => {
                let e = self.emit_expr(&args[0], pre)?;
                Ok(format!("repr({e})"))
            }
            ("is_nil", 1) => {
                let e = self.emit_expr(&args[0], pre)?;
                Ok(format!("({e} is None)"))
            }
            ("is_list", 1) => {
                let e = self.emit_expr(&args[0], pre)?;
                Ok(format!("isinstance({e}, list)"))
            }
            ("is_tuple", 1) => {
                let e = self.emit_expr(&args[0], pre)?;
                Ok(format!("isinstance({e}, tuple)"))
            }
            ("is_map", 1) => {
                let e = self.emit_expr(&args[0], pre)?;
                Ok(format!("isinstance({e}, dict)"))
            }
            ("is_binary", 1) | ("is_atom", 1) => {
                let e = self.emit_expr(&args[0], pre)?;
                Ok(format!("isinstance({e}, str)"))
            }
            ("is_integer", 1) => {
                let e = self.emit_expr(&args[0], pre)?;
                Ok(format!("isinstance({e}, int)"))
            }
            ("is_float", 1) => {
                let e = self.emit_expr(&args[0], pre)?;
                Ok(format!("isinstance({e}, float)"))
            }
            ("is_function", 1) => {
                let e = self.emit_expr(&args[0], pre)?;
                Ok(format!("callable({e})"))
            }
            ("is_boolean", 1) => {
                let e = self.emit_expr(&args[0], pre)?;
                Ok(format!("isinstance({e}, bool)"))
            }
            ("abs", 1) | ("max", 2) | ("min", 2) | ("round", 1) => {
                let a = self.emit_args(args, pre)?;
                Ok(format!("{name}({a})"))
            }
            ("trunc", 1) => {
                let e = self.emit_expr(&args[0], pre)?;
                Ok(format!("int({e})"))
            }
            ("map_size", 1) | ("tuple_size", 1) => {
                let e = self.emit_expr(&args[0], pre)?;
                Ok(format!("len({e})"))
            }
            ("quote", _) | ("unquote", _) | ("unquote_splicing", _) => Err(self.err(
                call.span,
                "quote/unquote are only valid inside defmacro bodies (GEP-0002)",
            )),
            ("raise", _) => {
                let t = self.fresh_tmp("tmp");
                let mut lines = Vec::new();
                self.emit_stmt(&Term::Call(Box::new(call.clone())), Dest::Ignore, &mut lines)?;
                pre.extend(lines);
                pre.push(format!("{} = None", self.tmp(t)));
                Ok(self.tmp(t).to_string())
            }
            _ => {
                // unsupported special surface forms produce a named diagnostic
                const UNSUPPORTED: &[&str] = &[
                    "defstruct", "defprotocol", "defimpl", "receive", "try", "for", "send",
                    "spawn", "defdelegate", "defguard", "sigil",
                ];
                if UNSUPPORTED.contains(&name) {
                    return Err(self.err(
                        call.span,
                        format!("'{name}' is not part of the v0 surface (GEP-0001-R007)"),
                    ));
                }
                let mut f = map_ident(name);
                if self
                    .private_funs
                    .contains(&(name.to_string(), args.len()))
                {
                    f.insert(0, '_');
                }
                let rendered = self.emit_args(args, pre)?;
                Ok(format!("{f}({rendered})"))
            }
        }
    }

    fn emit_operand(&mut self, term: &Term, _parent: &str, pre: &mut Vec<String>) -> Result<String> {
        let e = self.emit_expr(term, pre)?;
        let atomic = match term {
            Term::Call(c) => match &c.callee {
                Callee::Name(n) => {
                    !matches!(
                        n.as_str(),
                        "+" | "-"
                            | "*"
                            | "/"
                            | "=="
                            | "!="
                            | "<"
                            | ">"
                            | "<="
                            | ">="
                            | "and"
                            | "or"
                            | "not"
                            | "++"
                            | "<>"
                            | ".."
                    ) || c.args.len() != 2
                }
                _ => true,
            },
            _ => true,
        };
        if atomic || e.starts_with('(') {
            Ok(e)
        } else {
            Ok(format!("({e})"))
        }
    }

    fn emit_fn(&mut self, call: &Call, pre: &mut Vec<String>) -> Result<String> {
        let clauses = &call.args;
        // single clause with plain variable params and a single expression body
        if clauses.len() == 1 {
            if let Term::Call(c) = &clauses[0] {
                if let (Term::List(pats), body) = (&c.args[0], &c.args[1]) {
                    let plain = pats
                        .iter()
                        .all(|p| matches!(p, Term::Var(n, _) if n != "_"));
                    let stmts = body.as_block();
                    if plain && stmts.len() == 1 && split_when(&pats.first().cloned().unwrap_or(Term::Nil)).1.is_none() {
                        let mut inner_pre = Vec::new();
                        let e = self.emit_expr(&stmts[0], &mut inner_pre)?;
                        if inner_pre.is_empty() {
                            let names: Vec<String> = pats
                                .iter()
                                .map(|p| match p {
                                    Term::Var(n, ctx) => hygienic_name(n, *ctx),
                                    _ => unreachable!(),
                                })
                                .collect();
                            return Ok(format!("lambda {}: {e}", names.join(", ")));
                        }
                    }
                }
            }
        }
        // otherwise hoist a def
        let fname = format!("_gan_fn{}", self.fn_counter);
        self.fn_counter += 1;
        let fdef = FnDef {
            name: fname.clone(),
            private: false,
            doc: None,
            decorators: Vec::new(),
            clauses: clauses
                .iter()
                .map(|clause| {
                    let Term::Call(c) = clause else {
                        return Err(self.err(call.span, "fn requires -> clauses"));
                    };
                    let Term::List(pats) = &c.args[0] else {
                        return Err(self.err(call.span, "fn requires -> clauses"));
                    };
                    let mut params = Vec::new();
                    let mut guard = None;
                    for (i, p) in pats.iter().enumerate() {
                        let (pat, g) = split_when(p);
                        if g.is_some() {
                            if i + 1 != pats.len() {
                                return Err(
                                    self.err(c.span, "when must follow the last parameter")
                                );
                            }
                            guard = g.cloned();
                        }
                        params.push(pat.clone());
                    }
                    Ok((params, guard, c.args[1].clone()))
                })
                .collect::<Result<_>>()?,
            span: call.span,
        };
        let code = self.compile_fun(&fdef)?;
        for line in code.trim_matches('\n').lines() {
            pre.push(line.to_string());
        }
        Ok(map_ident(&fname))
    }

    fn emit_capture(&mut self, call: &Call, pre: &mut Vec<String>) -> Result<String> {
        let inner = &call.args[0];
        // &Mod.fun/1 and &fun/1 arrive as (& (/ target arity))
        if let Term::Call(c) = inner {
            if matches!(&c.callee, Callee::Name(n) if n == "/") && c.args.len() == 2 {
                if let Term::Int(_) = c.args[1] {
                    return match &c.args[0] {
                        Term::Var(n, _) => Ok(map_ident(n)),
                        t @ Term::Call(cc) if matches!(&cc.callee, Callee::Dot { .. }) => {
                            self.emit_expr(t, pre)
                        }
                        other => Err(self.err(
                            c.span,
                            format!("cannot capture {other:?}"),
                        )),
                    };
                }
            }
        }
        // &(expr) with &1..&n placeholders
        let max = max_placeholder(inner);
        if max == 0 {
            return Err(self.err(
                call.span,
                "a capture expression must use &1-style placeholders or name/arity",
            ));
        }
        let renamed = rename_placeholders(inner);
        let mut inner_pre = Vec::new();
        let e = self.emit_expr(&renamed, &mut inner_pre)?;
        if !inner_pre.is_empty() {
            return Err(self.err(
                call.span,
                "capture bodies must be simple expressions; use fn ... end instead",
            ));
        }
        let params: Vec<String> = (1..=max).map(|i| format!("_gan_cap{i}")).collect();
        pre.extend(Vec::<String>::new());
        Ok(format!("lambda {}: {e}", params.join(", ")))
    }

    // ---- patterns --------------------------------------------------------

    fn compile_pattern(&mut self, pat: &Term, guards: &mut Vec<String>) -> Result<String> {
        match pat {
            Term::Int(n) => Ok(n.to_string()),
            Term::Float(f) => Ok(format!("{f}")),
            Term::Bool(b) => Ok(if *b { "True" } else { "False" }.to_string()),
            Term::Nil => Ok("None".to_string()),
            Term::Atom(a) => Ok(py_str_lit(a)),
            Term::Str(parts) => match Term::Str(parts.clone()).as_plain_str() {
                Some(s) => Ok(py_str_lit(&s)),
                None => Err(self.err(
                    Span::default(),
                    "interpolated strings are not valid patterns",
                )),
            },
            Term::Var(n, _) if n == "_" => Ok("_".to_string()),
            Term::Var(n, ctx) => Ok(hygienic_name(n, *ctx)),
            Term::Tuple(items) => {
                let ps: Vec<String> = items
                    .iter()
                    .map(|p| self.compile_pattern(p, guards))
                    .collect::<Result<_>>()?;
                let t = self.fresh_tmp("t");
                let tname = self.tmp(t).to_string();
                guards.push(format!("isinstance({tname}, tuple)"));
                if ps.len() == 1 {
                    Ok(format!("({},) as {tname}", ps[0]))
                } else {
                    Ok(format!("({}) as {tname}", ps.join(", ")))
                }
            }
            Term::List(items) => {
                // [a, b | tail]
                if let Some(Term::Call(c)) = items.last() {
                    if matches!(&c.callee, Callee::Name(n) if n == "|") {
                        let fixed = &items[..items.len() - 1];
                        let mut ps: Vec<String> = fixed
                            .iter()
                            .map(|p| self.compile_pattern(p, guards))
                            .collect::<Result<_>>()?;
                        ps.push(self.compile_pattern(&c.args[0], guards)?);
                        let tail = match &c.args[1] {
                            Term::Var(n, _) if n == "_" => "*_".to_string(),
                            Term::Var(n, ctx) => format!("*{}", hygienic_name(n, *ctx)),
                            _ => {
                                return Err(self.err(
                                    c.span,
                                    "the tail of a cons pattern must be a variable or _",
                                ))
                            }
                        };
                        ps.push(tail);
                        let t = self.fresh_tmp("l");
                        let tname = self.tmp(t).to_string();
                        guards.push(format!("isinstance({tname}, list)"));
                        return Ok(format!("[{}] as {tname}", ps.join(", ")));
                    }
                }
                let ps: Vec<String> = items
                    .iter()
                    .map(|p| match p {
                        Term::Pair(k, v) => {
                            let vp = self.compile_pattern(v, guards)?;
                            Ok(format!("({}, {vp})", py_str_lit(k)))
                        }
                        other => self.compile_pattern(other, guards),
                    })
                    .collect::<Result<_>>()?;
                let t = self.fresh_tmp("l");
                let tname = self.tmp(t).to_string();
                guards.push(format!("isinstance({tname}, list)"));
                Ok(format!("[{}] as {tname}", ps.join(", ")))
            }
            Term::Map(entries) => {
                let ps: Vec<String> = entries
                    .iter()
                    .map(|(k, v)| {
                        let kp = match k {
                            Term::Atom(a) => py_str_lit(a),
                            Term::Int(n) => n.to_string(),
                            Term::Str(parts) => Term::Str(parts.clone())
                                .as_plain_str()
                                .map(|s| py_str_lit(&s))
                                .ok_or_else(|| {
                                    self.err(
                                        Span::default(),
                                        "map pattern keys must be literals",
                                    )
                                })?,
                            _ => {
                                return Err(self.err(
                                    Span::default(),
                                    "map pattern keys must be literals",
                                ))
                            }
                        };
                        let vp = self.compile_pattern(v, guards)?;
                        Ok(format!("{kp}: {vp}"))
                    })
                    .collect::<Result<_>>()?;
                Ok(format!("{{{}}}", ps.join(", ")))
            }
            Term::Call(c) => {
                if matches!(&c.callee, Callee::Name(n) if n == "^") {
                    let t = self.fresh_tmp("pin");
                    let tname = self.tmp(t).to_string();
                    let mut pre = Vec::new();
                    let e = self.emit_expr(&c.args[0], &mut pre)?;
                    if !pre.is_empty() {
                        return Err(
                            self.err(c.span, "pin expressions must be simple expressions")
                        );
                    }
                    guards.push(format!("{tname} == {e}"));
                    return Ok(tname);
                }
                Err(self.err(
                    c.span,
                    "this expression is not a valid pattern",
                ))
            }
            Term::Pair(_, _) => Err(self.err(
                Span::default(),
                "a keyword pair is not a valid pattern here",
            )),
            Term::Alias(_) => Err(self.err(
                Span::default(),
                "module names are not valid patterns",
            )),
        }
    }

    fn helper_code(&self) -> String {
        let mut out = String::new();
        if self.helpers.contains("truthy") {
            out.push_str(
                "\ndef _gan_truthy(value):\n    return value is not None and value is not False\n",
            );
        }
        if self.helpers.contains("and") {
            out.push_str(
                "\ndef _gan_and(value, then):\n    return then() if _gan_truthy(value) else value\n",
            );
        }
        if self.helpers.contains("or") {
            out.push_str(
                "\ndef _gan_or(value, then):\n    return value if _gan_truthy(value) else then()\n",
            );
        }
        if self.helpers.contains("intdiv") {
            out.push_str(
                "\ndef _gan_div(a, b):\n    q = a // b\n    if q < 0 and q * b != a:\n        q += 1\n    return q\n\n\ndef _gan_rem(a, b):\n    return a - _gan_div(a, b) * b\n",
            );
        }
        if self.helpers.contains("match_error") {
            out.push_str("\nclass GanMatchError(Exception):\n    pass\n");
        }
        if self.helpers.contains("inspect") {
            out.push_str(
                "\ndef _gan_inspect(value):\n    print(repr(value))\n    return value\n",
            );
        }
        out
    }
}

fn split_when(pat: &Term) -> (&Term, Option<&Term>) {
    if let Term::Call(c) = pat {
        if matches!(&c.callee, Callee::Name(n) if n == "when") && c.args.len() == 2 {
            return (&c.args[0], Some(&c.args[1]));
        }
    }
    (pat, None)
}

fn hygienic_name(name: &str, ctx: Option<u64>) -> String {
    match ctx {
        Some(id) => format!("{}__gan{id}", map_ident(name)),
        None => map_ident(name),
    }
}

fn max_placeholder(term: &Term) -> usize {
    match term {
        Term::Var(n, _) if n.starts_with('&') => n[1..].parse().unwrap_or(0),
        Term::List(items) | Term::Tuple(items) => {
            items.iter().map(max_placeholder).max().unwrap_or(0)
        }
        Term::Map(entries) => entries
            .iter()
            .map(|(k, v)| max_placeholder(k).max(max_placeholder(v)))
            .max()
            .unwrap_or(0),
        Term::Pair(_, v) => max_placeholder(v),
        Term::Str(parts) => parts
            .iter()
            .map(|p| match p {
                StrPart::Interp(e) => max_placeholder(e),
                _ => 0,
            })
            .max()
            .unwrap_or(0),
        Term::Call(c) => {
            let base = match &c.callee {
                Callee::Dot { base, .. } => max_placeholder(base),
                Callee::Apply(f) => max_placeholder(f),
                Callee::Name(_) => 0,
            };
            base.max(c.args.iter().map(max_placeholder).max().unwrap_or(0))
        }
        _ => 0,
    }
}

fn rename_placeholders(term: &Term) -> Term {
    match term {
        Term::Var(n, ctx) if n.starts_with('&') => {
            Term::Var(format!("_gan_cap{}", &n[1..]), *ctx)
        }
        Term::List(items) => Term::List(items.iter().map(rename_placeholders).collect()),
        Term::Tuple(items) => Term::Tuple(items.iter().map(rename_placeholders).collect()),
        Term::Map(entries) => Term::Map(
            entries
                .iter()
                .map(|(k, v)| (rename_placeholders(k), rename_placeholders(v)))
                .collect(),
        ),
        Term::Pair(k, v) => Term::Pair(k.clone(), Box::new(rename_placeholders(v))),
        Term::Str(parts) => Term::Str(
            parts
                .iter()
                .map(|p| match p {
                    StrPart::Interp(e) => StrPart::Interp(Box::new(rename_placeholders(e))),
                    t => t.clone(),
                })
                .collect(),
        ),
        Term::Call(c) => {
            let mut new_c = c.as_ref().clone();
            match &mut new_c.callee {
                Callee::Dot { base, .. } => **base = rename_placeholders(base),
                Callee::Apply(f) => **f = rename_placeholders(f),
                Callee::Name(_) => {}
            }
            new_c.args = new_c.args.iter().map(rename_placeholders).collect();
            Term::Call(Box::new(new_c))
        }
        other => other.clone(),
    }
}

fn py_str_lit(s: &str) -> String {
    format!("\"{}\"", escape_py_str(s, false))
}

fn escape_py_str(s: &str, fstring: bool) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            '{' if fstring => out.push_str("{{"),
            '}' if fstring => out.push_str("}}"),
            other => out.push(other),
        }
    }
    out
}

fn dotted_name(t: &Term) -> Option<String> {
    match t {
        Term::Var(n, _) => Some(n.clone()),
        Term::Call(c) => match &c.callee {
            Callee::Dot {
                base,
                name,
                is_call: false,
            } => Some(format!("{}.{name}", dotted_name(base)?)),
            _ => None,
        },
        _ => None,
    }
}

fn push_indented(out: &mut Vec<String>, lines: &[String]) {
    for l in lines {
        if l.is_empty() {
            out.push(String::new());
        } else {
            out.push(format!("    {l}"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expander::{collect_macros, Expander};
    use crate::parser::parse_file;

    fn compile(src: &str) -> String {
        let module = parse_file("<test>", src).unwrap();
        let macros = collect_macros("<test>", &module).unwrap();
        let mut ex = Expander::new("<test>", macros);
        let expanded = ex.expand_module(&module).unwrap();
        let mut cg = Codegen::new("<test>", vec![]);
        cg.compile(&expanded).unwrap()
    }

    fn compile_err(src: &str) -> String {
        let module = parse_file("<test>", src).unwrap();
        let macros = collect_macros("<test>", &module).unwrap();
        let mut ex = Expander::new("<test>", macros);
        let expanded = ex.expand_module(&module).unwrap();
        let mut cg = Codegen::new("<test>", vec![]);
        cg.compile(&expanded).unwrap_err().message
    }

    #[test]
    fn compiles_hello_world() {
        let py = compile(
            "defmodule Hello do\n  def main() do\n    IO.puts(\"Hello, world!\")\n  end\nend",
        );
        assert!(py.contains("def main():"), "{py}");
        assert!(py.contains("return print(\"Hello, world!\")"), "{py}");
        assert!(py.contains("if __name__ == \"__main__\":"), "{py}");
    }

    #[test]
    fn compiles_remote_atom_call() {
        let py = compile(
            "defmodule M do\n  def f(x) do\n    :math.sqrt(x)\n  end\nend",
        );
        assert!(py.contains("import math"), "{py}");
        assert!(py.contains("return math.sqrt(x)"), "{py}");
    }

    #[test]
    fn compiles_pipe_and_interpolation() {
        let py = compile(
            "defmodule M do\n  def f(name) do\n    \"hi #{name}!\"\n  end\nend",
        );
        assert!(py.contains("return f\"hi {name}!\""), "{py}");
    }

    #[test]
    fn compiles_multi_clause_with_patterns() {
        let py = compile(
            "defmodule M do\n  def fact(0), do: 1\n  def fact(n), do: n * fact(n - 1)\nend",
        );
        assert!(py.contains("def fact(*_gan_args):"), "{py}");
        assert!(py.contains("case (0,):"), "{py}");
        assert!(py.contains("return n * fact(n - 1)"), "{py}");
        assert!(py.contains("GanMatchError"), "{py}");
    }

    #[test]
    fn compiles_case_with_tuple_patterns() {
        let py = compile(
            "defmodule M do\n  def f(r) do\n    case r do\n      {:ok, v} -> v\n      {:error, _} -> nil\n    end\n  end\nend",
        );
        assert!(py.contains("match _gan_case0:"), "{py}");
        assert!(py.contains("(\"ok\", v)"), "{py}");
        assert!(py.contains("isinstance(_gan_t"), "{py}");
    }

    #[test]
    fn compiles_if_as_expression() {
        let py = compile(
            "defmodule M do\n  def f(x) do\n    y = if x > 0 do\n      :pos\n    else\n      :neg\n    end\n    y\n  end\nend",
        );
        assert!(py.contains("if x > 0:"), "{py}");
        assert!(py.contains("_gan_tmp0 = \"pos\""), "{py}");
        assert!(py.contains("y = _gan_tmp0"), "{py}");
    }

    #[test]
    fn compiles_keyword_args() {
        let py = compile(
            "defmodule M do\n  def f(data) do\n    :json.dumps(data, indent: 2)\n  end\nend",
        );
        assert!(py.contains("json.dumps(data, indent=2)"), "{py}");
    }

    #[test]
    fn compiles_pyimport_alias() {
        let py = compile(
            "defmodule M do\n  pyimport numpy, as: np\n  def f(xs) do\n    np.array(xs)\n  end\nend",
        );
        assert!(py.contains("import numpy as np"), "{py}");
        assert!(py.contains("return np.array(xs)"), "{py}");
    }

    #[test]
    fn compiles_cross_module_call() {
        let py = compile(
            "defmodule M do\n  alias App.Helpers, as: H\n  def f(x) do\n    H.tidy(x)\n  end\nend",
        );
        assert!(py.contains("import app.helpers"), "{py}");
        assert!(py.contains("return app.helpers.tidy(x)"), "{py}");
    }

    #[test]
    fn compiles_anonymous_fn_and_capture() {
        let py = compile(
            "defmodule M do\n  def f(xs) do\n    g = fn x -> x * 2 end\n    h = &(&1 + 1)\n    {g, h, xs}\n  end\nend",
        );
        assert!(py.contains("g = lambda x: x * 2"), "{py}");
        assert!(py.contains("h = lambda _gan_cap1: _gan_cap1 + 1"), "{py}");
    }

    #[test]
    fn compiles_truthiness() {
        let py = compile(
            "defmodule M do\n  def f(x) do\n    if x do\n      1\n    end\n  end\nend",
        );
        assert!(py.contains("_gan_truthy(x)"), "{py}");
    }

    #[test]
    fn compiles_decorators() {
        let py = compile(
            "defmodule M do\n  @decorate :functools.cache\n  def f(x) do\n    x\n  end\nend",
        );
        assert!(py.contains("@functools.cache"), "{py}");
        assert!(py.contains("import functools"), "{py}");
    }

    #[test]
    fn compiles_docstrings() {
        let py = compile(
            "defmodule M do\n  @moduledoc \"module doc\"\n  @doc \"fn doc\"\n  def f(), do: 1\nend",
        );
        assert!(py.starts_with("\"\"\"module doc\"\"\""), "{py}");
        assert!(py.contains("\"\"\"fn doc\"\"\""), "{py}");
    }

    #[test]
    fn compiles_cons_pattern_match() {
        let py = compile(
            "defmodule M do\n  def f(xs) do\n    [h | t] = xs\n    {h, t}\n  end\nend",
        );
        assert!(py.contains("[h, *t]"), "{py}");
    }

    #[test]
    fn rejects_unsupported_forms() {
        let msg = compile_err(
            "defmodule M do\n  def f(x) do\n    receive(x)\n  end\nend",
        );
        assert!(msg.contains("not part of the v0 surface"), "{msg}");
    }

    #[test]
    fn rejects_module_name_mismatch() {
        let module = parse_file("<test>", "defmodule Wrong do\nend").unwrap();
        let mut cg = Codegen::new("<test>", vec!["App".into(), "Right".into()]);
        let err = cg.compile(&module).unwrap_err();
        assert!(err.message.contains("does not match"), "{}", err.message);
    }

    #[test]
    fn maps_identifiers_injectively() {
        assert_eq!(map_ident("empty?"), "empty_p");
        assert_eq!(map_ident("save!"), "save_bang");
        assert_eq!(map_ident("valid"), "valid");
        assert_eq!(module_py_path(&["App".into(), "HelloWeb".into()]), "app.hello_web");
    }

    #[test]
    fn compiles_macro_generated_code() {
        let py = compile(
            "defmodule M do\n  defmacro double(x) do\n    quote do\n      unquote(x) * 2\n    end\n  end\n  def f(y) do\n    double(y)\n  end\nend",
        );
        assert!(py.contains("return y * 2"), "{py}");
    }

    #[test]
    fn compiles_with_expression() {
        let py = compile(
            "defmodule M do\n  def f(a, b) do\n    with {:ok, x} <- a, {:ok, y} <- b do\n      x + y\n    end\n  end\nend",
        );
        assert!(py.contains("match"), "{py}");
        assert!(py.contains("(\"ok\", x)"), "{py}");
    }

    #[test]
    fn compiles_elixir_integer_division() {
        let py = compile(
            "defmodule M do\n  def f(a, b) do\n    {div(a, b), rem(a, b), a // b}\n  end\nend",
        );
        assert!(py.contains("_gan_div(a, b)"), "{py}");
        assert!(py.contains("_gan_rem(a, b)"), "{py}");
    }
}
