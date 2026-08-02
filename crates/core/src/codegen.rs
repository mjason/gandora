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

/// A parsed `@doc`/`@moduledoc` value (GEP-0007-R001).
#[derive(Debug, Clone, Default)]
pub struct DocInfo {
    /// rendered `@spec` lines for this definition (GEP-0017-R004)
    pub specs: Vec<String>,
    /// locale tag -> markdown text; "default" is the fallback
    pub entries: Vec<(String, String)>,
    /// `@example` blocks: the only channel holding doctests
    /// (GEP-0007-R004)
    pub examples: Vec<String>,
    /// metadata from keyword-form `@doc since: "..."` lines (GEP-0007-R002)
    pub meta: Vec<(String, String)>,
    pub hidden: bool,
}

impl DocInfo {
    pub fn meta_value(&self, key: &str) -> Option<&str> {
        self.meta
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }
}

impl DocInfo {
    pub fn default_text(&self) -> Option<&str> {
        self.entries
            .iter()
            .find(|(tag, _)| tag == "default")
            .map(|(_, text)| text.as_str())
    }
}

/// Merge one `@doc`/`@moduledoc` attribute into a doc map. Elixir
/// semantics: a string sets the text, `false` hides, a keyword list sets
/// metadata; multiple attributes before one definition accumulate
/// (GEP-0007-R001/R001C).
pub fn merge_doc_value(
    file: &str,
    call: &Call,
    info: &mut DocInfo,
    attr: &str,
) -> crate::diag::Result<()> {
    match call.args.first() {
        Some(Term::Bool(false)) => {
            info.hidden = true;
            return Ok(());
        }
        Some(t) if t.as_plain_str().is_some() => {
            if info.default_text().is_some() {
                return Err(Diagnostic::new(
                    file,
                    call.span,
                    format!("{attr} text is given twice for the same definition"),
                ));
            }
            info.entries
                .push(("default".to_string(), t.as_plain_str().unwrap()));
            return Ok(());
        }
        Some(Term::Str(_)) => {
            return Err(Diagnostic::new(
                file,
                call.span,
                "doc text cannot use #{} interpolation; write \\#{ for a literal \
                 #{ (GEP-0007-R001)",
            ))
        }
        Some(Term::Pair(_, _)) | Some(Term::List(_)) => {}
        _ => {
            return Err(Diagnostic::new(
                file,
                call.span,
                format!(
                    "{attr} accepts a Markdown string, false, or metadata pairs \
                     like `{attr} since: \"1.2.0\"` (GEP-0007-R001)"
                ),
            ))
        }
    }
    // keyword form: metadata (Elixir's @doc since: / deprecated:)
    let mut pairs: Vec<&Term> = Vec::new();
    for arg in &call.args {
        match arg {
            Term::List(items) => pairs.extend(items.iter()),
            other => pairs.push(other),
        }
    }
    for p in pairs {
        let Term::Pair(key, value) = p else {
            return Err(Diagnostic::new(
                file,
                call.span,
                format!("{attr} metadata requires key: value pairs"),
            ));
        };
        let rendered = match value.as_ref() {
            t if t.as_plain_str().is_some() => t.as_plain_str().unwrap(),
            Term::Bool(b) => b.to_string(),
            Term::Int(n) => n.to_string(),
            Term::Float(f) => f.to_string(),
            Term::Atom(a) => a.clone(),
            other => {
                return Err(Diagnostic::new(
                    file,
                    call.span,
                    format!("{attr} {key}: value must be a literal, found {other:?}"),
                ))
            }
        };
        if info.meta.iter().any(|(k, _)| k == key) {
            return Err(Diagnostic::new(
                file,
                call.span,
                format!("duplicate {attr} metadata key {key}"),
            ));
        }
        info.meta.push((key.clone(), rendered));
    }
    Ok(())
}

/// Merge `@doc_trans <locale>: "..."` pairs into a doc map
/// (GEP-0007-R001A). May be repeated, one or more locales per line.
pub fn merge_doc_trans(
    file: &str,
    call: &Call,
    info: &mut DocInfo,
    attr: &str,
) -> crate::diag::Result<()> {
    let mut pairs: Vec<&Term> = Vec::new();
    for arg in &call.args {
        match arg {
            Term::List(items) => pairs.extend(items.iter()),
            other => pairs.push(other),
        }
    }
    if pairs.is_empty() {
        return Err(Diagnostic::new(
            file,
            call.span,
            format!("{attr} requires locale: \"text\" pairs"),
        ));
    }
    for p in pairs {
        let Term::Pair(key, value) = p else {
            return Err(Diagnostic::new(
                file,
                call.span,
                format!("{attr} requires locale: \"text\" pairs, e.g. {attr} zh_CN: \"...\""),
            ));
        };
        if key == "default" {
            return Err(Diagnostic::new(
                file,
                call.span,
                format!("the default text belongs in @doc, not {attr}"),
            ));
        }
        let tag = key.replace('_', "-");
        let Some(text) = value.as_plain_str() else {
            return Err(Diagnostic::new(
                file,
                call.span,
                format!("{attr} {key}: value must be a plain string"),
            ));
        };
        if info.entries.iter().any(|(t, _)| *t == tag) {
            return Err(Diagnostic::new(
                file,
                call.span,
                format!("duplicate {attr} locale {tag}"),
            ));
        }
        if text.lines().any(|l| l.trim_start().starts_with("gan> ")) {
            return Err(Diagnostic::new(
                file,
                call.span,
                format!(
                    "translations are prose-only: move gan> examples into an \
                     @example block so they are written and tested exactly once \
                     (GEP-0007-R007)"
                ),
            ));
        }
        info.entries.push((tag, text));
    }
    Ok(())
}

/// Parse an `@example` block: one plain string (GEP-0007-R004).
pub fn example_from_args(file: &str, call: &Call) -> crate::diag::Result<String> {
    match call.args.first() {
        Some(t) if t.as_plain_str().is_some() => Ok(t.as_plain_str().unwrap()),
        _ => Err(Diagnostic::new(
            file,
            call.span,
            "@example requires one plain string (usually a heredoc with gan> lines)",
        )),
    }
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
    spec: Option<Term>,
    doc: Option<DocInfo>,
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
    defaulted_funs: BTreeSet<String>,
    private_funs: BTreeSet<(String, usize)>,
    attr_names: BTreeSet<String>,
    /// python package prefix for this build's own modules (pyPackage)
    pub py_prefix: Option<String>,
    /// sibling module names of the same build
    pub project_modules: BTreeSet<String>,
    /// installed marker names -> dotted python paths (GEP-0006-R005A)
    pub installed_modules: BTreeMap<String, String>,
    struct_fields: Option<Vec<(String, Term)>>,
    /// non-fatal notices surfaced by `gan check`/`gan build`
    pub warnings: Vec<String>,
    /// true after `compile` when the module defines no runtime code
    /// (macros only) and should produce no Python file (GEP-0002-R009).
    pub compile_time_only: bool,
    /// (state_var, result_var) of enclosing `loop`s (GEP-0014)
    loop_stack: Vec<(String, String)>,
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
            defaulted_funs: BTreeSet::new(),
            private_funs: BTreeSet::new(),
            attr_names: BTreeSet::new(),
            py_prefix: None,
            project_modules: BTreeSet::new(),
            installed_modules: BTreeMap::new(),
            struct_fields: None,
            warnings: Vec::new(),
            compile_time_only: false,
            loop_stack: Vec::new(),
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
        if self.module_segs.is_empty() {
            self.module_segs = declared.clone();
        }
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

        let mut moduledoc: Option<DocInfo> = None;
        let mut pending_doc: Option<DocInfo> = None;
        let mut pending_spec: Option<Term> = None;
        let mut pending_decorators: Vec<Term> = Vec::new();
        let mut funs: Vec<FnDef> = Vec::new();
        let mut order: BTreeMap<String, usize> = BTreeMap::new();
        let mut attrs: Vec<(String, Term)> = Vec::new();

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
                    let info = moduledoc.get_or_insert_with(DocInfo::default);
                    merge_doc_value(&self.file, call, info, "@moduledoc")?;
                }
                "@doc" => {
                    let info = pending_doc.get_or_insert_with(DocInfo::default);
                    merge_doc_value(&self.file, call, info, "@doc")?;
                }
                "@example" => {
                    let ex = example_from_args(&self.file, call)?;
                    pending_doc
                        .get_or_insert_with(DocInfo::default)
                        .examples
                        .push(ex);
                }
                "@doc_trans" => {
                    let info = pending_doc.as_mut().ok_or_else(|| {
                        self.err(
                            call.span,
                            "@doc_trans must follow the @doc it translates (GEP-0007-R001A)",
                        )
                    })?;
                    merge_doc_trans(&self.file, call, info, "@doc_trans")?;
                }
                "@moduledoc_trans" => {
                    let info = moduledoc.as_mut().ok_or_else(|| {
                        self.err(
                            call.span,
                            "@moduledoc_trans must follow @moduledoc (GEP-0007-R001A)",
                        )
                    })?;
                    merge_doc_trans(&self.file, call, info, "@moduledoc_trans")?;
                }
                "@spec" => {
                    let value = call.args.first().ok_or_else(|| {
                        self.err(call.span, "@spec requires `name(types) :: type` (GEP-0017-R001)")
                    })?;
                    let ok = matches!(value,
                        Term::Call(c) if matches!(&c.callee, Callee::Name(n) if n == "::"));
                    if !ok {
                        return Err(self.err(
                            call.span,
                            "@spec requires `name(types) :: type` (GEP-0017-R001)",
                        ));
                    }
                    if pending_spec.is_some() {
                        return Err(self.err(
                            call.span,
                            "only one @spec may precede a definition (GEP-0017-R001)",
                        ));
                    }
                    pending_spec = Some(value.clone());
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
                "defstruct" => {
                    if self.struct_fields.is_some() {
                        return Err(self.err(
                            call.span,
                            "a module may declare at most one defstruct (GEP-0004-R001)",
                        ));
                    }
                    self.struct_fields = Some(self.parse_defstruct(call)?);
                }
                "def" | "defp" => {
                    let (fname, raw_params, guard, fbody) = self.parse_def(call)?;
                    // default parameters: `p \\ expr` (GEP-0011-R002)
                    let mut params: Vec<Term> = Vec::new();
                    let mut defaults: Vec<Term> = Vec::new();
                    for p in &raw_params {
                        match p {
                            Term::Call(c)
                                if matches!(&c.callee, Callee::Name(n) if n == "\\\\") =>
                            {
                                params.push(c.args[0].clone());
                                defaults.push(c.args[1].clone());
                            }
                            other => {
                                if !defaults.is_empty() {
                                    return Err(self.err(
                                        call.span,
                                        format!(
                                            "default parameters of {fname} must be \
                                             trailing (GEP-0011-R003)"
                                        ),
                                    ));
                                }
                                params.push(other.clone());
                            }
                        }
                    }
                    let key = fname.clone();
                    if !defaults.is_empty() {
                        if !self.defaulted_funs.insert(fname.clone()) {
                            return Err(self.err(
                                call.span,
                                format!(
                                    "only one definition of {fname} may declare \
                                     defaults (GEP-0011-R003)"
                                ),
                            ));
                        }
                    }
                    for arity in (params.len() - defaults.len())..=params.len() {
                        self.local_funs.insert((fname.clone(), arity));
                        if name == "defp" {
                            self.private_funs.insert((fname.clone(), arity));
                        }
                    }
                    let idx = if let Some(&i) = order.get(&key) {
                        if pending_doc.is_some()
                            || pending_spec.is_some()
                            || !pending_decorators.is_empty()
                        {
                            return Err(self.err(
                                call.span,
                                "@doc/@spec/@decorate must precede the first clause of a function",
                            ));
                        }
                        i
                    } else {
                        funs.push(FnDef {
                            name: fname.clone(),
                            private: name == "defp",
                            spec: pending_spec.take(),
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
                    let arity = params.len();
                    funs[idx].clauses.push((params.clone(), guard, fbody));
                    // each omitted default suffix becomes a delegating clause
                    for j in 1..=defaults.len() {
                        let keep = arity - j;
                        let synth_params: Vec<Term> = params[..keep].to_vec();
                        let mut args: Vec<Term> = synth_params.clone();
                        args.extend(defaults[defaults.len() - j..].iter().cloned());
                        let body = Term::Call(Box::new(Call {
                            callee: Callee::Name(fname.clone()),
                            args,
                            span: call.span,
                        }));
                        funs[idx].clauses.push((synth_params, None, body));
                    }
                }
                other if other.starts_with('@') => {
                    // a module attribute declaration: `@name expr` (GEP-0004-R009)
                    let attr = other.trim_start_matches('@').to_string();
                    if call.args.len() != 1 {
                        return Err(self.err(
                            call.span,
                            format!("@{attr} requires exactly one initializer expression"),
                        ));
                    }
                    if !self.attr_names.insert(attr.clone()) {
                        return Err(self.err(
                            call.span,
                            format!("module attribute @{attr} is declared twice (GEP-0004-R010)"),
                        ));
                    }
                    attrs.push((attr, call.args[0].clone()));
                }
                other => {
                    return Err(self.err(
                        call.span,
                        format!(
                            "'{other}' is not supported at module level \
                             (supported: def, defp, defmacro, defstruct, alias, import, \
                             require, pyimport, @doc, @moduledoc, @decorate, @<attribute>)"
                        ),
                    ))
                }
            }
        }

        // a module of nothing but macros has no runtime existence
        if funs.is_empty()
            && attrs.is_empty()
            && self.struct_fields.is_none()
            && self.star_imports.is_empty()
        {
            self.compile_time_only = true;
            return Ok(String::new());
        }

        for (attr, _) in &attrs {
            if self.local_funs.iter().any(|(n, _)| n == attr) {
                return Err(self.err(
                    Span::default(),
                    format!("module attribute @{attr} collides with a function name"),
                ));
            }
        }

        // struct class (GEP-0004-R002)
        let struct_code = match self.struct_fields.clone() {
            Some(fields) => Some(self.emit_struct_class(&fields)?),
            None => None,
        };

        // module attribute assignments, in source order (GEP-0004-R009)
        let mut attr_code = String::new();
        for (attr, init) in &attrs {
            let mut pre = Vec::new();
            let e = self.emit_expr(init, &mut pre)?;
            attr_code.push('\n');
            for line in pre {
                attr_code.push_str(&line);
                attr_code.push('\n');
            }
            attr_code.push_str(&format!("{} = {e}\n", map_ident(attr)));
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
        let module_docstring = self.docstring_text(moduledoc.as_ref(), Span::default())?;
        if let Some(doc) = &module_docstring {
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
        if let Some(sc) = struct_code {
            out.push('\n');
            out.push_str(&sc);
        }
        out.push_str(&attr_code);
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

    /// Field list of a `defstruct`: keyword pairs, atoms, or a list of both
    /// (GEP-0004-R001).
    fn parse_defstruct(&mut self, call: &Call) -> Result<Vec<(String, Term)>> {
        let mut fields = Vec::new();
        let mut items: Vec<&Term> = Vec::new();
        for arg in &call.args {
            match arg {
                Term::List(inner) => items.extend(inner.iter()),
                other => items.push(other),
            }
        }
        for item in items {
            match item {
                Term::Pair(k, v) => fields.push((k.clone(), (**v).clone())),
                Term::Atom(a) => fields.push((a.clone(), Term::Nil)),
                other => {
                    return Err(self.err(
                        call.span,
                        format!(
                            "defstruct fields must be `name: default` pairs or atoms, \
                             found {other:?}"
                        ),
                    ))
                }
            }
        }
        if fields.is_empty() {
            return Err(self.err(call.span, "defstruct requires at least one field"));
        }
        Ok(fields)
    }

    fn struct_class_name(&self) -> String {
        self.module_segs
            .last()
            .cloned()
            .unwrap_or_else(|| "Struct".to_string())
    }

    fn emit_struct_class(&mut self, fields: &[(String, Term)]) -> Result<String> {
        self.py_imports.insert("dataclasses".into());
        let mut out = String::new();
        out.push_str("\n@dataclasses.dataclass(frozen=True)\n");
        out.push_str(&format!("class {}:\n", self.struct_class_name()));
        for (name, default) in fields {
            let mut pre = Vec::new();
            let e = self.emit_expr(default, &mut pre)?;
            if !pre.is_empty() {
                return Err(self.err(
                    default.span(),
                    "struct field defaults must be simple expressions",
                ));
            }
            // mutable literal defaults become per-instance factories
            let rhs = match default {
                Term::List(_) | Term::Map(_) | Term::Tuple(_) => {
                    format!("dataclasses.field(default_factory=lambda: {e})")
                }
                _ => e,
            };
            out.push_str(&format!("    {}: object = {rhs}\n", map_ident(name)));
        }
        Ok(out)
    }

    /// The Python reference for the struct class of `segs` (imports as needed).
    fn struct_ref(&mut self, segs: &[String]) -> String {
        let resolved = self.resolve_alias(segs);
        if resolved == self.module_segs {
            return self.struct_class_name();
        }
        let path = self.gan_module_import(&resolved);
        format!("{path}.{}", resolved.last().unwrap())
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
                // `def unquote(name)(params)` after expansion: the callee is
                // an Apply of the computed name (GEP-0008-R002)
                Callee::Apply(inner) => match inner.as_ref() {
                    Term::Atom(s) => (s.clone(), c.args.clone()),
                    t if t.as_plain_str().is_some() => {
                        (t.as_plain_str().unwrap(), c.args.clone())
                    }
                    Term::Var(s, _) => (s.clone(), c.args.clone()),
                    other => {
                        return Err(self.err(
                            c.span,
                            format!(
                                "a computed def name must be an atom or string, \
                                 found {other:?} (GEP-0008-R002)"
                            ),
                        ))
                    }
                },
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


    /// A GEP-0017-R002 type expression rendered as a Python hint.
    fn spec_hint(&mut self, t: &Term) -> Result<String> {
        match t {
            Term::Nil => Ok("None".to_string()),
            Term::Call(c) => match &c.callee {
                Callee::Name(n) if n == "|" => {
                    let a = self.spec_hint(&c.args[0])?;
                    let b = self.spec_hint(&c.args[1])?;
                    Ok(format!("{a} | {b}"))
                }
                Callee::Name(n) => match (n.as_str(), c.args.len()) {
                    ("integer", 0) => Ok("int".into()),
                    ("float", 0) => Ok("float".into()),
                    ("number", 0) => Ok("int | float".into()),
                    ("boolean", 0) => Ok("bool".into()),
                    ("string", 0) => Ok("str".into()),
                    ("atom", 0) => Ok("str".into()),
                    ("any", 0) => Ok("object".into()),
                    ("list", 0) => Ok("list".into()),
                    ("list", 1) => Ok(format!("list[{}]", self.spec_hint(&c.args[0])?)),
                    ("map", 0) => Ok("dict".into()),
                    ("map", 2) => Ok(format!(
                        "dict[{}, {}]",
                        self.spec_hint(&c.args[0])?,
                        self.spec_hint(&c.args[1])?
                    )),
                    ("tuple", 0) => Ok("tuple".into()),
                    ("tuple", _) => {
                        let parts: Vec<String> = c
                            .args
                            .iter()
                            .map(|a| self.spec_hint(a))
                            .collect::<Result<_>>()?;
                        Ok(format!("tuple[{}]", parts.join(", ")))
                    }
                    ("fun", 0) => {
                        self.py_imports.insert("collections.abc".into());
                        Ok("collections.abc.Callable".into())
                    }
                    _ => Err(self.err(
                        c.span,
                        format!("'{n}' is not a type (GEP-0017-R002)"),
                    )),
                },
                Callee::Dot { base, name, .. } => match base.as_ref() {
                    // $mod.Type — the host's own types at the boundary
                    Term::PyRef(m) => {
                        self.py_imports.insert(m.clone());
                        Ok(format!("{m}.{name}"))
                    }
                    // Mod.t() — the struct class generated for Mod (GEP-0004)
                    Term::Alias(segs) if name == "t" => {
                        let segs = segs.clone();
                        Ok(self.struct_ref(&segs))
                    }
                    _ => Err(self.err(
                        c.span,
                        "types are built-ins, $mod.Type, or Mod.t() (GEP-0017-R002)",
                    )),
                },
                _ => Err(self.err(
                    c.span,
                    "types are built-ins, $mod.Type, or Mod.t() (GEP-0017-R002)",
                )),
            },
            other => Err(self.err(
                other.span(),
                "types are built-ins, $mod.Type, or Mod.t() (GEP-0017-R002)",
            )),
        }
    }

    /// Split a stored `@spec` term into (declared name, arg types, return).
    fn spec_parts<'a>(&self, spec: &'a Term) -> Result<(String, Vec<&'a Term>, &'a Term)> {
        let Term::Call(sc) = spec else { unreachable!() };
        let head = &sc.args[0];
        let ret = &sc.args[1];
        match head {
            Term::Call(hc) => match &hc.callee {
                Callee::Name(n) => Ok((n.clone(), hc.args.iter().collect(), ret)),
                _ => Err(self.err(
                    sc.span,
                    "@spec head must be `name(types)` (GEP-0017-R001)",
                )),
            },
            _ => Err(self.err(
                sc.span,
                "@spec head must be `name(types)` (GEP-0017-R001)",
            )),
        }
    }

    fn compile_fun(&mut self, f: &FnDef) -> Result<String> {
        let mut arities: Vec<usize> = f.clauses.iter().map(|(p, _, _)| p.len()).collect();
        arities.sort_unstable();
        arities.dedup();
        let arity_label = arities
            .iter()
            .map(|a| a.to_string())
            .collect::<Vec<_>>()
            .join(",");
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
        let spec_info = match &f.spec {
            None => None,
            Some(spec) => {
                let (declared, arg_types, ret) = self.spec_parts(spec)?;
                if declared != f.name {
                    return Err(self.err(
                        f.span,
                        format!(
                            "@spec names {declared} but the definition is {} (GEP-0017-R001)",
                            f.name
                        ),
                    ));
                }
                if !arities.contains(&arg_types.len()) {
                    return Err(self.err(
                        f.span,
                        format!(
                            "@spec for {}/{} matches no clause (arities: {arity_label}) \
                             (GEP-0017-R001)",
                            f.name,
                            arg_types.len()
                        ),
                    ));
                }
                let hints: Vec<String> = arg_types
                    .iter()
                    .map(|t| self.spec_hint(t))
                    .collect::<Result<_>>()?;
                let ret_hint = self.spec_hint(ret)?;
                Some((hints, ret_hint))
            }
        };
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
            match &spec_info {
                Some((hints, ret)) if hints.len() == names.len() => {
                    let typed: Vec<String> = names
                        .iter()
                        .zip(hints)
                        .map(|(n, h)| format!("{n}: {h}"))
                        .collect();
                    lines.push(format!("def {py_name}({}) -> {ret}:", typed.join(", ")));
                }
                Some((_, ret)) => {
                    lines.push(format!("def {py_name}({}) -> {ret}:", names.join(", ")));
                }
                None => lines.push(format!("def {py_name}({}):", names.join(", "))),
            }
            let mut body_lines = Vec::new();
            if let Some(compiled) = self.docstring_text(f.doc.as_ref(), f.span)? {
                body_lines.push(format!(
                    "\"\"\"{}\"\"\"",
                    compiled.replace("\"\"\"", "\\\"\\\"\\\"")
                ));
            }
            self.emit_stmt_block(body, Dest::Return, &mut body_lines)?;
            push_indented(&mut lines, &body_lines);
        } else {
            match &spec_info {
                Some((_, ret)) => {
                    lines.push(format!("def {py_name}(*_gan_args) -> {ret}:"))
                }
                None => lines.push(format!("def {py_name}(*_gan_args):")),
            }
            let mut body_lines = Vec::new();
            if let Some(compiled) = self.docstring_text(f.doc.as_ref(), f.span)? {
                body_lines.push(format!(
                    "\"\"\"{}\"\"\"",
                    compiled.replace("\"\"\"", "\\\"\\\"\\\"")
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
                "raise GanMatchError(\"no clause of {n}/{arity_label} matched \" + repr(_gan_args))"
            ));
            push_indented(&mut lines, &body_lines);
        }
        Ok(format!("\n{}\n", lines.join("\n")))
    }

    /// Assemble the docstring: opaque doc text, compiled example blocks,
    /// then the well-known metadata trailer (GEP-0007-R005).
    fn docstring_text(
        &mut self,
        info: Option<&DocInfo>,
        span: Span,
    ) -> Result<Option<String>> {
        let Some(info) = info else { return Ok(None) };
        if info.hidden {
            return Ok(None);
        }
        let mut parts: Vec<String> = Vec::new();
        if let Some(text) = info.default_text() {
            if text.lines().any(|l| l.trim_start().starts_with("gan> ")) {
                self.warnings.push(format!(
                    "{}:{}: doc text contains a gan> line; it will not be \
                     tested — move examples into @example (GEP-0007-R005)",
                    self.file, span.line
                ));
            }
            parts.push(text.trim_end().to_string());
        }
        for ex in &info.examples {
            parts.push(self.compile_doctests(ex.trim_end(), span)?);
        }
        if let Some(v) = info.meta_value("deprecated") {
            parts.push(format!("Deprecated: {v}"));
        }
        if let Some(v) = info.meta_value("since") {
            parts.push(format!("Since: {v}"));
        }
        if parts.is_empty() {
            return Ok(None);
        }
        let mut joined = parts.join("\n\n");
        if joined.contains('\n') {
            joined.push('\n');
        }
        Ok(Some(joined))
    }

    /// Compile `gan> expr` doctest lines into native Python doctests
    /// (GEP-0007-R006).
    fn compile_doctests(&mut self, text: &str, span: Span) -> Result<String> {
        if !text.contains("gan> ") {
            return Ok(text.to_string());
        }
        let mut out = Vec::new();
        for line in text.lines() {
            let trimmed = line.trim_start();
            if let Some(expr_src) = trimmed.strip_prefix("gan> ") {
                let indent = &line[..line.len() - trimmed.len()];
                let term =
                    crate::parser::parse_expr_str(&self.file, expr_src).map_err(|mut d| {
                        d.span = span;
                        d.message = format!("in doctest `{expr_src}`: {}", d.message);
                        d
                    })?;
                let mut pre = Vec::new();
                let e = self.emit_expr(&term, &mut pre)?;
                if !pre.is_empty() {
                    return Err(self.err(
                        span,
                        format!(
                            "doctest `{expr_src}` needs statements; doctest expressions \
                             must be single simple expressions (GEP-0007-R007)"
                        ),
                    ));
                }
                out.push(format!("{indent}>>> {e}"));
            } else {
                out.push(line.to_string());
            }
        }
        Ok(out.join("\n"))
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
                        "try" => return self.emit_try(call, dest, out),
                        "loop" => return self.emit_loop(call, dest, out),
                        "recur" | "break" => {
                            let Some((state, result)) = self.loop_stack.last().cloned() else {
                                return Err(self.err(
                                    call.span,
                                    format!(
                                        "{name} is only valid inside a loop body \
                                         (GEP-0014-R005)"
                                    ),
                                ));
                            };
                            if call.args.len() != 1 {
                                return Err(self.err(
                                    call.span,
                                    format!("{name} takes exactly one argument"),
                                ));
                            }
                            let mut pre = Vec::new();
                            let e = self.emit_expr(&call.args[0], &mut pre)?;
                            out.extend(pre);
                            if name == "recur" {
                                out.push(format!("{state} = {e}"));
                                out.push("continue".into());
                            } else {
                                out.push(format!("{result} = {e}"));
                                out.push("break".into());
                            }
                            return Ok(());
                        }
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

    /// try/rescue/after (GEP-0014-R001..R003).
    fn emit_try(&mut self, call: &Call, dest: Dest, out: &mut Vec<String>) -> Result<()> {
        let body = Term::keyword_arg(&call.args, "do")
            .ok_or_else(|| self.err(call.span, "try requires a do block"))?
            .clone();
        let rescue = Term::keyword_arg(&call.args, "rescue").cloned();
        let after = Term::keyword_arg(&call.args, "after").cloned();
        if rescue.is_none() && after.is_none() {
            return Err(self.err(
                call.span,
                "try requires rescue and/or after (GEP-0014-R003)",
            ));
        }
        out.push("try:".into());
        let mut body_lines = Vec::new();
        self.emit_stmt_block(&body, dest, &mut body_lines)?;
        if body_lines.is_empty() {
            body_lines.push("pass".into());
        }
        push_indented(out, &body_lines);
        if let Some(rescue) = rescue {
            let clauses = self.clause_list(&rescue, call.span)?;
            let clauses: Vec<Term> = clauses.into_iter().cloned().collect();
            for clause in &clauses {
                let Term::Call(c) = clause else { continue };
                let Term::List(pats) = &c.args[0] else { continue };
                let (var, ty) = match &pats[0] {
                    Term::Var(n, ctx) => (hygienic_name(n, *ctx), None),
                    Term::Call(inc)
                        if matches!(&inc.callee, Callee::Name(n) if n == "in")
                            && inc.args.len() == 2 =>
                    {
                        let Term::Var(n, ctx) = &inc.args[0] else {
                            return Err(self.err(
                                inc.span,
                                "rescue patterns are `e` or `e in Type` (GEP-0014-R002)",
                            ));
                        };
                        let mut pre = Vec::new();
                        let te = self.emit_expr(&inc.args[1], &mut pre)?;
                        if !pre.is_empty() {
                            return Err(self.err(
                                inc.span,
                                "rescue types must be simple expressions",
                            ));
                        }
                        (hygienic_name(n, *ctx), Some(te))
                    }
                    other => {
                        return Err(self.err(
                            c.span,
                            format!(
                                "rescue patterns are `e` or `e in Type`, found {other:?} \
                                 (GEP-0014-R002)"
                            ),
                        ))
                    }
                };
                let ty = ty.unwrap_or_else(|| "Exception".to_string());
                out.push(format!("except {ty} as {var}:"));
                let mut arm = Vec::new();
                self.emit_stmt_block(&c.args[1], dest, &mut arm)?;
                if arm.is_empty() {
                    arm.push("pass".into());
                }
                push_indented(out, &arm);
            }
        }
        if let Some(after) = after {
            out.push("finally:".into());
            let mut fin = Vec::new();
            self.emit_stmt_block(&after, Dest::Ignore, &mut fin)?;
            if fin.is_empty() {
                fin.push("pass".into());
            }
            push_indented(out, &fin);
        }
        Ok(())
    }

    /// loop/recur/break (GEP-0014-R004..R006).
    fn emit_loop(&mut self, call: &Call, dest: Dest, out: &mut Vec<String>) -> Result<()> {
        let head = call
            .args
            .first()
            .ok_or_else(|| self.err(call.span, "loop requires `pattern = initial`"))?;
        let (pat, init) = match head {
            Term::Call(c) if matches!(&c.callee, Callee::Name(n) if n == "=") => {
                (c.args[0].clone(), c.args[1].clone())
            }
            other => {
                return Err(self.err(
                    call.span,
                    format!("loop requires `pattern = initial`, found {other:?}"),
                ))
            }
        };
        let body = Term::keyword_arg(&call.args, "do")
            .ok_or_else(|| self.err(call.span, "loop requires a do block"))?
            .clone();
        let s = self.fresh_tmp("loop");
        let state = self.tmp(s).to_string();
        let r = self.fresh_tmp("res");
        let result = self.tmp(r).to_string();
        let mut pre = Vec::new();
        let init_e = self.emit_expr(&init, &mut pre)?;
        out.extend(pre);
        out.push(format!("{state} = {init_e}"));
        out.push(format!("{result} = None"));
        out.push("while True:".into());
        let mut inner: Vec<String> = Vec::new();
        // rebind the state pattern each iteration (GEP-0014-R005)
        match &pat {
            Term::Var(n, ctx) if n != "_" => {
                inner.push(format!("{} = {state}", hygienic_name(n, *ctx)));
            }
            _ => {
                let mut guards = Vec::new();
                let p = self.compile_pattern(&pat, &mut guards)?;
                self.helpers.insert("match_error");
                inner.push(format!("match {state}:"));
                let case_line = if guards.is_empty() {
                    format!("case {p}:")
                } else {
                    format!("case {p} if {}:", guards.join(" and "))
                };
                push_indented(&mut inner, &[case_line, "    pass".into()]);
                push_indented(
                    &mut inner,
                    &[
                        "case _:".into(),
                        format!(
                            "    raise GanMatchError(\"loop state did not match: \" + repr({state}))"
                        ),
                    ],
                );
            }
        }
        self.loop_stack.push((state.clone(), result.clone()));
        let body_result = (|| -> Result<()> {
            let stmts = body.as_block();
            for (i, stmt) in stmts.iter().enumerate() {
                if i + 1 == stmts.len() {
                    let d = Dest::Assign(r);
                    self.emit_stmt(stmt, d, &mut inner)?;
                } else {
                    self.emit_stmt(stmt, Dest::Ignore, &mut inner)?;
                }
            }
            Ok(())
        })();
        self.loop_stack.pop();
        body_result?;
        inner.push("break".into());
        push_indented(out, &inner);
        self.finish_value(result, dest, out);
        Ok(())
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
                    "==" | "!=" | "<" | ">" | "<=" | ">=" | "not" | "and" | "or" | "in"
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
            Term::PyRef(m) => {
                // first-class module reference (GEP-0003-R002)
                self.py_imports.insert(m.clone());
                Ok(m.clone())
            }
            Term::Str(parts) => self.emit_string(parts, pre),
            Term::Var(name, ctx) => Ok(hygienic_name(name, *ctx)),
            Term::Alias(segs) => {
                // a bare module reference
                let resolved = self.resolve_alias(segs);
                Ok(self.gan_module_import(&resolved))
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
                    // f-string expressions may not contain quotes matching the
                    // delimiter, backslashes, or `#` before Python 3.12; hoist
                    // such expressions so the output runs on targetPython 3.11
                    let ee = if ee.contains(['"', '\\', '#', '\n']) {
                        let idx = self.fresh_tmp("fstr");
                        let name = self.tmp(idx).to_string();
                        pre.push(format!("{name} = {ee}"));
                        name
                    } else {
                        ee
                    };
                    let _ = write!(body, "{{{ee}}}");
                }
            }
        }
        Ok(format!("f\"{body}\""))
    }

    /// The Python import path for a resolved Gandora module reference.
    /// Precedence: sibling project modules (with the project's pyPackage
    /// prefix), installed-package marker names, then the mechanical
    /// GEP-0001-R014 mapping (GEP-0006-R005A).
    fn gan_module_import(&mut self, resolved: &[String]) -> String {
        let joined = resolved.join(".");
        let path = if self.project_modules.contains(&joined) {
            match &self.py_prefix {
                Some(prefix) => format!("{prefix}.{}", module_py_path(resolved)),
                None => module_py_path(resolved),
            }
        } else if let Some(installed) = self.installed_modules.get(&joined) {
            installed.clone()
        } else {
            module_py_path(resolved)
        };
        self.gan_imports.insert(path.clone());
        path
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
                    // $module.fun(...) — remote reference (GEP-0003-R001/R002)
                    Term::PyRef(module) => {
                        self.py_imports.insert(module.clone());
                        let f = map_ident(name);
                        if *is_call {
                            let args = self.emit_args(&call.args, pre)?;
                            Ok(format!("{module}.{f}({args})"))
                        } else {
                            Ok(format!("{module}.{f}"))
                        }
                    }
                    // revision-1 spelling: atoms are data now (GEP-0003-R009)
                    Term::Atom(module) => Err(self.err(
                        span,
                        format!(
                            "atoms are data and no longer name Python modules; \
                             write ${module}.{name} (GEP-0003-R009)"
                        ),
                    )),
                    // Mod.fun(...) — Gandora cross-module call (GEP-0001-R017)
                    Term::Alias(segs) => {
                        let resolved = self.resolve_alias(segs);
                        if resolved == vec!["IO".to_string()] {
                            return self.emit_io_call(name, call, pre);
                        }
                        // a qualified reference to the current module is a
                        // local call: no self-import
                        if resolved == self.module_segs {
                            let mut f = map_ident(name);
                            if self
                                .private_funs
                                .contains(&(name.clone(), call.args.len()))
                            {
                                f.insert(0, '_');
                            }
                            if *is_call {
                                let args = self.emit_args(&call.args, pre)?;
                                return Ok(format!("{f}({args})"));
                            }
                            return Ok(f);
                        }
                        let path = self.gan_module_import(&resolved);
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
                        let needs_paren = match other {
                            Term::Int(_) | Term::Float(_) => true,
                            // binary operations bind looser than attribute access
                            Term::Call(c) => matches!(&c.callee, Callee::Name(n)
                                if c.args.len() == 2
                                    && matches!(n.as_str(),
                                        "+" | "-" | "*" | "/" | "//" | "++" | "<>" | ".."
                                        | "==" | "!=" | "<" | ">" | "<=" | ">=" | "and"
                                        | "or" | "in")),
                            _ => false,
                        };
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
        if name == "::" {
            return Err(self.err(
                call.span,
                "'::' is only valid inside @spec (GEP-0017-R001)",
            ));
        }
        if name == "|" && call.args.len() == 2 {
            return Err(self.err(
                call.span,
                "'|' outside patterns is the @spec union operator (GEP-0017-R001)",
            ));
        }
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
            ("in", 2) => {
                let a = self.emit_operand(&args[0], "in", pre)?;
                let b = self.emit_operand(&args[1], "in", pre)?;
                Ok(format!("{a} in {b}"))
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
            ("-", 1) => match &args[0] {
                Term::Int(n) => Ok(format!("-{n}")),
                Term::Float(v) => {
                    let e = self.emit_expr(&args[0], pre)?;
                    let _ = v;
                    Ok(format!("-{e}"))
                }
                other => {
                    let e = self.emit_expr(other, pre)?;
                    Ok(format!("-({e})"))
                }
            },
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
            ("~w", 1) => {
                // word list (GEP-0005-R004)
                let Term::Str(parts) = &args[0] else {
                    return Err(self.err(call.span, "~w requires a sigil body"));
                };
                if let Some(text) = args[0].as_plain_str() {
                    let words: Vec<String> =
                        text.split_whitespace().map(|w| py_str_lit(w)).collect();
                    Ok(format!("[{}]", words.join(", ")))
                } else {
                    let s = self.emit_string(&parts.clone(), pre)?;
                    Ok(format!("{s}.split()"))
                }
            }
            ("~s", 1) => {
                let Term::Str(parts) = &args[0] else {
                    return Err(self.err(call.span, "~s requires a sigil body"));
                };
                self.emit_string(&parts.clone(), pre)
            }
            ("~r", 1) => {
                // compiled Python regex (GEP-0005-R006)
                let Term::Str(parts) = &args[0] else {
                    return Err(self.err(call.span, "~r requires a sigil body"));
                };
                self.py_imports.insert("re".into());
                let s = self.emit_string(&parts.clone(), pre)?;
                Ok(format!("re.compile({s})"))
            }
            ("~python", 1) => {
                // embedded Python: verbatim code with <%= %> code splices
                // (GEP-0005-R007, GEP-0009-R003)
                let body = args[0].as_plain_str().ok_or_else(|| {
                    self.err(call.span, "~python bodies are raw and cannot interpolate")
                })?;
                let mut out = String::new();
                for part in split_splices(&body) {
                    match part {
                        SplicePart::Text(t) => out.push_str(&t),
                        SplicePart::Expr(src) => {
                            let term = crate::parser::parse_expr_str(&self.file, &src)
                                .map_err(|mut d| {
                                    d.span = call.span;
                                    d.message =
                                        format!("in ~python splice: {}", d.message);
                                    d
                                })?;
                            let mut pre = Vec::new();
                            let e = self.emit_expr(&term, &mut pre)?;
                            if !pre.is_empty() {
                                return Err(self.err(
                                    call.span,
                                    "splices must be single simple expressions \
                                     (GEP-0009-R002)",
                                ));
                            }
                            out.push_str(&format!("({e})"));
                        }
                    }
                }
                let body = out.trim().to_string();
                if body.is_empty() {
                    return Err(self.err(call.span, "~python requires a Python expression"));
                }
                Ok(format!("({body})"))
            }
            (sigil, 1) if sigil.starts_with('~') => {
                // embedded-language sigil: a string with value splices
                // (GEP-0009-R001/R004)
                let body = args[0].as_plain_str().ok_or_else(|| {
                    self.err(call.span, "embedded sigil bodies are raw")
                })?;
                let parts = split_splices(&body);
                if parts.iter().all(|p| matches!(p, SplicePart::Text(_))) {
                    let text: String = parts
                        .iter()
                        .map(|p| match p {
                            SplicePart::Text(s) => s.as_str(),
                            SplicePart::Expr(_) => unreachable!(),
                        })
                        .collect();
                    return Ok(py_str_lit(&text));
                }
                let mut out = String::from("f\"");
                for part in parts {
                    match part {
                        SplicePart::Text(t) => out.push_str(&escape_py_str(&t, true)),
                        SplicePart::Expr(src) => {
                            let term = crate::parser::parse_expr_str(&self.file, &src)
                                .map_err(|mut d| {
                                    d.span = call.span;
                                    d.message =
                                        format!("in {sigil} splice: {}", d.message);
                                    d
                                })?;
                            let e = self.emit_expr(&term, pre)?;
                            // same targetPython 3.11 restriction as emit_string
                            let e = if e.contains(['"', '\\', '#', '\n']) {
                                let idx = self.fresh_tmp("fstr");
                                let name = self.tmp(idx).to_string();
                                pre.push(format!("{name} = {e}"));
                                name
                            } else {
                                e
                            };
                            let _ = write!(out, "{{{e}}}");
                        }
                    }
                }
                out.push('"');
                Ok(out)
            }
            ("%struct%", 2) => {
                let Term::Alias(segs) = &args[0] else {
                    return Err(self.err(call.span, "struct literals need a module name"));
                };
                let class = self.struct_ref(&segs.clone());
                let kwargs = self.emit_struct_kwargs(&args[1], pre)?;
                Ok(format!("{class}({kwargs})"))
            }
            ("%struct_update%", 3) => {
                let Term::Alias(segs) = &args[0] else {
                    return Err(self.err(call.span, "struct updates need a module name"));
                };
                let _class = self.struct_ref(&segs.clone());
                self.py_imports.insert("dataclasses".into());
                let base = self.emit_expr(&args[1], pre)?;
                let kwargs = self.emit_struct_kwargs(&args[2], pre)?;
                Ok(format!("dataclasses.replace({base}, {kwargs})"))
            }
            ("%map_update%", 2) => {
                let base = self.emit_operand(&args[0], "+", pre)?;
                let Term::List(pairs) = &args[1] else {
                    return Err(self.err(call.span, "map updates need field: value pairs"));
                };
                let mut parts = vec![format!("**{base}")];
                for p in pairs.clone() {
                    let Term::Pair(k, v) = p else { continue };
                    let ve = self.emit_expr(&v, pre)?;
                    parts.push(format!("{}: {ve}", py_str_lit(&k)));
                }
                Ok(format!("{{{}}}", parts.join(", ")))
            }
            (attr, 0) if attr.starts_with('@') => {
                let name = attr.trim_start_matches('@');
                if self.attr_names.contains(name) {
                    Ok(map_ident(name))
                } else {
                    Err(self.err(
                        call.span,
                        format!("undefined module attribute @{name} (GEP-0004-R011)"),
                    ))
                }
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
            ("recur", 1) | ("break", 1) => Err(self.err(
                call.span,
                "recur/break are statements; use them directly in a loop body \
                 (GEP-0014-R005)",
            )),
            ("if", _) | ("unless", _) | ("case", _) | ("cond", _) | ("with", _) | ("try", _)
            | ("loop", _) => {
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
                    "defstruct", "defprotocol", "defimpl", "receive", "for", "send",
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

    fn emit_struct_kwargs(&mut self, pairs: &Term, pre: &mut Vec<String>) -> Result<String> {
        let Term::List(items) = pairs else {
            return Err(self.err(pairs.span(), "expected field: value pairs"));
        };
        let mut parts = Vec::new();
        for p in items.clone() {
            let Term::Pair(k, v) = p else { continue };
            let ve = self.emit_expr(&v, pre)?;
            parts.push(format!("{}={ve}", map_ident(&k)));
        }
        Ok(parts.join(", "))
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
            spec: None,
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
                // struct pattern: %Mod{field: pat} (GEP-0004-R007)
                if matches!(&c.callee, Callee::Name(n) if n == "%struct%") {
                    let Term::Alias(segs) = &c.args[0] else {
                        return Err(self.err(c.span, "struct patterns need a module name"));
                    };
                    let class = self.struct_ref(&segs.clone());
                    let Term::List(pairs) = &c.args[1] else {
                        return Err(self.err(c.span, "struct patterns need field: pattern pairs"));
                    };
                    let mut parts = Vec::new();
                    for p in pairs.clone() {
                        let Term::Pair(k, v) = p else { continue };
                        let vp = self.compile_pattern(&v, guards)?;
                        parts.push(format!("{}={vp}", map_ident(&k)));
                    }
                    return Ok(format!("{class}({})", parts.join(", ")));
                }
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
            Term::PyRef(_) => Err(self.err(
                Span::default(),
                "module references are not valid patterns",
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

enum SplicePart {
    Text(String),
    Expr(String),
}

/// Split an embedded body on `<%= expr %>` markers; `<%%=` escapes a
/// literal `<%=` (GEP-0009-R002).
fn split_splices(body: &str) -> Vec<SplicePart> {
    let mut parts = Vec::new();
    let mut text = String::new();
    let mut rest = body;
    loop {
        match rest.find("<%") {
            None => break,
            Some(i) => {
                text.push_str(&rest[..i]);
                let after = &rest[i + 2..];
                if let Some(stripped) = after.strip_prefix("%=") {
                    text.push_str("<%=");
                    rest = stripped;
                } else if let Some(after_eq) = after.strip_prefix('=') {
                    match after_eq.find("%>") {
                        Some(j) => {
                            if !text.is_empty() {
                                parts.push(SplicePart::Text(std::mem::take(&mut text)));
                            }
                            parts.push(SplicePart::Expr(after_eq[..j].trim().to_string()));
                            rest = &after_eq[j + 2..];
                        }
                        None => {
                            text.push_str("<%=");
                            rest = after_eq;
                        }
                    }
                } else {
                    text.push_str("<%");
                    rest = after;
                }
            }
        }
    }
    text.push_str(rest);
    if !text.is_empty() || parts.is_empty() {
        parts.push(SplicePart::Text(text));
    }
    parts
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
pub mod tests_helpers {
    use super::*;
    use crate::expander::{collect_macros, Expander};
    use crate::parser::parse_file;

    pub fn compile(src: &str) -> String {
        let module = parse_file("<test>", src).unwrap();
        let macros = collect_macros("<test>", &module).unwrap();
        let mut ex = Expander::new("<test>", macros);
        let expanded = ex.expand_module(&module).unwrap();
        let mut cg = Codegen::new("<test>", vec![]);
        cg.compile(&expanded).unwrap()
    }

    pub fn compile_err(src: &str) -> String {
        let module = parse_file("<test>", src).unwrap();
        let macros = collect_macros("<test>", &module).unwrap();
        let mut ex = Expander::new("<test>", macros);
        let expanded = ex.expand_module(&module).unwrap();
        let mut cg = Codegen::new("<test>", vec![]);
        cg.compile(&expanded).unwrap_err().message
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
    fn specs_compile_to_annotations() {
        // GEP-0017-R002/R003
        let py = compile(
            "defmodule M do\n  @spec add(integer(), number()) :: float()\n  def add(a, b), do: a + b\n\n  @spec pick(list(string()) | nil) :: string()\n  def pick([h | _]), do: h\n  def pick(nil), do: \"\"\nend",
        );
        assert!(py.contains("def add(a: int, b: int | float) -> float:"), "{py}");
        assert!(py.contains("def pick(*_gan_args) -> str:"), "{py}");
    }

    #[test]
    fn spec_interop_and_struct_types() {
        let py = compile(
            "defmodule M do\n  @spec load(string()) :: $decimal.Decimal | map()\n  def load(p), do: p\nend",
        );
        assert!(py.contains("def load(p: str) -> decimal.Decimal | dict:"), "{py}");
        assert!(py.contains("import decimal"), "{py}");
    }

    #[test]
    fn spec_arity_mismatch_is_an_error() {
        let err = compile_err(
            "defmodule M do\n  @spec f(integer()) :: integer()\n  def f(a, b), do: a + b\nend",
        );
        assert!(err.contains("GEP-0017-R001"), "{err}");
    }

    #[test]
    fn stray_spec_operators_are_errors() {
        let err = compile_err(
            "defmodule M do\n  def f(x) do\n    x :: integer()\n  end\nend",
        );
        assert!(err.contains("GEP-0017-R001"), "{err}");
        let err2 = compile_err("defmodule M do\n  def f(x) do\n    x | 1\n  end\nend");
        assert!(err2.contains("GEP-0017-R001"), "{err2}");
    }

    #[test]
    fn compiles_remote_atom_call() {
        let py = compile(
            "defmodule M do\n  def f(x) do\n    $math.sqrt(x)\n  end\nend",
        );
        assert!(py.contains("import math"), "{py}");
        assert!(py.contains("return math.sqrt(x)"), "{py}");
    }

    #[test]
    fn bare_module_reference_is_first_class() {
        // $math alone is the module object (GEP-0003-R002)
        let py = compile(
            "defmodule M do\n  def f() do\n    m = $math\n    m.sqrt(4.0)\n  end\nend",
        );
        assert!(py.contains("import math"), "{py}");
        assert!(py.contains("m = math"), "{py}");
        assert!(py.contains("return m.sqrt(4.0)"), "{py}");
    }

    #[test]
    fn atom_dot_is_a_migration_error() {
        // revision-1 spelling gets the R009 diagnostic
        let err = compile_err(
            "defmodule M do\n  def f(x), do: :math.sqrt(x)\nend",
        );
        assert!(err.contains("$math.sqrt"), "{err}");
        assert!(err.contains("GEP-0003-R009"), "{err}");
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
            "defmodule M do\n  def f(data) do\n    $json.dumps(data, indent: 2)\n  end\nend",
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
            "defmodule M do\n  @decorate $functools.cache\n  def f(x) do\n    x\n  end\nend",
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

#[cfg(test)]
mod gep0004_tests {
    use super::tests_helpers::compile;
    use super::*;

    #[test]
    fn compiles_defstruct_to_frozen_dataclass() {
        let py = compile(
            "defmodule App.User do\n  defstruct name: nil, age: 0, tags: []\nend",
        );
        assert!(py.contains("@dataclasses.dataclass(frozen=True)"), "{py}");
        assert!(py.contains("class User:"), "{py}");
        assert!(py.contains("name: object = None"), "{py}");
        assert!(py.contains("age: object = 0"), "{py}");
        assert!(
            py.contains("tags: object = dataclasses.field(default_factory=lambda: [])"),
            "{py}"
        );
    }

    #[test]
    fn compiles_atom_list_defstruct() {
        let py = compile("defmodule App.Point do\n  defstruct [:x, :y]\nend");
        assert!(py.contains("x: object = None"), "{py}");
        assert!(py.contains("y: object = None"), "{py}");
    }

    #[test]
    fn compiles_struct_literal_same_module() {
        let py = compile(
            "defmodule App.User do\n  defstruct name: nil, age: 0\n  def new(n) do\n    %App.User{name: n, age: 1}\n  end\nend",
        );
        assert!(py.contains("return User(name=n, age=1)"), "{py}");
    }

    #[test]
    fn compiles_struct_literal_cross_module_with_alias() {
        let py = compile(
            "defmodule App.Api do\n  alias App.User\n  def new(n) do\n    %User{name: n}\n  end\nend",
        );
        assert!(py.contains("import app.user"), "{py}");
        assert!(py.contains("return app.user.User(name=n)"), "{py}");
    }

    #[test]
    fn compiles_struct_pattern() {
        let py = compile(
            "defmodule App.User do\n  defstruct name: nil, age: 0\n  def name_of(u) do\n    case u do\n      %App.User{name: n} -> n\n    end\n  end\nend",
        );
        assert!(py.contains("case User(name=n):"), "{py}");
    }

    #[test]
    fn compiles_struct_update() {
        let py = compile(
            "defmodule App.User do\n  defstruct name: nil, age: 0\n  def older(u) do\n    %App.User{u | age: u.age + 1}\n  end\nend",
        );
        assert!(py.contains("dataclasses.replace(u, age=u.age + 1)"), "{py}");
    }

    #[test]
    fn compiles_map_update() {
        let py = compile(
            "defmodule M do\n  def f(m) do\n    %{m | count: 2}\n  end\nend",
        );
        assert!(py.contains("return {**m, \"count\": 2}"), "{py}");
    }

    #[test]
    fn compiles_module_attributes_and_reads() {
        let py = compile(
            "defmodule M do\n  @sep \"-\"\n  @limits %{max: 10}\n  def join(a, b) do\n    a <> @sep <> b\n  end\nend",
        );
        assert!(py.contains("sep = \"-\""), "{py}");
        assert!(py.contains("limits = {\"max\": 10}"), "{py}");
        assert!(py.contains("return a + (sep + b)"), "{py}");
    }

    #[test]
    fn attribute_decorator_chain() {
        let py = compile(
            "defmodule M do\n  @registry $collections.OrderedDict()\n  @decorate @registry.setdefault\n  def handler(x) do\n    x\n  end\nend",
        );
        assert!(py.contains("registry = collections.OrderedDict()"), "{py}");
        assert!(py.contains("@registry.setdefault"), "{py}");
    }

    #[test]
    fn rejects_duplicate_attribute() {
        let err = super::tests_helpers::compile_err(
            "defmodule M do\n  @sep \"-\"\n  @sep \"+\"\nend",
        );
        assert!(err.contains("GEP-0004-R010"), "{err}");
    }

    #[test]
    fn rejects_undefined_attribute_read() {
        let err = super::tests_helpers::compile_err(
            "defmodule M do\n  def f() do\n    @missing\n  end\nend",
        );
        assert!(err.contains("GEP-0004-R011"), "{err}");
    }

    #[test]
    fn rejects_second_defstruct() {
        let err = super::tests_helpers::compile_err(
            "defmodule M do\n  defstruct [:a]\n  defstruct [:b]\nend",
        );
        assert!(err.contains("GEP-0004-R001"), "{err}");
    }
}

#[cfg(test)]
mod gep0005_tests {
    use super::tests_helpers::{compile, compile_err};

    #[test]
    fn compiles_word_sigil() {
        let py = compile("defmodule M do\n  def f(), do: ~w(alpha beta gamma)\nend");
        assert!(py.contains("return [\"alpha\", \"beta\", \"gamma\"]"), "{py}");
    }

    #[test]
    fn compiles_interpolated_word_sigil() {
        let py = compile("defmodule M do\n  def f(x), do: ~w(a #{x} c)\nend");
        assert!(py.contains("return f\"a {x} c\".split()"), "{py}");
    }

    #[test]
    fn compiles_regex_sigil() {
        let py = compile("defmodule M do\n  def f(s) do\n    ~r/\\d+/.findall(s)\n  end\nend");
        assert!(py.contains("import re"), "{py}");
        assert!(py.contains("re.compile(\"\\\\d+\").findall(s)"), "{py}");
    }

    #[test]
    fn compiles_py_sigil_expression() {
        let py = compile(
            "defmodule M do\n  def squares(n) do\n    ~python(sum(i * i for i in range(n)))\n  end\nend",
        );
        assert!(py.contains("return (sum(i * i for i in range(n)))"), "{py}");
    }

    #[test]
    fn py_sigil_composes_with_pipes() {
        let py = compile(
            "defmodule M do\n  def f(xs) do\n    xs |> $builtins.sorted() |> ~python(list)()\n  end\nend",
        );
        assert!(py.contains("return (list)(builtins.sorted(xs))"), "{py}");
    }

    #[test]
    fn any_name_is_an_embedded_sigil_now() {
        // GEP-0005-R009 was repealed by GEP-0009-R001
        let py = compile("defmodule M do\n  def f(), do: ~z(nope)\nend");
        assert!(py.contains("return \"nope\""), "{py}");
    }
}

#[cfg(test)]
mod gep0007_tests {
    use super::tests_helpers::{compile, compile_err};

    #[test]
    fn doctests_compile_to_python_doctests() {
        let py = compile(
            "defmodule M do\n  @doc \"Adds one.\"\n  @example \"\"\"\n    gan> inc(1)\n    2\n    gan> [1, 2] |> $builtins.len()\n    2\n\"\"\"\n  def inc(x), do: x + 1\nend",
        );
        assert!(py.contains("    >>> inc(1)\n    2"), "{py}");
        assert!(py.contains("    >>> builtins.len([1, 2])\n    2"), "{py}");
    }

    #[test]
    fn localized_doc_keeps_default_in_docstring() {
        let py = compile(
            "defmodule M do\n  @doc \"Adds one.\"\n  @doc_trans zh_CN: \"加一。\"\n  @doc_trans ja: \"一を足す。\"\n  def inc(x), do: x + 1\nend",
        );
        assert!(py.contains("\"\"\"Adds one.\"\"\""), "{py}");
        assert!(!py.contains("加一"), "{py}");
    }

    #[test]
    fn doc_false_hides_docstring() {
        let py = compile(
            "defmodule M do\n  @doc false\n  def secret(x), do: x\nend",
        );
        assert!(!py.contains("\"\"\""), "{py}");
    }

    #[test]
    fn doc_trans_without_doc_is_an_error() {
        let err = compile_err(
            "defmodule M do\n  @doc_trans zh_CN: \"只有中文\"\n  def f(x), do: x\nend",
        );
        assert!(err.contains("GEP-0007-R001A"), "{err}");
    }

    #[test]
    fn duplicate_doc_trans_locale_is_an_error() {
        let err = compile_err(
            "defmodule M do\n  @doc \"d\"\n  @doc_trans zh_CN: \"一\"\n  @doc_trans zh_CN: \"二\"\n  def f(x), do: x\nend",
        );
        assert!(err.contains("duplicate"), "{err}");
    }

    #[test]
    fn broken_doctest_is_a_compile_error() {
        let err = compile_err(
            "defmodule M do\n  @example \"\"\"\n    gan> 1 +\n    2\n\"\"\"\n  def f(x), do: x\nend",
        );
        assert!(err.contains("in doctest"), "{err}");
    }
}

#[cfg(test)]
mod doc_robustness_tests {
    use super::tests_helpers::{compile, compile_err};

    #[test]
    fn chinese_prose_and_comments_survive_extraction() {
        let py = compile(
            "defmodule M do\n  @doc \"\"\"\n符号判断。# 这不是注释是正文\n\n以上 # 中文井号 无碍。\n\"\"\"\n  @example \"\"\"\n    gan> classify(-3) # 行尾中文注释会被词法器剥掉\n    'negative'\n\"\"\"\n  def classify(x), do: :negative\nend",
        );
        // prose with Chinese # passes through verbatim
        assert!(py.contains("符号判断。# 这不是注释是正文"), "{py}");
        assert!(py.contains("以上 # 中文井号 无碍。"), "{py}");
        // the doctest compiled, trailing comment stripped by the lexer
        assert!(py.contains(">>> classify(-3)\n    'negative'"), "{py}");
        assert!(!py.contains("行尾中文注释"), "{py}");
    }

    #[test]
    fn interpolation_in_doc_has_a_clear_error() {
        let err = compile_err(
            "defmodule M do\n  @doc \"价格 #{price}\"\n  def f(x), do: x\nend",
        );
        assert!(err.contains("interpolation"), "{err}");
    }
}

#[cfg(test)]
mod doc_merge_tests {
    use super::tests_helpers::{compile, compile_err};
    use super::*;
    use crate::expander::{collect_macros, Expander};
    use crate::parser::parse_file;

    #[test]
    fn doc_channels_assemble_the_docstring() {
        let py = compile(
            "defmodule M do\n  @doc since: \"1.3.0\"\n  @doc \"Adds one.\"\n  @doc_trans zh_CN: \"加一。\"\n  @example \"\"\"\n      gan> inc(1)\n      2\n  \"\"\"\n  def inc(x), do: x + 1\nend",
        );
        assert!(py.contains("Adds one."), "{py}");
        assert!(py.contains(">>> inc(1)\n    2"), "{py}");
        assert!(py.contains("Since: 1.3.0"), "{py}");
        assert!(!py.contains("加一"), "{py}");
        // assembly order: text, examples, trailer
        let text_at = py.find("Adds one.").unwrap();
        let ex_at = py.find(">>> inc(1)").unwrap();
        let meta_at = py.find("Since:").unwrap();
        assert!(text_at < ex_at && ex_at < meta_at, "{py}");
    }

    #[test]
    fn example_without_doc_still_works() {
        let py = compile(
            "defmodule M do\n  @example \"\"\"\n      gan> dbl(2)\n      4\n  \"\"\"\n  def dbl(x), do: x * 2\nend",
        );
        assert!(py.contains(">>> dbl(2)"), "{py}");
    }

    #[test]
    fn doc_text_is_opaque_and_warns_on_gan_prompt() {
        let src = "defmodule M do\n  @doc \"\"\"\n      gan> f(1)\n      1\n  \"\"\"\n  def f(x), do: x\nend";
        let module = parse_file("<test>", src).unwrap();
        let macros = collect_macros("<test>", &module).unwrap();
        let mut ex = Expander::new("<test>", macros);
        let expanded = ex.expand_module(&module).unwrap();
        let mut cg = Codegen::new("<test>", vec![]);
        let py = cg.compile(&expanded).unwrap();
        // not compiled, passes through verbatim
        assert!(py.contains("gan> f(1)"), "{py}");
        assert!(!py.contains(">>>"), "{py}");
        assert!(cg.warnings.iter().any(|w| w.contains("GEP-0007-R005")), "{:?}", cg.warnings);
    }

    #[test]
    fn doc_text_twice_is_an_error() {
        let err = compile_err(
            "defmodule M do\n  @doc \"one\"\n  @doc \"two\"\n  def f(x), do: x\nend",
        );
        assert!(err.contains("twice"), "{err}");
    }

    #[test]
    fn gan_prompt_in_translation_is_rejected() {
        let err = compile_err(
            "defmodule M do\n  @doc \"d\"\n  @doc_trans zh_CN: \"\"\"\n  示例：\n      gan> f(1)\n      1\n  \"\"\"\n  def f(x), do: x\nend",
        );
        assert!(err.contains("GEP-0007-R007"), "{err}");
        assert!(err.contains("@example"), "{err}");
    }
}

#[cfg(test)]
mod doc_meta_tests {
    use super::tests_helpers::{compile, compile_err};

    #[test]
    fn since_and_deprecated_reach_the_docstring() {
        let py = compile(
            "defmodule M do\n  @doc \"Old adder.\"\n  @doc since: \"0.1.0\", deprecated: \"Use add/2 instead.\"\n  def old_add(a, b), do: a + b\nend",
        );
        assert!(py.contains("Old adder."), "{py}");
        assert!(py.contains("Deprecated: Use add/2 instead."), "{py}");
        assert!(py.contains("Since: 0.1.0"), "{py}");
    }

    #[test]
    fn custom_meta_is_tooling_only() {
        let py = compile(
            "defmodule M do\n  @doc \"f.\"\n  @doc authors: \"MJ\", stable: true\n  def f(x), do: x\nend",
        );
        assert!(!py.contains("MJ"), "{py}");
        assert!(!py.contains("stable"), "{py}");
    }

    #[test]
    fn duplicate_meta_key_is_an_error() {
        let err = compile_err(
            "defmodule M do\n  @doc \"f.\"\n  @doc since: \"1\"\n  @doc since: \"2\"\n  def f(x), do: x\nend",
        );
        assert!(err.contains("duplicate"), "{err}");
    }
}

#[cfg(test)]
mod gep0009_tests {
    use super::tests_helpers::compile;

    #[test]
    fn arbitrary_language_sigils_are_strings() {
        let py = compile(
            "defmodule M do\n  def q(), do: ~sql(SELECT * FROM users WHERE age > 30)\nend",
        );
        assert!(py.contains("return \"SELECT * FROM users WHERE age > 30\""), "{py}");
    }

    #[test]
    fn fstring_interp_with_quotes_hoists_for_py311() {
        // f-string expressions may not contain the delimiter quote before
        // Python 3.12; the compiler hoists such expressions (targetPython 3.11)
        let py = compile(
            "defmodule M do\n  def f(), do: \"v #{$builtins.len(\"ab\")}\"\nend",
        );
        assert!(py.contains("_gan_fstr0 = builtins.len(\"ab\")"), "{py}");
        assert!(py.contains("f\"v {_gan_fstr0}\""), "{py}");
    }

    #[test]
    fn value_splices_compile_to_fstrings() {
        let py = compile(
            "defmodule M do\n  def report(name, xs) do\n    ~markdown\"\"\"\n# Report for <%= name.upper() %>\n\nTotal: <%= xs |> $builtins.sum() %>\n\"\"\"\n  end\nend",
        );
        assert!(py.contains("f\"# Report for {name.upper()}\\n\\nTotal: {builtins.sum(xs)}\\n\""), "{py}");
    }

    #[test]
    fn python_sigil_code_splices() {
        let py = compile(
            "defmodule M do\n  def evens(xs, limit) do\n    ~python([x for x in <%= xs %> if x % 2 == 0][:<%= limit + 1 %>])\n  end\nend",
        );
        assert!(py.contains("return ([x for x in (xs) if x % 2 == 0][:(limit + 1)])"), "{py}");
    }

    #[test]
    fn escaped_marker_stays_literal() {
        let py = compile(
            "defmodule M do\n  def t(), do: ~eex(a <%%= b %> c)\nend",
        );
        assert!(py.contains("\"a <%= b %> c\""), "{py}");
    }
}

#[cfg(test)]
mod gep0008_tests {
    use super::tests_helpers::compile;
    use crate::expander::{collect_macros, Expander};
    use crate::parser::parse_file;

    fn expand_err(src: &str) -> String {
        let module = parse_file("<test>", src).unwrap();
        let macros = collect_macros("<test>", &module).unwrap();
        let mut ex = Expander::new("<test>", macros);
        ex.expand_module(&module).unwrap_err().message
    }

    #[test]
    fn declaration_macro_generates_defs() {
        let py = compile(
            "defmodule M do\n  defmacro make_pair() do\n    quote do\n      def left(), do: 1\n      def right(), do: 2\n    end\n  end\n  make_pair()\n  def sum(), do: left() + right()\nend",
        );
        assert!(py.contains("def left():"), "{py}");
        assert!(py.contains("def right():"), "{py}");
        assert!(py.contains("return left() + right()"), "{py}");
    }

    #[test]
    fn def_unquote_computes_names() {
        let py = compile(
            "defmodule M do\n  defmacro define_getter(name, value) do\n    quote do\n      def unquote(name)(), do: unquote(value)\n    end\n  end\n  define_getter(:answer, 42)\n  define_getter(:pi, 3.14)\nend",
        );
        assert!(py.contains("def answer():\n    return 42"), "{py}");
        assert!(py.contains("def pi():\n    return 3.14"), "{py}");
    }

    #[test]
    fn use_invokes_using_macro() {
        let py = compile(
            "defmodule M do\n  defmacro __using__() do\n    quote do\n      def injected(), do: :from_using\n    end\n  end\n  use M\nend",
        );
        assert!(py.contains("def injected():"), "{py}");
        assert!(py.contains("return \"from_using\""), "{py}");
    }

    #[test]
    fn use_without_using_is_an_error() {
        let err = expand_err("defmodule M do\n  use NoSuch.Thing\nend");
        assert!(err.contains("GEP-0008-R003"), "{err}");
    }

    #[test]
    fn defattr_hook_rewrites_definitions() {
        let py = compile(
            "defmodule M do\n  defattr :route, accumulate: true\n  defmacro on_def(kind, head, attrs, body) do\n    quote do\n      def unquote(head) do\n        unquote(body)\n      end\n      def route_count(), do: unquote(length(attrs))\n    end\n  end\n  @on_definition M.on_def\n  @route {:get, \"/users\"}\n  @route {:post, \"/users\"}\n  def list_users(), do: :ok\nend",
        );
        assert!(py.contains("def list_users():"), "{py}");
        assert!(py.contains("def route_count():\n    return 2"), "{py}");
    }

    #[test]
    fn accumulated_attr_reads_as_list() {
        let py = compile(
            "defmodule M do\n  defattr :tag, accumulate: true\n  @tag :a\n  @tag :b\n  def tags(), do: @tag\nend",
        );
        assert!(py.contains("return [\"a\", \"b\"]"), "{py}");
    }

    #[test]
    fn builtin_attr_collision_is_an_error() {
        let err = expand_err("defmodule M do\n  defattr :doc\nend");
        assert!(err.contains("GEP-0008-R006"), "{err}");
    }

    #[test]
    fn non_accumulating_duplicate_is_an_error() {
        let err = expand_err(
            "defmodule M do\n  defattr :owner\n  @owner :a\n  @owner :b\n  def f(), do: nil\nend",
        );
        assert!(err.contains("GEP-0008-R004"), "{err}");
    }
}

#[cfg(test)]
mod gep0011_tests {
    use super::tests_helpers::{compile, compile_err};

    #[test]
    fn two_arities_share_one_function() {
        let py = compile(
            "defmodule M do\n  def get(m, k), do: m.get(k)\n  def get(m, k, d), do: m.get(k, d)\nend",
        );
        assert_eq!(py.matches("def get(").count(), 1, "{py}");
        assert!(py.contains("case (m, k,):"), "{py}");
        assert!(py.contains("case (m, k, d,):"), "{py}");
        assert!(py.contains("get/2,3 matched"), "{py}");
    }

    #[test]
    fn defaults_synthesize_delegating_clauses() {
        let py = compile(
            "defmodule M do\n  def greet(name, greeting \\\\ \"hello\", mark \\\\ \"!\") do\n    greeting <> \", \" <> name <> mark\n  end\nend",
        );
        assert!(py.contains("case (name, greeting, mark,):"), "{py}");
        assert!(py.contains("case (name, greeting,):"), "{py}");
        assert!(py.contains("case (name,):"), "{py}");
        assert!(py.contains("return greet(name, greeting, \"!\")"), "{py}");
        assert!(py.contains("return greet(name, \"hello\", \"!\")"), "{py}");
    }

    #[test]
    fn non_trailing_default_is_an_error() {
        let err = compile_err(
            "defmodule M do\n  def f(a \\\\ 1, b), do: a + b\nend",
        );
        assert!(err.contains("trailing"), "{err}");
    }

    #[test]
    fn two_defaulted_definitions_are_an_error() {
        let err = compile_err(
            "defmodule M do\n  def f(a, b \\\\ 1), do: a + b\n  def f(a, b \\\\ 2, c \\\\ 3), do: a + b + c\nend",
        );
        assert!(err.contains("GEP-0011-R003"), "{err}");
    }
}

/// Compile a statement sequence for interactive use (GEP-0012-R003):
/// the returned Python code executes the snippet and leaves the value of
/// its final expression in `_`.
pub fn compile_snippet(
    file: &str,
    term: &Term,
    project_modules: std::collections::BTreeSet<String>,
    installed_modules: BTreeMap<String, String>,
) -> crate::diag::Result<String> {
    let mut cg = Codegen::new(file, vec![]);
    cg.project_modules = project_modules;
    cg.installed_modules = installed_modules;
    cg.tmp_names.push("_".to_string());
    let result = 0usize;
    let stmts = term.as_block();
    let mut lines: Vec<String> = Vec::new();
    if stmts.is_empty() {
        lines.push("_ = None".to_string());
    }
    for (i, stmt) in stmts.iter().enumerate() {
        let dest = if i + 1 == stmts.len() {
            Dest::Assign(result)
        } else {
            Dest::Ignore
        };
        cg.emit_stmt(stmt, dest, &mut lines)?;
    }
    let mut out = String::new();
    for m in &cg.py_imports {
        out.push_str(&format!("import {m}\n"));
    }
    for m in &cg.gan_imports {
        out.push_str(&format!("import {m}\n"));
    }
    let helpers = cg.helper_code();
    if !helpers.is_empty() {
        out.push_str(&helpers);
        out.push('\n');
    }
    for l in &lines {
        out.push_str(l);
        out.push('\n');
    }
    Ok(out)
}

#[cfg(test)]
mod gep0014_tests {
    use super::tests_helpers::{compile, compile_err};

    #[test]
    fn try_rescue_after_compiles_to_python_machinery() {
        let py = compile(
            "defmodule M do\n  def parse(s) do\n    try do\n      {:ok, $builtins.int(s)}\n    rescue\n      e in $builtins.ValueError -> {:error, to_string(e)}\n      e -> {:error, :unknown}\n    after\n      IO.puts(\"done\")\n    end\n  end\nend",
        );
        assert!(py.contains("try:"), "{py}");
        assert!(py.contains("except builtins.ValueError as e:"), "{py}");
        assert!(py.contains("except Exception as e:"), "{py}");
        assert!(py.contains("finally:"), "{py}");
        assert!(py.contains("print(\"done\")"), "{py}");
    }

    #[test]
    fn try_is_an_expression() {
        let py = compile(
            "defmodule M do\n  def f(s) do\n    v = try do\n      $builtins.int(s)\n    rescue\n      _e -> 0\n    end\n    v + 1\n  end\nend",
        );
        assert!(py.contains("return v + 1"), "{py}");
    }

    #[test]
    fn loop_recur_break_compile_to_while() {
        let py = compile(
            "defmodule M do\n  def count_to(n) do\n    loop {acc, i} = {0, 0} do\n      if i >= n do\n        break(acc)\n      else\n        recur({acc + i, i + 1})\n      end\n    end\n  end\nend",
        );
        assert!(py.contains("while True:"), "{py}");
        assert!(py.contains("continue"), "{py}");
        assert!(py.contains("break"), "{py}");
        assert!(py.contains("case (acc, i)"), "{py}");
    }

    #[test]
    fn recur_outside_loop_is_an_error() {
        let err = compile_err("defmodule M do\n  def f(), do: recur(1)\nend");
        assert!(err.contains("GEP-0014-R005"), "{err}");
    }

    #[test]
    fn try_without_rescue_or_after_is_an_error() {
        let err = compile_err(
            "defmodule M do\n  def f() do\n    try do\n      1\n    end\n  end\nend",
        );
        assert!(err.contains("GEP-0014-R003"), "{err}");
    }

    #[test]
    fn membership_in_operator() {
        let py = compile(
            "defmodule M do\n  def has?(x, xs), do: x in xs\nend",
        );
        assert!(py.contains("return x in xs"), "{py}");
    }
}
