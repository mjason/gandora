//! Python code generation (GEP-0001-R009..R017, GEP-0003).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use crate::ast::{Call, Callee, StrPart, Term};
use crate::diag::{Diagnostic, Result, Span};

const PY_KEYWORDS: &[&str] = &[
    "False", "None", "True", "and", "as", "assert", "async", "await", "break", "class",
    "continue", "def", "del", "elif", "else", "except", "finally", "for", "from", "global", "if",
    "import", "in", "is", "lambda", "nonlocal", "not", "or", "pass", "raise", "return", "try",
    "while", "with", "yield",
];

/// Gandora characters to Python spelling, shared by both positions.
fn map_ident_chars(name: &str) -> String {
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
    out
}

/// Map a Gandora identifier to a Python identifier (GEP-0001-R015).
/// Binding position: a Python keyword collision must be renamed
/// (`class` -> `class__kw`) or the generated code will not parse.
/// `match` is only a soft keyword — legal everywhere — so it is not
/// renamed anywhere.
pub fn map_ident(name: &str) -> String {
    let mut out = map_ident_chars(name);
    if PY_KEYWORDS.contains(&out.as_str()) {
        out.push_str("__kw");
    }
    out
}

/// Attribute position (`obj.name`): keywords are never renamed here —
/// `pattern.match(...)` must stay `match`. A hard keyword cannot be a
/// Python attribute spelling at all; None tells the caller to error
/// with a getattr recipe.
fn map_attr(name: &str) -> Option<String> {
    let out = map_ident_chars(name);
    if PY_KEYWORDS.contains(&out.as_str()) {
        None
    } else {
        Some(out)
    }
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
    /// `@param` docs in declaration order: (name, [(locale, text)])
    /// with "default" first (GEP-0018)
    pub params: Vec<(String, Vec<(String, String)>)>,
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
    /// compiled recursion shape (GEP-0019-R006): "loop" — tail recursion
    /// became `while True:`; "stack" — self-recursive on the call stack
    pub tco: Option<String>,
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

/// The default documentation channel stays in English (GEP-0007-R011):
/// the retrieval index ranks its words, so CJK prose there betrays a
/// translation written into the wrong channel. `_trans` channels are
/// exactly where that prose belongs.
pub fn english_channel(
    file: &str,
    span: Span,
    attr: &str,
    text: &str,
) -> crate::diag::Result<()> {
    let cjk = text.chars().any(|c| {
        let u = c as u32;
        (0x3000..=0x30FF).contains(&u)       // CJK punctuation + kana
            || (0x3400..=0x4DBF).contains(&u) // CJK extension A
            || (0x4E00..=0x9FFF).contains(&u) // CJK unified
            || (0xAC00..=0xD7AF).contains(&u) // Hangul
            || (0xF900..=0xFAFF).contains(&u) // CJK compatibility
            || (0xFF00..=0xFFEF).contains(&u) // fullwidth forms
    });
    if cjk {
        return Err(Diagnostic::new(
            file,
            span,
            format!(
                "{attr} is the default documentation channel and stays in English — \
                 the retrieval index ranks its words (GEP-0007-R011); localized prose \
                 belongs in the {attr}_trans channel, e.g. {attr}_trans zh_CN: \"...\""
            ),
        ));
    }
    Ok(())
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
            let text = t.as_plain_str().unwrap();
            english_channel(file, call.span, attr, &text)?;
            info.entries.push(("default".to_string(), text));
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
    /// enclosing locals a hoisted `fn` reads, snapshot as `name=name`
    /// keyword-only defaults (GEP-0021); empty for module-level defs
    captures: Vec<String>,
    /// acknowledged lints from `@allow :target` (GEP-0019-R007)
    allows: BTreeSet<String>,
    /// declared `p \\ expr` defaults (GEP-0011); when the group is one
    /// real clause and every default is an immutable literal, the
    /// signature emits native Python defaults instead of a dispatcher
    defaults: Vec<Term>,
    /// defined with `async def`/`async defp`: compiles to Python's
    /// `async def`, and `await` is legal in the body (GEP-0030)
    is_async: bool,
}

pub struct Codegen {
    file: String,
    typevars: BTreeSet<String>,
    /// active TCO context: (name, arities, simple param names) (GEP-0019)
    tail_ctx: Option<(String, BTreeSet<usize>, Option<Vec<String>>)>,
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
    /// named types of the whole project: (dotted module, name) ->
    /// (params, body) (GEP-0027)
    pub project_types: BTreeMap<(String, String), (Vec<String>, Term)>,
    /// this module's own named types
    local_types: BTreeMap<String, (Vec<String>, Term)>,
    /// expansion stack for cycle detection in named types
    type_stack: Vec<String>,
    /// installed marker names -> dotted python paths (GEP-0006-R005A)
    pub installed_modules: BTreeMap<String, String>,
    struct_fields: Option<Vec<(String, Term)>>,
    /// non-fatal notices surfaced by `gan check`/`gan build` and, with
    /// their spans, as editor warning diagnostics
    pub warnings: Vec<crate::diag::Diagnostic>,
    /// true after `compile` when the module defines no runtime code
    /// (macros only) and should produce no Python file (GEP-0002-R009).
    pub compile_time_only: bool,
    tmp_counter: usize,
    tmp_names: Vec<String>,
    fn_counter: usize,
    /// inside the body of an `async def`, where `await` is legal
    /// (GEP-0030-R002)
    async_ctx: bool,
    /// locals of each enclosing function scope, innermost last — the
    /// names a closure may capture and must snapshot (GEP-0021)
    scope_bound: Vec<BTreeSet<String>>,
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
            project_types: BTreeMap::new(),
            local_types: BTreeMap::new(),
            type_stack: Vec::new(),
            installed_modules: BTreeMap::new(),
            struct_fields: None,
            warnings: Vec::new(),
            typevars: BTreeSet::new(),
            tail_ctx: None,
            async_ctx: false,
            compile_time_only: false,
            tmp_counter: 0,
            tmp_names: Vec::new(),
            fn_counter: 0,
            scope_bound: Vec::new(),
        }
    }

    fn err(&self, span: Span, msg: impl Into<String>) -> Diagnostic {
        Diagnostic::new(&self.file, span, msg)
    }

    /// Parse `name(params) :: body` of a `@type` declaration
    /// (GEP-0027-R001): lowercase name, 1–2 letter parameters, and a
    /// body that uses only declared parameters as bare variables.
    fn parse_type_decl(
        &mut self,
        value: &Term,
        span: Span,
    ) -> Result<(String, Vec<String>, Term)> {
        let Term::Call(cc) = value else {
            return Err(self.err(span, "@type requires `name(params) :: type` (GEP-0027-R001)"));
        };
        if !matches!(&cc.callee, Callee::Name(n) if n == "::") {
            return Err(self.err(span, "@type requires `name(params) :: type` (GEP-0027-R001)"));
        }
        let head = &cc.args[0];
        let body = cc.args[1].clone();
        let Term::Call(h) = head else {
            return Err(self.err(
                span,
                "a type is a call — write @type name() :: ... (GEP-0027-R001)",
            ));
        };
        let Callee::Name(tname) = &h.callee else {
            return Err(self.err(span, "@type names are plain lowercase words (GEP-0027-R001)"));
        };
        let tname = tname.clone();
        if tname == "t" {
            return Err(self.err(
                span,
                "`t` was retired — the module itself is its struct type \
                 (Mod()); give this type a meaningful name (GEP-0017 rev 5)",
            ));
        }
        if !tname.chars().next().is_some_and(|c| c.is_ascii_lowercase()) {
            return Err(self.err(span, "@type names are plain lowercase words (GEP-0027-R001)"));
        }
        if spec_builtin_type(&tname) {
            return Err(self.err(
                span,
                format!("@type {tname} shadows the built-in type {tname}() (GEP-0027-R001)"),
            ));
        }
        let mut params = Vec::new();
        for p in &h.args {
            match p {
                Term::Var(v, _) if v.chars().count() <= 2 => params.push(v.clone()),
                _ => {
                    return Err(self.err(
                        span,
                        "@type parameters are type variables: 1-2 lowercase \
                         letters (GEP-0027-R002)",
                    ))
                }
            }
        }
        // every bare variable in the body must be a declared parameter
        let mut free = BTreeSet::new();
        type_body_vars(&body, &mut free);
        for v in &free {
            if !params.contains(v) {
                return Err(self.err(
                    span,
                    format!(
                        "type variable `{v}` is not declared by @type {tname}({}) \
                         (GEP-0027-R002)",
                        params.join(", ")
                    ),
                ));
            }
        }
        Ok((tname, params, body))
    }

    /// Expand a named-type reference to its Python annotation
    /// (GEP-0027-R003): arity-checked, parameters substituted, cycles
    /// rejected.
    fn expand_named_type(
        &mut self,
        label: &str,
        params: &[String],
        body: &Term,
        args: &[Term],
        span: Span,
    ) -> Result<String> {
        if args.len() != params.len() {
            return Err(self.err(
                span,
                format!(
                    "{label} takes {} type parameter(s), got {} (GEP-0027-R003)",
                    params.len(),
                    args.len()
                ),
            ));
        }
        if self.type_stack.iter().any(|l| l == label) || self.type_stack.len() > 32 {
            return Err(self.err(
                span,
                format!("recursive named types are not supported: {label} (GEP-0027-R003)"),
            ));
        }
        let substituted = substitute_type_vars(body, params, args);
        self.type_stack.push(label.to_string());
        let out = self.spec_hint(&substituted);
        self.type_stack.pop();
        out
    }

    /// A Python attribute name, or the getattr recipe for the rare
    /// hard-keyword collision (GEP-0001-R015 attribute position).
    fn attr_name(&self, name: &str, span: Span) -> Result<String> {
        map_attr(name).ok_or_else(|| {
            self.err(
                span,
                format!(
                    "`.{name}` collides with a Python keyword and cannot be \
                     an attribute — use $builtins.getattr(x, \"{name}\") \
                     (GEP-0003)"
                ),
            )
        })
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
        let mut pending_params: Vec<(String, Vec<(String, String)>)> = Vec::new();
        let mut pending_decorators: Vec<Term> = Vec::new();
        let mut pending_allows: BTreeSet<String> = BTreeSet::new();
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
                "@param" => {
                    let (name_t, text_t) = match (call.args.first(), call.args.get(1)) {
                        (Some(n), Some(t)) => (n, t),
                        _ => {
                            return Err(self.err(
                                call.span,
                                "@param requires `name, \"text\"` (GEP-0018-R001)",
                            ))
                        }
                    };
                    let pname = match name_t {
                        Term::Var(n, _) => n.clone(),
                        _ => {
                            return Err(self.err(
                                call.span,
                                "@param requires a parameter name (GEP-0018-R001)",
                            ))
                        }
                    };
                    let text = text_t.as_plain_str().ok_or_else(|| {
                        self.err(call.span, "@param text must be a plain string (GEP-0018-R001)")
                    })?;
                    english_channel(&self.file, call.span, "@param", &text)?;
                    if pending_params.iter().any(|(n, _)| *n == pname) {
                        return Err(self.err(
                            call.span,
                            format!("duplicate @param for {pname} (GEP-0018-R001)"),
                        ));
                    }
                    pending_params.push((pname, vec![("default".to_string(), text)]));
                }
                "@param_trans" => {
                    let pname = match call.args.first() {
                        Some(Term::Var(n, _)) => n.clone(),
                        _ => {
                            return Err(self.err(
                                call.span,
                                "@param_trans requires `name, locale: \"text\"` (GEP-0018-R003)",
                            ))
                        }
                    };
                    let entry = pending_params
                        .iter_mut()
                        .find(|(n, _)| *n == pname)
                        .ok_or_else(|| {
                            self.err(
                                call.span,
                                format!(
                                    "@param_trans {pname} has no preceding @param \
                                     (GEP-0018-R003)"
                                ),
                            )
                        })?;
                    for arg in call.args.iter().skip(1) {
                        let Term::Pair(locale, value) = arg else {
                            return Err(self.err(
                                call.span,
                                "@param_trans takes `locale: \"text\"` pairs (GEP-0018-R003)",
                            ));
                        };
                        let text = value.as_plain_str().ok_or_else(|| {
                            self.err(
                                call.span,
                                "@param_trans text must be a plain string (GEP-0018-R003)",
                            )
                        })?;
                        entry.1.push((locale.replace('_', "-"), text));
                    }
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
                "@type" => {
                    // a named type: `@type name(params) :: type` — the
                    // declaration site of generics (GEP-0027)
                    let value = call.args.first().ok_or_else(|| {
                        self.err(call.span, "@type requires `name(params) :: type` (GEP-0027-R001)")
                    })?;
                    let (tname, params, body) = self.parse_type_decl(value, call.span)?;
                    // a pending @doc belongs to the type, not the next def
                    let _ = pending_doc.take();
                    if !pending_params.is_empty() || pending_spec.is_some() {
                        return Err(self.err(
                            call.span,
                            "@param/@spec do not apply to @type (GEP-0027-R001)",
                        ));
                    }
                    if self
                        .local_types
                        .insert(tname.clone(), (params, body))
                        .is_some()
                    {
                        return Err(self.err(
                            call.span,
                            format!("@type {tname} is declared twice (GEP-0027-R001)"),
                        ));
                    }
                }
                "@decorate" => {
                    if let Some(e) = call.args.first() {
                        pending_decorators.push(e.clone());
                    }
                }
                "@allow" => {
                    // lint acknowledgment for the next definition
                    // (GEP-0019-R007); unknown targets are typos
                    for a in &call.args {
                        match a {
                            Term::Atom(s)
                                if s == "stack_recursion" || s == "unused_function" =>
                            {
                                pending_allows.insert(s.clone());
                            }
                            Term::Atom(s) => {
                                return Err(self.err(
                                    call.span,
                                    format!(
                                        "@allow does not recognize :{s}; known \
                                         targets: :stack_recursion, \
                                         :unused_function (GEP-0019-R007, GEP-0022)"
                                    ),
                                ));
                            }
                            _ => {
                                return Err(self.err(
                                    call.span,
                                    "@allow takes atom targets, e.g. \
                                     @allow :stack_recursion (GEP-0019-R007)",
                                ));
                            }
                        }
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
                    let resolved_path = self.resolve_module_path(&segs);
                    self.gan_imports.insert(resolved_path);
                    self.aliases.insert(short, segs);
                }
                "import" => {
                    let segs = match call.args.first() {
                        Some(Term::Alias(segs)) => segs.clone(),
                        _ => return Err(self.err(call.span, "import requires a module name")),
                    };
                    let resolved_path = self.resolve_module_path(&segs);
                    self.star_imports.insert(resolved_path);
                }
                "require" => { /* compile-time only */ }
                "defmacro" => {
                    // compile-time only — but its @doc/@param/@spec belong
                    // to IT (served by the doc surfaces), not to the next
                    // def; consume them here so they cannot leak
                    let _ = pending_doc.take();
                    let _ = pending_spec.take();
                    pending_params.clear();
                    pending_decorators.clear();
                    pending_allows.clear();
                }
                "defstruct" => {
                    if self.struct_fields.is_some() {
                        return Err(self.err(
                            call.span,
                            "a module may declare at most one defstruct (GEP-0004-R001)",
                        ));
                    }
                    self.struct_fields = Some(self.parse_defstruct(call)?);
                }
                "def" | "defp" | "async def" | "async defp" => {
                    let is_async = name.starts_with("async ");
                    let private = name.ends_with("defp");
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
                        if private {
                            self.private_funs.insert((fname.clone(), arity));
                        }
                    }
                    let idx = if let Some(&i) = order.get(&key) {
                        if pending_doc.is_some()
                            || pending_spec.is_some()
                            || !pending_params.is_empty()
                            || !pending_decorators.is_empty()
                            || !pending_allows.is_empty()
                        {
                            return Err(self.err(
                                call.span,
                                "@doc/@spec/@decorate/@allow must precede the first \
                                 clause of a function",
                            ));
                        }
                        i
                    } else {
                        if is_async && fname == "main" {
                            return Err(self.err(
                                call.span,
                                "main stays synchronous — enter the coroutine \
                                 world with Task.run (GEP-0030-R001)",
                            ));
                        }
                        funs.push(FnDef {
                            name: fname.clone(),
                            private,
                            spec: pending_spec.take(),
                            doc: {
                                let mut d = pending_doc.take();
                                if !pending_params.is_empty() {
                                    d.get_or_insert_with(DocInfo::default).params =
                                        std::mem::take(&mut pending_params);
                                }
                                d
                            },
                            decorators: std::mem::take(&mut pending_decorators),
                            clauses: Vec::new(),
                            span: call.span,
                            captures: Vec::new(),
                            allows: std::mem::take(&mut pending_allows),
                            defaults: Vec::new(),
                            is_async,
                        });
                        order.insert(key, funs.len() - 1);
                        funs.len() - 1
                    };
                    if funs[idx].private != private {
                        return Err(self.err(
                            call.span,
                            format!("clauses of {fname} mix def and defp"),
                        ));
                    }
                    if funs[idx].is_async != is_async {
                        return Err(self.err(
                            call.span,
                            format!(
                                "clauses of {fname} mix def and async def \
                                 (GEP-0030-R001)"
                            ),
                        ));
                    }
                    let arity = params.len();
                    funs[idx].clauses.push((params.clone(), guard, fbody));
                    if !defaults.is_empty() {
                        funs[idx].defaults = defaults.clone();
                    }
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
        // a private function nothing references is dead code
        // (GEP-0022-R005); references from its own group don't count
        for (pi, p) in funs.iter().enumerate() {
            if !p.private || p.allows.contains("unused_function") {
                continue;
            }
            let referenced = funs.iter().enumerate().any(|(i, other)| {
                i != pi
                    && other.clauses.iter().any(|(params, guard, body)| {
                        params.iter().any(|t| name_referenced(t, &p.name))
                            || guard
                                .as_ref()
                                .is_some_and(|g| name_referenced(g, &p.name))
                            || name_referenced(body, &p.name)
                    })
            }) || attrs.iter().any(|(_, v)| name_referenced(v, &p.name))
                || funs.iter().enumerate().any(|(i, other)| {
                    i != pi
                        && other
                            .decorators
                            .iter()
                            .any(|d| name_referenced(d, &p.name))
                });
            if !referenced {
                self.warnings.push(crate::diag::Diagnostic::new(
                    &self.file,
                    p.span,
                    format!(
                        "defp {} is never referenced in this module; remove it \
                         or acknowledge with @allow :unused_function \
                         (GEP-0022-R005)",
                        p.name
                    ),
                ));
            }
        }

        let mut out = String::new();
        let module_docstring =
            self.docstring_text(moduledoc.as_ref(), Span::default())?;
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
        if !self.typevars.is_empty() {
            out.push('\n');
            for tv in &self.typevars {
                out.push_str(&format!("{tv} = typing.TypeVar(\"{tv}\")\n"));
            }
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
            // a short lowercase name is a type variable (GEP-0017-R005):
            // compiled to a module-level typing.TypeVar
            Term::Var(n, _) if n.len() <= 2 => {
                self.py_imports.insert("typing".into());
                let tv = format!("_T_{n}");
                self.typevars.insert(tv.clone());
                Ok(tv)
            }
            Term::Var(n, _) => match spec_type_suggestion(n) {
                Some(fix) => Err(self.err(
                    Span::default(),
                    format!("'{n}' is not a type — write {fix} (GEP-0017-R002)"),
                )),
                None => Err(self.err(
                    Span::default(),
                    format!("'{n}' is not a type — did you mean {n}()? (GEP-0017-R002)"),
                )),
            },
            Term::Call(c) => match &c.callee {
                // a named argument `name :: type` contributes its type
                // (GEP-0018-R006)
                Callee::Name(n) if n == "::" => self.spec_hint(&c.args[1]),
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
                    ("any", 0) | ("term", 0) => Ok("object".into()),
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
                    ("keyword", 0) => Ok("list[tuple[str, object]]".into()),
                    ("iterable", 0) | ("sequence", 0) | ("mapping", 0) => {
                        self.py_imports.insert("collections.abc".into());
                        let class = match n.as_str() {
                            "iterable" => "Iterable",
                            "sequence" => "Sequence",
                            _ => "Mapping",
                        };
                        Ok(format!("collections.abc.{class}"))
                    }
                    ("iterable", 1) | ("sequence", 1) => {
                        self.py_imports.insert("collections.abc".into());
                        let class = if n == "iterable" { "Iterable" } else { "Sequence" };
                        Ok(format!(
                            "collections.abc.{class}[{}]",
                            self.spec_hint(&c.args[0])?
                        ))
                    }
                    ("mapping", 2) => {
                        self.py_imports.insert("collections.abc".into());
                        Ok(format!(
                            "collections.abc.Mapping[{}, {}]",
                            self.spec_hint(&c.args[0])?,
                            self.spec_hint(&c.args[1])?
                        ))
                    }
                    _ => {
                        // a named type of this module (GEP-0027)
                        if let Some((params, body)) = self.local_types.get(n).cloned() {
                            return self.expand_named_type(
                                &format!("{}.{n}", self.module_segs.join(".")),
                                &params,
                                &body,
                                &c.args,
                                c.span,
                            );
                        }
                        match spec_type_suggestion(n) {
                            Some(fix) => Err(self.err(
                                c.span,
                                format!(
                                    "'{n}' is not a type — write {fix} \
                                     (GEP-0017-R002)"
                                ),
                            )),
                            None => {
                                let mut hint = String::new();
                                let known: Vec<&String> =
                                    self.local_types.keys().collect();
                                if let Some(near) = known
                                    .iter()
                                    .find(|k| strsim_close(k, n))
                                {
                                    hint = format!(" — did you mean {near}()?");
                                }
                                Err(self.err(
                                    c.span,
                                    format!(
                                        "'{n}' is not a type; built-ins look \
                                         like integer(), string(), list(t), \
                                         and @type declares named ones \
                                         (GEP-0017-R002){hint}"
                                    ),
                                ))
                            }
                        }
                    }
                },
                Callee::Dot {
                    base,
                    name,
                    is_call,
                } => {
                    // one rule (GEP-0017 rev 5): a type is a call —
                    // `Mod()`, `$mod.Type()`; bare dotted spellings error
                    if !is_call {
                        return Err(self.err(
                            c.span,
                            format!(
                                "a type is a call — write {}.{name}() \
                                 (GEP-0017-R002)",
                                spec_dot_base_text(base)
                            ),
                        ));
                    }
                    match base.as_ref() {
                        chain_base if pyref_chain(chain_base).is_some() => {
                            let (mut segs, bounded) = pyref_chain(chain_base).unwrap();
                            segs.push(name.clone());
                            self.py_imports.insert(pyref_import_path(&segs, bounded));
                            let path = segs.join(".");
                            if c.args.is_empty() {
                                Ok(path)
                            } else {
                                let parts: Vec<String> = c
                                    .args
                                    .iter()
                                    .map(|a| self.spec_hint(a))
                                    .collect::<Result<_>>()?;
                                Ok(format!("{path}[{}]", parts.join(", ")))
                            }
                        }
                        // $mod.Type() — the host's own types at the boundary;
                        // $mod.Type(t, ...) parametrizes: mod.Type[t, ...]
                        Term::PyRef(m, _) => {
                            self.py_imports.insert(m.clone());
                            if c.args.is_empty() {
                                Ok(format!("{m}.{name}"))
                            } else {
                                let parts: Vec<String> = c
                                    .args
                                    .iter()
                                    .map(|a| self.spec_hint(a))
                                    .collect::<Result<_>>()?;
                                Ok(format!("{m}.{name}[{}]", parts.join(", ")))
                            }
                        }
                        // the retired `.t()` spelling: the module itself is
                        // the type now (GEP-0017 rev 5)
                        Term::Alias(segs) if name == "t" => Err(self.err(
                            c.span,
                            format!(
                                "the struct type is the module itself — write \
                                 {}() instead of {}.t() (GEP-0017 rev 5)",
                                segs.join("."),
                                segs.join(".")
                            ),
                        )),
                        // `Mod.name(...)` — a named type of another module
                        // (GEP-0027)
                        Term::Alias(segs) => {
                            let resolved = self.resolve_alias(segs);
                            let joined = resolved.join(".");
                            if resolved == self.module_segs {
                                if let Some((params, body)) =
                                    self.local_types.get(name).cloned()
                                {
                                    return self.expand_named_type(
                                        &format!("{joined}.{name}"),
                                        &params,
                                        &body,
                                        &c.args,
                                        c.span,
                                    );
                                }
                            }
                            if let Some((params, body)) = self
                                .project_types
                                .get(&(joined.clone(), name.clone()))
                                .cloned()
                            {
                                return self.expand_named_type(
                                    &format!("{joined}.{name}"),
                                    &params,
                                    &body,
                                    &c.args,
                                    c.span,
                                );
                            }
                            Err(self.err(
                                c.span,
                                format!(
                                    "{joined} declares no @type {name}() \
                                     (GEP-0027-R003)"
                                ),
                            ))
                        }
                        _ => Err(self.err(
                            c.span,
                            "types are built-ins, $mod.Type(), or Mod() (GEP-0017-R002)",
                        )),
                    }
                }
                // `App.Shop()` — the module IS the type: its struct class
                // (GEP-0017 rev 5; uppercase call = class, like $mod.Type())
                Callee::Apply(inner) => match inner.as_ref() {
                    Term::Alias(segs) if c.args.is_empty() => {
                        let segs = segs.clone();
                        Ok(self.struct_ref(&segs))
                    }
                    Term::Alias(_) => Err(self.err(
                        c.span,
                        "Mod() names the struct class and takes no parameters \
                         (GEP-0017-R002)",
                    )),
                    _ => Err(self.err(
                        c.span,
                        "types are built-ins, $mod.Type(), or Mod() (GEP-0017-R002)",
                    )),
                },
            },
            Term::Atom(a) => Err(self.err(
                Span::default(),
                format!(
                    "':{a}' is a value, not a type — the type of atoms is \
                     atom(); a nil return is nil (GEP-0017-R002)"
                ),
            )),
            // a bare capitalized name (Int, String) is a cross-language
            // reflex, not a struct reference
            Term::Alias(segs) if segs.len() == 1 => {
                match spec_type_suggestion(&segs[0]) {
                    Some(fix) => Err(self.err(
                        Span::default(),
                        format!(
                            "'{}' is not a type — write {fix} (GEP-0017-R002)",
                            segs[0]
                        ),
                    )),
                    None => Err(self.err(
                        Span::default(),
                        format!(
                            "'{}' is not a type; a struct type is the \
                             module called — {}() (GEP-0017-R002)",
                            segs[0], segs[0]
                        ),
                    )),
                }
            }
            // a bare dotted module (App.Shop) — a type is a call
            Term::Alias(segs) => Err(self.err(
                Span::default(),
                format!(
                    "a type is a call — write {}() (GEP-0017-R002)",
                    segs.join(".")
                ),
            )),
            // {:ok, t} is an Elixir-spec reflex — the shape documents as
            // a tuple() type here
            Term::Tuple(parts) => Err(self.err(
                Span::default(),
                format!(
                    "a tuple literal is not a type — a {}-tuple is spelled \
                     tuple({}); the {{:ok, x}} shape documents as \
                     tuple(atom(), x) (GEP-0017-R002)",
                    parts.len(),
                    vec!["t"; parts.len()].join(", ")
                ),
            )),
            other => Err(self.err(
                other.span(),
                "types are built-ins like integer(), string(), list(t), \
                 sequence(t), $mod.Type(), or Mod() (GEP-0017-R002)",
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
        if let Some(doc) = &f.doc {
            if !doc.params.is_empty() {
                let mut head_vars: BTreeSet<String> = BTreeSet::new();
                for (params, _, _) in &f.clauses {
                    for p in params {
                        collect_vars(p, &mut head_vars);
                    }
                }
                for (pname, _) in &doc.params {
                    if !head_vars.contains(pname) {
                        return Err(self.err(
                            f.span,
                            format!(
                                "@param {pname} names no parameter of {} (GEP-0018-R002)",
                                f.name
                            ),
                        ));
                    }
                }
            }
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
        let arity_set: BTreeSet<usize> = arities.iter().copied().collect();
        let tail = f
            .clauses
            .iter()
            .any(|(_, _, b)| self.has_tail_self(b, &f.name, &arity_set));
        // self-recursive but never in tail position: it will grow the
        // Python call stack — say so where the developer can see it
        // (GEP-0019-R007; same predicate as the "stack" doc shape)
        if !tail
            && !f.allows.contains("stack_recursion")
            && f.clauses
                .iter()
                .any(|(_, _, b)| calls_self(b, &f.name, &arity_set))
        {
            self.warnings.push(crate::diag::Diagnostic::new(
                &self.file,
                f.span,
                format!(
                    "{}/{arity_label} is self-recursive outside tail position: \
                     it grows the Python call stack (~1000 frames); make the \
                     self-call the last expression (accumulator form) or assert \
                     constant stack with recur (GEP-0019-R007)",
                    f.name
                ),
            ));
        }
        let saved_tail = self.tail_ctx.take();
        let saved_async = self.async_ctx;
        self.async_ctx = f.is_async;
        let def_kw = if f.is_async { "async def" } else { "def" };
        // this function's locals: what closures inside may capture
        let mut fn_scope: BTreeSet<String> = f.captures.iter().cloned().collect();
        for (params, _, body) in &f.clauses {
            for p in params {
                pattern_binds(p, &mut fn_scope);
            }
            scope_binders(body, &mut fn_scope);
        }
        self.lint_fun(f, &fn_scope, &arity_label);
        self.scope_bound.push(fn_scope);
        // keyword-only creation-time snapshots of captured locals
        // (GEP-0021-R001): `def f(x, *, n=n):` / `def f(*_gan_args, n=n):`
        let caps: Vec<String> = f.captures.iter().map(|c| format!("{c}={c}")).collect();
        let with_caps = |base: String| -> String {
            if caps.is_empty() {
                base
            } else if base.is_empty() {
                format!("*, {}", caps.join(", "))
            } else {
                format!("{base}, *, {}", caps.join(", "))
            }
        };
        let cap_star = if caps.is_empty() {
            String::new()
        } else {
            format!(", {}", caps.join(", "))
        };
        // one real clause whose omitted-suffix delegates can fold into
        // native Python defaults — only immutable literals, so Python's
        // def-time evaluation matches Elixir's call-time semantics
        let const_literal = |t: &Term| {
            matches!(t, Term::Int(_) | Term::Float(_) | Term::Bool(_) | Term::Nil)
                || matches!(t, Term::Atom(_))
                || matches!(t, Term::Str(_) if t.as_plain_str().is_some())
        };
        let native_defaults = !f.defaults.is_empty()
            && f.clauses.len() == f.defaults.len() + 1
            && f.defaults.iter().all(const_literal)
            && f.clauses[1..].iter().all(|(ps, g, b)| {
                g.is_none()
                    && ps.len() < f.clauses[0].0.len()
                    && matches!(b, Term::Call(c)
                        if matches!(&c.callee, Callee::Name(n) if *n == f.name))
            });
        let simple = (f.clauses.len() == 1 || native_defaults)
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
            // trailing `= literal` suffixes for folded defaults
            let n_defaults = if native_defaults { f.defaults.len() } else { 0 };
            let mut default_py: Vec<Option<String>> = vec![None; names.len()];
            for (j, d) in f.defaults.iter().enumerate().take(n_defaults) {
                let mut pre = Vec::new();
                let e = self.emit_expr(d, &mut pre)?;
                default_py[names.len() - n_defaults + j] = Some(e);
            }
            // `x: int = 1` annotated, `x=1` bare — the way a reviewer writes
            let with_default = |i: usize, base: String| match &default_py[i] {
                Some(d) if base.contains(':') => format!("{base} = {d}"),
                Some(d) => format!("{base}={d}"),
                None => base,
            };
            match &spec_info {
                Some((hints, ret)) if hints.len() == names.len() => {
                    let typed: Vec<String> = names
                        .iter()
                        .zip(hints)
                        .enumerate()
                        .map(|(i, (n, h))| with_default(i, format!("{n}: {h}")))
                        .collect();
                    lines.push(format!(
                        "{def_kw} {py_name}({}) -> {ret}:",
                        with_caps(typed.join(", "))
                    ));
                }
                Some((_, ret)) => {
                    let sig: Vec<String> = names
                        .iter()
                        .enumerate()
                        .map(|(i, n)| with_default(i, n.clone()))
                        .collect();
                    lines.push(format!(
                        "{def_kw} {py_name}({}) -> {ret}:",
                        with_caps(sig.join(", "))
                    ));
                }
                None => {
                    let sig: Vec<String> = names
                        .iter()
                        .enumerate()
                        .map(|(i, n)| with_default(i, n.clone()))
                        .collect();
                    lines.push(format!("{def_kw} {py_name}({}):", with_caps(sig.join(", "))))
                }
            }
            let mut body_lines = Vec::new();
            if let Some(compiled) =
                self.docstring_text(f.doc.as_ref(), f.span)?
            {
                body_lines.push(format!(
                    "\"\"\"{}\"\"\"",
                    compiled.replace("\"\"\"", "\\\"\\\"\\\"")
                ));
            }
            if tail {
                self.tail_ctx =
                    Some((f.name.clone(), arity_set.clone(), Some(names.clone())));
                let mut core = Vec::new();
                self.emit_stmt_block(body, Dest::Return, &mut core)?;
                body_lines.push("while True:".to_string());
                push_indented(&mut body_lines, &core);
            } else {
                self.emit_stmt_block(body, Dest::Return, &mut body_lines)?;
            }
            push_indented(&mut lines, &body_lines);
        } else {
            match &spec_info {
                Some((_, ret)) => {
                    lines.push(format!("{def_kw} {py_name}(*_gan_args{cap_star}) -> {ret}:"))
                }
                None => lines.push(format!("{def_kw} {py_name}(*_gan_args{cap_star}):")),
            }
            let mut body_lines = Vec::new();
            if let Some(compiled) =
                self.docstring_text(f.doc.as_ref(), f.span)?
            {
                body_lines.push(format!(
                    "\"\"\"{}\"\"\"",
                    compiled.replace("\"\"\"", "\\\"\\\"\\\"")
                ));
            }
            if tail {
                self.tail_ctx = Some((f.name.clone(), arity_set.clone(), None));
            }
            let mut core: Vec<String> = Vec::new();
            core.push("match _gan_args:".to_string());
            let body_lines_saved = std::mem::take(&mut body_lines);
            body_lines = core;
            let docstring_prefix = body_lines_saved;
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
            let mut assembled = docstring_prefix;
            if tail {
                assembled.push("while True:".to_string());
                push_indented(&mut assembled, &body_lines);
            } else {
                assembled.extend(body_lines);
            }
            push_indented(&mut lines, &assembled);
        }
        self.scope_bound.pop();
        self.tail_ctx = saved_tail;
        self.async_ctx = saved_async;
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
                self.warnings.push(crate::diag::Diagnostic::new(
                    &self.file,
                    span,
                    "doc text contains a gan> line; it will not be tested — \
                     move examples into @example (GEP-0007-R005)",
                ));
            }
            parts.push(text.trim_end().to_string());
        }
        if let Some(section) = params_section(info, "default", "## Parameters") {
            parts.push(section);
        }
        for ex in &info.examples {
            // every example is a doctest, async defs included: theirs are
            // written through the sync rim, `Task.run(...)` (GEP-0030-R005)
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
        // the docstring is itself a Python string literal: a bare
        // backslash would halve on load, corrupting doctests like
        // re.compile("\\d+") into a SyntaxWarning
        Ok(Some(joined.replace('\\', "\\\\")))
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
                        "for" => return self.emit_for(call, dest, out),
                        "loop" => {
                            return Err(self.err(
                                call.span,
                                "loop was retired (GEP-0014-R007): write a recursive \
                                 helper — `defp step(state) do body end; step(init)`; \
                                 recur(x) stays, break(v) becomes the value v",
                            ))
                        }
                        "break" => {
                            return Err(self.err(
                                call.span,
                                "break was retired with loop (GEP-0014-R007): return \
                                 the value directly from your recursive helper",
                            ))
                        }
                        "recur" => {
                            let Some((_, arities, params)) = self.tail_ctx.clone() else {
                                return Err(self.err(
                                    call.span,
                                    "recur restarts the enclosing function and is only \
                                     valid in a function tail; inside try it cannot be \
                                     optimized (GEP-0014-R005, GEP-0019)",
                                ));
                            };
                            if !matches!(dest, Dest::Return) {
                                return Err(self.err(
                                    call.span,
                                    "recur must be in tail position (GEP-0019-R005)",
                                ));
                            }
                            if !arities.contains(&call.args.len()) {
                                return Err(self.err(
                                    call.span,
                                    format!(
                                        "recur/{} matches no clause of this \
                                         function (GEP-0019-R005)",
                                        call.args.len()
                                    ),
                                ));
                            }
                            let mut pre = Vec::new();
                            let args: Vec<String> = call
                                .args
                                .iter()
                                .map(|a| self.emit_expr(a, &mut pre))
                                .collect::<Result<_>>()?;
                            out.extend(pre);
                            match &params {
                                Some(ps) if ps.len() == args.len() && !ps.is_empty() => {
                                    out.push(format!(
                                        "{} = {}",
                                        ps.join(", "),
                                        args.join(", ")
                                    ));
                                }
                                Some(_) => {}
                                None => {
                                    let trailing = if args.len() == 1 { "," } else { "" };
                                    out.push(format!(
                                        "_gan_args = ({}{trailing})",
                                        args.join(", ")
                                    ));
                                }
                            }
                            out.push("continue".into());
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
                // tail self-call: rebind parameters and continue (GEP-0019-R002)
                if matches!(dest, Dest::Return) {
                    if let Some((tname, arities, params)) = self.tail_ctx.clone() {
                        if let Callee::Name(n) = &call.callee {
                            if *n == tname && arities.contains(&call.args.len()) {
                                let mut pre = Vec::new();
                                let args: Vec<String> = call
                                    .args
                                    .iter()
                                    .map(|a| self.emit_expr(a, &mut pre))
                                    .collect::<Result<_>>()?;
                                out.extend(pre);
                                match &params {
                                    Some(ps) if ps.len() == args.len() && !ps.is_empty() => {
                                        out.push(format!(
                                            "{} = {}",
                                            ps.join(", "),
                                            args.join(", ")
                                        ));
                                    }
                                    Some(_) => {}
                                    None => {
                                        let trailing = if args.len() == 1 { "," } else { "" };
                                        out.push(format!(
                                            "_gan_args = ({}{trailing})",
                                            args.join(", ")
                                        ));
                                    }
                                }
                                out.push("continue".into());
                                return Ok(());
                            }
                        }
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

    /// GEP-0019-R001: does this body end in a tail call to `name`?
    fn has_tail_self(&self, term: &Term, name: &str, arities: &BTreeSet<usize>) -> bool {
        term_has_tail_self(term, name, arities)
    }

    /// Per-function unsafety lints (GEP-0022): undefined variables,
    /// unused bindings, unreachable clauses.
    fn lint_fun(&mut self, f: &FnDef, fn_scope: &BTreeSet<String>, arity_label: &str) {
        let display = if f.name.starts_with("_gan_fn") {
            "this anonymous fn".to_string()
        } else {
            format!("{}/{arity_label}", f.name)
        };
        let mut fn_reads: BTreeSet<String> = BTreeSet::new();
        for (params, guard, body) in &f.clauses {
            for p in params {
                pattern_pin_reads(p, &mut fn_reads);
            }
            if let Some(g) = guard {
                scope_reads(g, &mut fn_reads);
            }
            scope_reads(body, &mut fn_reads);
        }
        // a name bound but never read wants a _ prefix (GEP-0022-R002)
        for name in fn_scope {
            if name.starts_with('_')
                || name.contains("__gan")
                || f.captures.contains(name)
                || fn_reads.contains(name)
            {
                continue;
            }
            self.warnings.push(crate::diag::Diagnostic::new(
                &self.file,
                f.span,
                format!(
                    "variable {name} is bound but never used in {display}; \
                     prefix it with _ to keep the intent visible (GEP-0022-R002)"
                ),
            ));
        }
        // a read nothing binds is a guaranteed NameError (GEP-0022-R001);
        // `import Mod` makes bare names statically unknowable — stand down
        if self.star_imports.is_empty() {
            let mut known: BTreeSet<String> = self
                .local_funs
                .iter()
                .map(|(n, _)| map_ident(n))
                .collect();
            for imp in &self.py_imports {
                match imp.split(" as ").nth(1) {
                    Some(alias) => {
                        known.insert(alias.to_string());
                    }
                    // `pyimport os.path` binds the bare first segment
                    None => {
                        if let Some(first) = imp.split('.').next() {
                            known.insert(first.to_string());
                        }
                    }
                }
            }
            for name in &fn_reads {
                if name.starts_with('_')
                    || name.contains("__gan")
                    || fn_scope.contains(name)
                    || known.contains(name)
                {
                    continue;
                }
                self.warnings.push(crate::diag::Diagnostic::new(
                    &self.file,
                    f.span,
                    format!(
                        "variable {name} is never bound in {display}: this is a \
                         guaranteed NameError at runtime (GEP-0022-R001)"
                    ),
                ));
            }
        }
        // clauses behind a guard-less all-variable head can never match
        // (GEP-0022-R003); compared per arity — other arities still run
        let arities: BTreeSet<usize> =
            f.clauses.iter().map(|(p, _, _)| p.len()).collect();
        for arity in arities {
            let same: Vec<usize> = f
                .clauses
                .iter()
                .enumerate()
                .filter(|(_, (p, _, _))| p.len() == arity)
                .map(|(i, _)| i)
                .collect();
            let catch_all = same.iter().position(|&i| {
                let (params, guard, _) = &f.clauses[i];
                guard.is_none()
                    && params.iter().all(|p| matches!(p, Term::Var(_, _)))
            });
            if let Some(pos) = catch_all {
                if pos + 1 < same.len() {
                    self.warnings.push(crate::diag::Diagnostic::new(
                        &self.file,
                        f.span,
                        format!(
                            "clause {} of {display} already matches every \
                             argument; the {} clause(s) after it can never run \
                             (GEP-0022-R003)",
                            pos + 1,
                            same.len() - pos - 1
                        ),
                    ));
                }
            }
        }
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
        let n_clauses = clauses.len();
        out.push(format!("match {tname}:"));
        let mut any_wild = false;
        let mut arms: Vec<String> = Vec::new();
        for (ci, clause) in clauses.into_iter().enumerate() {
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
                // clauses after this one are dead (GEP-0022-R003)
                if !any_wild && ci + 1 < n_clauses {
                    self.warnings.push(crate::diag::Diagnostic::new(
                        &self.file,
                        c.span,
                        "this case clause matches every value; the clauses \
                         after it can never run (GEP-0022-R003)",
                    ));
                }
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

    /// `for` comprehensions (GEP-0020): generators, filters, into.
    fn emit_for(&mut self, call: &Call, dest: Dest, out: &mut Vec<String>) -> Result<()> {
        let span = call.span;
        if matches!(dest, Dest::Ignore) {
            // building a collection nobody looks at (GEP-0022-R004)
            self.warnings.push(crate::diag::Diagnostic::new(
                &self.file,
                span,
                "the comprehension's result is discarded; `for` builds a \
                 collection — use Enum.each for side effects (GEP-0022-R004)",
            ));
        }
        let mut clauses: Vec<Term> = Vec::new();
        let mut body: Option<Term> = None;
        let mut into: Option<Term> = None;
        for a in &call.args {
            match a {
                Term::Pair(k, v) if k == "do" => body = Some(v.as_ref().clone()),
                Term::Pair(k, v) if k == "into" => into = Some(v.as_ref().clone()),
                Term::Pair(k, _) => {
                    return Err(self.err(span, format!("for does not take {k}: (GEP-0020)")))
                }
                other => clauses.push(other.clone()),
            }
        }
        let body = body.ok_or_else(|| self.err(span, "for requires a do: body (GEP-0020-R001)"))?;
        let body_stmts = body.as_block();
        if body_stmts.len() != 1 {
            return Err(self.err(
                span,
                "a for body must be a single expression (GEP-0020-R001)",
            ));
        }
        let body_expr = &body_stmts[0];

        let mut parts: Vec<String> = Vec::new();
        let mut first_pre: Vec<String> = Vec::new();
        let mut seen_generator = false;
        for c in &clauses {
            let is_gen = matches!(&c,
                Term::Call(cc) if matches!(&cc.callee, Callee::Name(n) if n == "<-"));
            if is_gen {
                let Term::Call(cc) = c else { unreachable!() };
                let pat = &cc.args[0];
                let mut pre = Vec::new();
                let it = self.emit_expr(&cc.args[1], &mut pre)?;
                if !pre.is_empty() {
                    if seen_generator {
                        return Err(self.err(
                            span,
                            "for generators after the first must be simple expressions",
                        ));
                    }
                    first_pre.extend(pre);
                }
                match pat {
                    Term::Var(n, ctx) if n != "_" => {
                        parts.push(format!("for {} in {it}", hygienic_name(n, *ctx)));
                    }
                    Term::Var(_, _) => {
                        let t = self.fresh_tmp("for");
                        parts.push(format!("for {} in {it}", self.tmp(t)));
                    }
                    _ => {
                        let t = self.fresh_tmp("for");
                        let g = self.tmp(t).to_string();
                        parts.push(format!("for {g} in {it}"));
                        let mut guards = Vec::new();
                        let mut binds: Vec<(String, String)> = Vec::new();
                        self.comp_pattern(pat, &g, &mut guards, &mut binds)?;
                        if !guards.is_empty() {
                            parts.push(format!("if {}", guards.join(" and ")));
                        }
                        if !binds.is_empty() {
                            let names: Vec<&str> =
                                binds.iter().map(|(n, _)| n.as_str()).collect();
                            let exprs: Vec<&str> =
                                binds.iter().map(|(_, e)| e.as_str()).collect();
                            parts.push(format!(
                                "for ({},) in [({},)]",
                                names.join(", "),
                                exprs.join(", ")
                            ));
                        }
                    }
                }
                seen_generator = true;
            } else {
                let mut pre = Vec::new();
                let e = self.emit_bool_expr(c, &mut pre)?;
                if !pre.is_empty() {
                    return Err(self.err(span, "for filters must be simple expressions"));
                }
                parts.push(format!("if {e}"));
            }
        }
        if !seen_generator {
            return Err(self.err(span, "for requires at least one `pattern <- enumerable`"));
        }

        let mut pre = Vec::new();
        let comp = match &into {
            None => {
                let elem = self.emit_expr(body_expr, &mut pre)?;
                format!("[{elem} {}]", parts.join(" "))
            }
            Some(Term::Map(entries)) if entries.is_empty() => {
                let Term::Tuple(kv) = body_expr else {
                    return Err(self.err(
                        span,
                        "into: %{} needs a {key, value} body (GEP-0020-R003)",
                    ));
                };
                if kv.len() != 2 {
                    return Err(self.err(
                        span,
                        "into: %{} needs a {key, value} body (GEP-0020-R003)",
                    ));
                }
                let k = self.emit_expr(&kv[0], &mut pre)?;
                let v = self.emit_expr(&kv[1], &mut pre)?;
                format!("{{{k}: {v} {}}}", parts.join(" "))
            }
            Some(_) => {
                return Err(self.err(
                    span,
                    "into: supports %{} in this revision (GEP-0020-R003)",
                ))
            }
        };
        if !pre.is_empty() {
            return Err(self.err(span, "a for body must be a simple expression"));
        }
        out.extend(first_pre);
        self.finish_value(comp, dest, out);
        Ok(())
    }

    /// Structural guard + bindings for a comprehension generator pattern
    /// (GEP-0020-R002): non-matching elements are skipped, never raised.
    fn comp_pattern(
        &mut self,
        pat: &Term,
        subject: &str,
        guards: &mut Vec<String>,
        binds: &mut Vec<(String, String)>,
    ) -> Result<()> {
        match pat {
            Term::Var(n, _) if n == "_" || n.starts_with('_') => Ok(()),
            Term::Var(n, ctx) => {
                binds.push((hygienic_name(n, *ctx), subject.to_string()));
                Ok(())
            }
            Term::Int(v) => {
                guards.push(format!("{subject} == {v}"));
                Ok(())
            }
            Term::Atom(a) => {
                guards.push(format!("{subject} == {}", py_str_lit(a)));
                Ok(())
            }
            Term::Bool(b) => {
                guards.push(format!("{subject} is {}", if *b { "True" } else { "False" }));
                Ok(())
            }
            Term::Nil => {
                guards.push(format!("{subject} is None"));
                Ok(())
            }
            Term::Str(parts) => match Term::Str(parts.clone()).as_plain_str() {
                Some(text) => {
                    guards.push(format!("{subject} == {}", py_str_lit(&text)));
                    Ok(())
                }
                None => Err(self.err(
                    Span::default(),
                    "interpolated strings are not comprehension patterns (GEP-0020-R002)",
                )),
            },
            Term::Tuple(items) => {
                guards.push(format!(
                    "isinstance({subject}, tuple) and len({subject}) == {}",
                    items.len()
                ));
                for (i, item) in items.iter().enumerate() {
                    self.comp_pattern(item, &format!("{subject}[{i}]"), guards, binds)?;
                }
                Ok(())
            }
            Term::List(items) => {
                guards.push(format!(
                    "isinstance({subject}, list) and len({subject}) == {}",
                    items.len()
                ));
                for (i, item) in items.iter().enumerate() {
                    self.comp_pattern(item, &format!("{subject}[{i}]"), guards, binds)?;
                }
                Ok(())
            }
            other => Err(self.err(
                other.span(),
                "this pattern is not supported in a comprehension generator \
                 (GEP-0020-R002)",
            )),
        }
    }

    fn emit_try(&mut self, call: &Call, dest: Dest, out: &mut Vec<String>) -> Result<()> {
        let saved_tail = self.tail_ctx.take();
        let r = self.emit_try_inner(call, dest, out);
        self.tail_ctx = saved_tail;
        return r;
    }

    fn emit_try_inner(&mut self, call: &Call, dest: Dest, out: &mut Vec<String>) -> Result<()> {
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
                        | "is_nil" | "is_list" | "is_map" | "is_tuple" | "is_atom"
                        | "is_binary" | "is_boolean" | "is_integer" | "is_float"
                        | "is_function"
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
            Term::PyRef(m, _) => {
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
    /// The importable Python path of a Gandora module: project prefix
    /// and installed markers respected (GEP-0006/GEP-0010).
    fn resolve_module_path(&self, resolved: &[String]) -> String {
        let joined = resolved.join(".");
        if self.project_modules.contains(&joined) {
            match &self.py_prefix {
                Some(prefix) => format!("{prefix}.{}", module_py_path(resolved)),
                None => module_py_path(resolved),
            }
        } else if let Some(installed) = self.installed_modules.get(&joined) {
            installed.clone()
        } else {
            module_py_path(resolved)
        }
    }

    fn gan_module_import(&mut self, resolved: &[String]) -> String {
        let path = self.resolve_module_path(resolved);
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
                    // $mod.sub.Type.member chains: the lowercase prefix is the
                    // module path, imported whole (GEP-0003-R010)
                    chain_base if pyref_chain(chain_base).is_some() => {
                        let (mut segs, bounded) = pyref_chain(chain_base).unwrap();
                        segs.push(name.clone());
                        self.py_imports.insert(pyref_import_path(&segs, bounded));
                        let mut path: Vec<String> = segs[..segs.len() - 1].to_vec();
                        path.push(self.attr_name(name, span)?);
                        let path = path.join(".");
                        if *is_call {
                            let args = self.emit_args(&call.args, pre)?;
                            Ok(format!("{path}({args})"))
                        } else {
                            Ok(path)
                        }
                    }
                    // $module.fun(...) — remote reference (GEP-0003-R001/R002)
                    Term::PyRef(module, _) => {
                        self.py_imports.insert(module.clone());
                        let f = self.attr_name(name, span)?;
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
                        let f = self.attr_name(name, span)?;
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
            ("await", 1) => {
                // native await (GEP-0030-R002): bare, no deadline, no
                // wrapper — and only where Python itself allows it
                if !self.async_ctx {
                    return Err(self.err(
                        call.span,
                        "await is legal only in the body of an async def — \
                         a fn closure is synchronous and cannot await \
                         (GEP-0030-R002/R003)",
                    ));
                }
                let e = self.emit_expr(&args[0], pre)?;
                Ok(format!("(await {e})"))
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
            ("$python", 1) => {
                // embedded Python: verbatim code with <%= %> code splices
                // (GEP-0005-R007, GEP-0009-R003)
                let body = args[0].as_plain_str().ok_or_else(|| {
                    self.err(call.span, "$python bodies are raw and cannot interpolate")
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
                                        format!("in $python splice: {}", d.message);
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
                    return Err(self.err(call.span, "$python requires a Python expression"));
                }
                Ok(format!("({body})"))
            }
            (sigil, 1) if sigil.starts_with('~') => {
                // sigil name discipline (GEP-0005 rev 3): short names are
                // the functional whitelist; a language tag is 3+ chars
                let name = &sigil[1..];
                if name.chars().count() <= 2 && !matches!(name, "w" | "s" | "r" | "p") {
                    return Err(self.err(
                        call.span,
                        format!(
                            "unknown sigil ~{name} — functional sigils are \
                             ~w ~s ~r ~p; language-tagged text sigils use \
                             names of 3+ characters (GEP-0005-R010)"
                        ),
                    ));
                }
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
            ("recur", _) => Err(self.err(
                call.span,
                "recur restarts the enclosing function and must be the whole \
                 tail expression, not part of a larger one (GEP-0019-R005)",
            )),
            ("break", _) => Err(self.err(
                call.span,
                "break was retired with loop (GEP-0014-R007): return the value \
                 directly from your recursive helper",
            )),
            ("if", _) | ("unless", _) | ("case", _) | ("cond", _) | ("with", _) | ("try", _) | ("for", _)
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
                    "defstruct", "defprotocol", "defimpl", "receive", "send",
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

    /// Whether `e` is one parenthesized unit — `(a or b)` is, while
    /// `(a) or (b)` merely *starts* with a paren. The old starts_with
    /// check let the latter through unwrapped, so `(a or b) and c`
    /// compiled to `(a) or (b) and (c)`, which Python reads as
    /// `a or (b and c)` — a real miscompile.
    fn fully_parenthesized(e: &str) -> bool {
        if !e.starts_with('(') || !e.ends_with(')') {
            return false;
        }
        let mut depth = 0usize;
        for (i, ch) in e.char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return i == e.len() - 1;
                    }
                }
                _ => {}
            }
        }
        false
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
        if atomic || Self::fully_parenthesized(&e) {
            Ok(e)
        } else {
            Ok(format!("({e})"))
        }
    }

    fn emit_fn(&mut self, call: &Call, pre: &mut Vec<String>) -> Result<String> {
        // a closure body is a sync function even inside an async def:
        // `await` may not cross a lambda boundary (GEP-0030-R003)
        let saved_async = self.async_ctx;
        self.async_ctx = false;
        let out = self.emit_fn_inner(call, pre);
        self.async_ctx = saved_async;
        out
    }

    fn emit_fn_inner(&mut self, call: &Call, pre: &mut Vec<String>) -> Result<String> {
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
                            let caps = self.closure_captures(&fn_free_vars(clauses));
                            return Ok(format!(
                                "lambda {}: {e}",
                                lambda_params(&names, &caps)
                            ));
                        }
                    }
                }
            }
        }
        // otherwise hoist a def
        let fname = format!("_gan_fn{}", self.fn_counter);
        self.fn_counter += 1;
        let captures = self.closure_captures(&fn_free_vars(clauses));
        let fdef = FnDef {
            spec: None,
            name: fname.clone(),
            private: false,
            doc: None,
            decorators: Vec::new(),
            captures,
            allows: BTreeSet::new(),
            defaults: Vec::new(),
            is_async: false,
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
                if let Term::Int(arity) = c.args[1] {
                    let arity = arity as usize;
                    return match &c.args[0] {
                        // kernel forms inline at call sites — a capture
                        // needs the Python callable itself
                        Term::Var(n, _) if n == "to_string" && arity == 1 => {
                            Ok("str".to_string())
                        }
                        Term::Var(n, _) if n == "inspect" && arity == 1 => {
                            Ok("repr".to_string())
                        }
                        // local captures resolve like call sites: a defp
                        // compiles under its private name
                        Term::Var(n, _) => {
                            let mut f = map_ident(n);
                            if self.private_funs.contains(&(n.clone(), arity)) {
                                f.insert(0, '_');
                            }
                            Ok(f)
                        }
                        // an unquoted atom names the function — the
                        // GEP-0008-R002 def-head coercion, in capture
                        // position: `&unquote(name)/1` from a hook
                        Term::Atom(n) => {
                            let mut f = map_ident(n);
                            if self.private_funs.contains(&(n.clone(), arity)) {
                                f.insert(0, '_');
                            }
                            Ok(f)
                        }
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
        let mut reads = BTreeSet::new();
        scope_reads(&renamed, &mut reads);
        for p in &params {
            reads.remove(p);
        }
        let caps = self.closure_captures(&reads);
        Ok(format!("lambda {}: {e}", lambda_params(&params, &caps)))
    }

    /// The enclosing locals among a closure's free variables — the names
    /// whose creation-time values the closure must snapshot (GEP-0021).
    fn closure_captures(&self, free: &BTreeSet<String>) -> Vec<String> {
        match self.scope_bound.last() {
            Some(bound) => free.intersection(bound).cloned().collect(),
            None => Vec::new(),
        }
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
            Term::PyRef(..) => Err(self.err(
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

/// Whether a tail position of `term` calls `name` with an arity in
/// `arities` (or spells `recur`) — the test that turns a definition
/// group into a `while True:` loop (GEP-0019-R001/R002).
pub fn term_has_tail_self(term: &Term, name: &str, arities: &BTreeSet<usize>) -> bool {
    let stmts = term.as_block();
    let Some(last) = stmts.last() else { return false };
    let Term::Call(c) = last else { return false };
    match &c.callee {
        Callee::Name(n) if n == name && arities.contains(&c.args.len()) => true,
        Callee::Name(n) if n == "recur" => true,
        Callee::Name(n) => match n.as_str() {
            "if" | "unless" | "with" => ["do", "else"].iter().any(|k| {
                Term::keyword_arg(&c.args, k)
                    .is_some_and(|b| term_has_tail_self(b, name, arities))
            }),
            "case" | "cond" => Term::keyword_arg(&c.args, "do")
                .is_some_and(|b| match b {
                    Term::Call(cl)
                        if matches!(&cl.callee, Callee::Name(x) if x == "__clauses__") =>
                    {
                        cl.args.iter().any(|clause| match clause {
                            Term::Call(cc) => cc
                                .args
                                .last()
                                .is_some_and(|b| term_has_tail_self(b, name, arities)),
                            _ => false,
                        })
                    }
                    _ => false,
                }),
            _ => false,
        },
        _ => false,
    }
}

/// Whether `term` mentions `name` at all — as a callee (any arity) or a
/// bare reference (`&name/1` targets included) (GEP-0022-R005).
fn name_referenced(term: &Term, name: &str) -> bool {
    match term {
        Term::Var(n, _) => n == name,
        Term::Str(parts) => parts.iter().any(|p| match p {
            StrPart::Interp(t) => name_referenced(t, name),
            _ => false,
        }),
        Term::List(items) | Term::Tuple(items) => {
            items.iter().any(|t| name_referenced(t, name))
        }
        Term::Map(entries) => entries
            .iter()
            .any(|(k, v)| name_referenced(k, name) || name_referenced(v, name)),
        Term::Pair(_, v) => name_referenced(v, name),
        Term::Call(c) => {
            // `&unquote(name)/n` captures through an atom
            // (GEP-0008-R002 coercion) — that atom names a function
            if matches!(&c.callee, Callee::Name(n) if n == "/") {
                if let Some(Term::Atom(a)) = c.args.first() {
                    if a == name {
                        return true;
                    }
                }
            }
            let own = match &c.callee {
                Callee::Name(n) => n == name,
                Callee::Dot { base, .. } => name_referenced(base, name),
                Callee::Apply(t) => name_referenced(t, name),
            };
            own || c.args.iter().any(|t| name_referenced(t, name))
        }
        _ => false,
    }
}

/// Whether `term` calls `name` with an arity in `arities` anywhere.
fn calls_self(term: &Term, name: &str, arities: &BTreeSet<usize>) -> bool {
    match term {
        Term::Str(parts) => parts.iter().any(|p| match p {
            StrPart::Interp(t) => calls_self(t, name, arities),
            _ => false,
        }),
        Term::List(items) | Term::Tuple(items) => {
            items.iter().any(|t| calls_self(t, name, arities))
        }
        Term::Map(entries) => entries
            .iter()
            .any(|(k, v)| calls_self(k, name, arities) || calls_self(v, name, arities)),
        Term::Pair(_, v) => calls_self(v, name, arities),
        Term::Call(c) => {
            let own = match &c.callee {
                Callee::Name(n) => n == name && arities.contains(&c.args.len()),
                Callee::Dot { base, .. } => calls_self(base, name, arities),
                Callee::Apply(t) => calls_self(t, name, arities),
            };
            own || c.args.iter().any(|t| calls_self(t, name, arities))
        }
        _ => false,
    }
}

/// How a definition group compiles (GEP-0019-R006): `Some("loop")` when
/// tail self-recursion collapses to `while True:` rebinding,
/// `Some("stack")` when the group is self-recursive only outside tail
/// position (native call stack), `None` when it is not self-recursive.
pub fn recursion_shape(
    name: &str,
    arities: &BTreeSet<usize>,
    bodies: &[&Term],
) -> Option<&'static str> {
    if bodies.iter().any(|b| term_has_tail_self(b, name, arities)) {
        Some("loop")
    } else if bodies.iter().any(|b| calls_self(b, name, arities)) {
        Some("stack")
    } else {
        None
    }
}

/// Parameter list of a generated lambda: declared params, then a bare
/// `*` guarding keyword-only creation-time snapshots (GEP-0021-R002 —
/// an extra positional argument must still raise, never bind a snapshot).
fn lambda_params(params: &[String], caps: &[String]) -> String {
    let snaps: Vec<String> = caps.iter().map(|c| format!("{c}={c}")).collect();
    match (params.is_empty(), snaps.is_empty()) {
        (_, true) => params.join(", "),
        (true, false) => format!("*, {}", snaps.join(", ")),
        (false, false) => format!("{}, *, {}", params.join(", "), snaps.join(", ")),
    }
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

/// Hygienic names a pattern binds; pins (`^x`) reference, never bind.
fn pattern_binds(pat: &Term, out: &mut BTreeSet<String>) {
    match pat {
        Term::Var(n, ctx) => {
            let h = hygienic_name(n, *ctx);
            if h != "_" {
                out.insert(h);
            }
        }
        Term::List(items) | Term::Tuple(items) => {
            items.iter().for_each(|p| pattern_binds(p, out));
        }
        Term::Map(entries) => entries.iter().for_each(|(_, v)| pattern_binds(v, out)),
        Term::Pair(_, v) => pattern_binds(v, out),
        Term::Call(c) => match &c.callee {
            Callee::Name(n) if n == "^" => {}
            Callee::Name(n) if n == "when" => {
                if let Some(p) = c.args.first() {
                    pattern_binds(p, out);
                }
            }
            _ => c.args.iter().for_each(|p| pattern_binds(p, out)),
        },
        _ => {}
    }
}

/// Var reads a pattern performs: pin (`^x`) targets only.
fn pattern_pin_reads(pat: &Term, out: &mut BTreeSet<String>) {
    match pat {
        Term::Call(c) if matches!(&c.callee, Callee::Name(n) if n == "^") => {
            c.args.iter().for_each(|a| scope_reads(a, out));
        }
        Term::List(items) | Term::Tuple(items) => {
            items.iter().for_each(|p| pattern_pin_reads(p, out));
        }
        Term::Map(entries) => entries.iter().for_each(|(k, v)| {
            pattern_pin_reads(k, out);
            pattern_pin_reads(v, out);
        }),
        Term::Pair(_, v) => pattern_pin_reads(v, out),
        Term::Call(c) => c.args.iter().for_each(|p| pattern_pin_reads(p, out)),
        _ => {}
    }
}

/// Hygienic names a term binds in the enclosing function scope: `=` and
/// `<-` left sides plus `->` clause patterns, which all compile to Python
/// assignments. `fn` bodies are their own scope; `cond` clause heads are
/// conditions, not patterns.
fn scope_binders(term: &Term, out: &mut BTreeSet<String>) {
    match term {
        Term::Str(parts) => {
            for p in parts {
                if let StrPart::Interp(t) = p {
                    scope_binders(t, out);
                }
            }
        }
        Term::List(items) | Term::Tuple(items) => {
            items.iter().for_each(|t| scope_binders(t, out));
        }
        Term::Map(entries) => entries.iter().for_each(|(k, v)| {
            scope_binders(k, out);
            scope_binders(v, out);
        }),
        Term::Pair(_, v) => scope_binders(v, out),
        Term::Call(c) => {
            match &c.callee {
                // fn bodies are their own scope; quote bodies are data
                Callee::Name(n) if n == "fn" || n == "quote" => return,
                Callee::Name(n) if (n == "=" || n == "<-") && c.args.len() == 2 => {
                    pattern_binds(&c.args[0], out);
                    scope_binders(&c.args[1], out);
                    return;
                }
                Callee::Name(n) if n == "->" && c.args.len() == 2 => {
                    if let Term::List(pats) = &c.args[0] {
                        for p in pats {
                            pattern_binds(split_when(p).0, out);
                        }
                    }
                    scope_binders(&c.args[1], out);
                    return;
                }
                Callee::Name(n) if n == "cond" => {
                    if let Some(Term::Call(cl)) = Term::keyword_arg(&c.args, "do") {
                        for clause in &cl.args {
                            if let Term::Call(arrow) = clause {
                                if let Some(body) = arrow.args.last() {
                                    scope_binders(body, out);
                                }
                            }
                        }
                    }
                    return;
                }
                Callee::Dot { base, .. } => scope_binders(base, out),
                Callee::Apply(t) => scope_binders(t, out),
                Callee::Name(_) => {}
            }
            c.args.iter().for_each(|t| scope_binders(t, out));
        }
        _ => {}
    }
}

/// Every hygienic `Var` reference a term reads. Pattern binders show up
/// too — callers subtract the binder set, so a name is a capture only
/// when nothing in the scope binds it. Nested `fn` scopes contribute
/// their own free vars.
fn scope_reads(term: &Term, out: &mut BTreeSet<String>) {
    match term {
        Term::Var(n, ctx) => {
            let h = hygienic_name(n, *ctx);
            if h != "_" && !n.starts_with('&') {
                out.insert(h);
            }
        }
        Term::Str(parts) => {
            for p in parts {
                if let StrPart::Interp(t) = p {
                    scope_reads(t, out);
                }
            }
        }
        Term::List(items) | Term::Tuple(items) => {
            items.iter().for_each(|t| scope_reads(t, out));
        }
        Term::Map(entries) => entries.iter().for_each(|(k, v)| {
            scope_reads(k, out);
            scope_reads(v, out);
        }),
        Term::Pair(_, v) => scope_reads(v, out),
        Term::Call(c) => {
            match &c.callee {
                Callee::Name(n) if n == "fn" => {
                    out.extend(fn_free_vars(&c.args));
                    return;
                }
                // &name/arity references a function, not a variable
                Callee::Name(n) if n == "&" && c.args.len() == 1 => {
                    if let Some(Term::Call(inner)) = c.args.first() {
                        if matches!(&inner.callee, Callee::Name(x) if x == "/")
                            && inner.args.len() == 2
                            && matches!(inner.args[1], Term::Int(_))
                        {
                            return;
                        }
                    }
                }
                // pattern sides read only through pins — a binding
                // occurrence is not a use (GEP-0022-R002)
                Callee::Name(n) if (n == "=" || n == "<-") && c.args.len() == 2 => {
                    pattern_pin_reads(&c.args[0], out);
                    scope_reads(&c.args[1], out);
                    return;
                }
                Callee::Name(n) if n == "->" && c.args.len() == 2 => {
                    if let Term::List(pats) = &c.args[0] {
                        for p in pats {
                            let (pat, guard) = split_when(p);
                            pattern_pin_reads(pat, out);
                            if let Some(g) = guard {
                                scope_reads(g, out);
                            }
                        }
                    } else {
                        scope_reads(&c.args[0], out);
                    }
                    scope_reads(&c.args[1], out);
                    return;
                }
                // cond clause heads are conditions, not patterns
                Callee::Name(n) if n == "cond" => {
                    if let Some(Term::Call(cl)) = Term::keyword_arg(&c.args, "do") {
                        for clause in &cl.args {
                            if let Term::Call(arrow) = clause {
                                for a in &arrow.args {
                                    scope_reads(a, out);
                                }
                            }
                        }
                    }
                    return;
                }
                // sigil templates read through `<%= expr %>` splices
                // (GEP-0009-R002); $python code splices read too
                Callee::Name(n) if n.starts_with('~') || n == "$python" => {
                    for a in &c.args {
                        match a.as_plain_str() {
                            Some(body) => {
                                for part in split_splices(&body) {
                                    if let SplicePart::Expr(src) = part {
                                        if let Ok(t) = crate::parser::parse_expr_str(
                                            "<lint>", &src,
                                        ) {
                                            scope_reads(&t, out);
                                        }
                                    }
                                }
                            }
                            None => scope_reads(a, out),
                        }
                    }
                    return;
                }
                Callee::Dot { base, .. } => scope_reads(base, out),
                Callee::Apply(t) => scope_reads(t, out),
                Callee::Name(_) => {}
            }
            c.args.iter().for_each(|t| scope_reads(t, out));
        }
        _ => {}
    }
}

/// Free variables of an `fn`: everything its clauses read minus
/// everything they bind (params, `=`/`->` patterns). What remains
/// resolves in the enclosing function and must be snapshot-captured
/// (GEP-0021-R001).
fn fn_free_vars(clauses: &[Term]) -> BTreeSet<String> {
    let mut reads = BTreeSet::new();
    let mut binds = BTreeSet::new();
    for clause in clauses {
        let Term::Call(c) = clause else { continue };
        if c.args.len() != 2 {
            continue;
        }
        if let Term::List(pats) = &c.args[0] {
            for p in pats {
                let (pat, guard) = split_when(p);
                pattern_binds(pat, &mut binds);
                scope_reads(pat, &mut reads);
                if let Some(g) = guard {
                    scope_reads(g, &mut reads);
                }
            }
        }
        scope_binders(&c.args[1], &mut binds);
        scope_reads(&c.args[1], &mut reads);
    }
    &reads - &binds
}

/// Every variable bound in a parameter pattern (GEP-0018-R002).
fn collect_vars(term: &Term, out: &mut BTreeSet<String>) {
    match term {
        Term::Var(n, _) => {
            if !n.starts_with('_') {
                out.insert(n.clone());
            }
        }
        Term::List(items) | Term::Tuple(items) => {
            for i in items {
                collect_vars(i, out);
            }
        }
        Term::Map(entries) => {
            for (_, v) in entries {
                collect_vars(v, out);
            }
        }
        Term::Call(c) => {
            for a in &c.args {
                collect_vars(a, out);
            }
        }
        _ => {}
    }
}

/// The generated `## Parameters` section for one locale, or None when the
/// definition has no @param docs (GEP-0018-R004).
pub fn params_section(info: &DocInfo, locale: &str, heading: &str) -> Option<String> {
    if info.params.is_empty() {
        return None;
    }
    let mut lines = vec![heading.to_string(), String::new()];
    for (name, entries) in &info.params {
        let text = entries
            .iter()
            .find(|(l, _)| l == locale)
            .or_else(|| entries.iter().find(|(l, _)| l == "default"))
            .map(|(_, t)| t.as_str())
            .unwrap_or("");
        lines.push(format!("  - {name}: {text}"));
    }
    Some(lines.join("\n"))
}


/// Fold a `$mod.seg.seg...` attribute chain rooted at a PyRef.
/// Returns (segments) when the whole chain is plain attribute access.
fn pyref_chain(term: &Term) -> Option<(Vec<String>, bool)> {
    match term {
        Term::PyRef(m, bounded) => Some((vec![m.clone()], *bounded)),
        Term::Call(c) => match &c.callee {
            Callee::Dot {
                base,
                name,
                is_call: false,
            } if c.args.is_empty() => {
                let (mut segs, bounded) = pyref_chain(base)?;
                segs.push(name.clone());
                Some((segs, bounded))
            }
            _ => None,
        },
        _ => None,
    }
}

/// The import path for a folded chain (GEP-0003-R010): the leading run of
/// lowercase segments, stopping before the final segment — dotted
/// submodules import whole, members resolve as attributes.

/// A directed correction for a misspelled spec type — the errors an
/// agent guessing from other languages actually makes (GEP-0017-R002).
/// Small edit-distance closeness for did-you-mean hints (≤2 edits).
fn strsim_close(a: &str, b: &str) -> bool {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    if a.len().abs_diff(b.len()) > 2 {
        return false;
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut cur = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            cur.push((prev[j] + cost).min(prev[j + 1] + 1).min(cur[j] + 1));
        }
        prev = cur;
    }
    prev[b.len()] <= 2
}

/// Whether `name` is one of the built-in spec types (GEP-0017).
fn spec_builtin_type(name: &str) -> bool {
    matches!(
        name,
        "integer"
            | "float"
            | "number"
            | "boolean"
            | "string"
            | "atom"
            | "any"
            | "term"
            | "list"
            | "map"
            | "tuple"
            | "fun"
            | "keyword"
            | "iterable"
            | "sequence"
            | "mapping"
    )
}

/// Bare 1–2 letter variables of a type body (GEP-0027-R002).
fn type_body_vars(t: &Term, out: &mut BTreeSet<String>) {
    match t {
        Term::Var(v, _) if v.chars().count() <= 2 => {
            out.insert(v.clone());
        }
        Term::Call(c) => {
            for a in &c.args {
                type_body_vars(a, out);
            }
        }
        _ => {}
    }
}

/// Substitute declared type parameters with argument terms.
fn substitute_type_vars(t: &Term, params: &[String], args: &[Term]) -> Term {
    match t {
        Term::Var(v, _) => match params.iter().position(|p| p == v) {
            Some(i) => args[i].clone(),
            None => t.clone(),
        },
        Term::Call(c) => {
            let mut cc = (**c).clone();
            cc.args = cc
                .args
                .iter()
                .map(|a| substitute_type_vars(a, params, args))
                .collect();
            Term::Call(Box::new(cc))
        }
        other => other.clone(),
    }
}

/// The dotted-base text for a spec-type error message (`App.Shop`,
/// `$math`), best-effort.
fn spec_dot_base_text(base: &Term) -> String {
    match base {
        Term::Alias(segs) => segs.join("."),
        Term::PyRef(m, _) => format!("${m}"),
        _ => "Mod".to_string(),
    }
}

fn spec_type_suggestion(name: &str) -> Option<&'static str> {
    match name.to_ascii_lowercase().as_str() {
        "int" | "integer" => Some("integer()"),
        "str" | "string" | "binary" => Some("string()"),
        "float" | "double" => Some("float()"),
        "bool" | "boolean" => Some("boolean()"),
        "number" | "num" => Some("number()"),
        "atom" | "symbol" => Some("atom()"),
        "any" | "term" | "object" => Some("term()"),
        "none" | "nil" | "null" | "void" | "ok" => Some("nil (or atom() for a named atom)"),
        "list" | "array" => Some("list(term())"),
        "map" | "dict" => Some("map(term(), term())"),
        "tuple" => Some("tuple(term(), term())"),
        "fun" | "function" | "callable" | "lambda" => Some("fun()"),
        _ => None,
    }
}

fn pyref_import_path(segs: &[String], bounded: bool) -> String {
    // `$(...)` declares the boundary explicitly — single-segment or
    // dotted, the heuristic must not extend it (GEP-0003-R010)
    if bounded || segs[0].contains('.') {
        return segs[0].clone();
    }
    let mut end = 1;
    while end < segs.len() - 1
        && segs[end]
            .chars()
            .next()
            .is_some_and(|c| c.is_lowercase() || c == '_')
    {
        end += 1;
    }
    segs[..end].join(".")
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
    fn named_types_declare_and_expand() {
        // GEP-0027: @type is the declaration site of generics
        let py = compile(
            "defmodule M do\n  @type result(t) :: tuple(atom(), t)\n  @type age() :: integer()\n  @type scores() :: map(string(), age())\n  @spec f(string()) :: result(integer())\n  def f(_s), do: {:ok, 1}\n  @spec g() :: scores()\n  def g(), do: %{}\nend",
        );
        assert!(py.contains("-> tuple[str, int]:"), "{py}");
        assert!(py.contains("-> dict[str, int]:"), "{py}");
    }

    #[test]
    fn named_type_errors_teach() {
        let m1 = compile_err(
            "defmodule M do\n  @type opt(t) :: t | u\n  def f(), do: nil\nend",
        );
        assert!(m1.contains("`u` is not declared"), "{m1}");
        let m2 = compile_err(
            "defmodule M do\n  @type a1(t) :: list(t)\n  @spec f(a1(integer(), string())) :: term()\n  def f(_x), do: nil\nend",
        );
        assert!(m2.contains("takes 1 type parameter"), "{m2}");
        let m3 = compile_err(
            "defmodule M do\n  @type loop() :: loop()\n  @spec f(loop()) :: term()\n  def f(_x), do: nil\nend",
        );
        assert!(m3.contains("recursive named types"), "{m3}");
        let m4 = compile_err(
            "defmodule M do\n  @type t() :: integer()\n  def f(), do: nil\nend",
        );
        assert!(m4.contains("retired"), "{m4}");
        let m5 = compile_err(
            "defmodule M do\n  @type list() :: integer()\n  def f(), do: nil\nend",
        );
        assert!(m5.contains("shadows the built-in"), "{m5}");
        let m6 = compile_err(
            "defmodule M do\n  @type resul(t) :: list(t)\n  @spec f(result(integer())) :: term()\n  def f(_x), do: nil\nend",
        );
        assert!(m6.contains("did you mean resul()"), "{m6}");
    }

    #[test]
    fn spec_types_are_calls() {
        // GEP-0017 rev 5: the module IS the type; every misspelling
        // teaches the call form
        let py = compile(
            "defmodule M do\n  @spec f(App.Shop()) :: App.Shop()\n  def f(x), do: x\nend",
        );
        assert!(py.contains("app.shop.Shop"), "{py}");
        let m1 = compile_err(
            "defmodule M do\n  @spec f(App.Shop.t()) :: term()\n  def f(_x), do: nil\nend",
        );
        assert!(m1.contains("write App.Shop()"), "{m1}");
        let m2 = compile_err(
            "defmodule M do\n  @spec f($math.Pi) :: term()\n  def f(_x), do: nil\nend",
        );
        assert!(m2.contains("$math.Pi()"), "{m2}");
        let m3 = compile_err(
            "defmodule M do\n  @spec f(App.Shop(a)) :: term()\n  def f(_x), do: nil\nend",
        );
        assert!(m3.contains("no parameters"), "{m3}");
        let m4 = compile_err(
            "defmodule M do\n  @spec f(App.Shop) :: term()\n  def f(_x), do: nil\nend",
        );
        assert!(m4.contains("write App.Shop()"), "{m4}");
    }

    #[test]
    fn short_sigil_names_are_a_whitelist() {
        let msg = compile_err("defmodule M do\n  def f(), do: ~q(hello)\nend");
        assert!(msg.contains("~w ~s ~r ~p"), "{msg}");
        // ~p is the blessed prompt sigil: raw text
        let py = compile("defmodule M do\n  def f(), do: ~p(raw \"x\" {y})\nend");
        assert!(py.contains("raw \\\"x\\\" {y}"), "{py}");
    }

    #[test]
    fn tilde_names_are_uniformly_text_now() {
        // `~` has one semantic: text — even python/json as names are
        // just language tags for a string body (GEP-0009 rev 5)
        let py = compile("defmodule M do\n  def f(x), do: ~python(x + 1)\nend");
        assert!(py.contains("\"x + 1\""), "{py}");
        let py2 = compile("defmodule M do\n  def f(), do: ~json([1])\nend");
        assert!(py2.contains("\"[1]\""), "{py2}");
    }

    #[test]
    fn soft_keyword_attributes_and_bindings_stay_unmangled() {
        // `match` is a Python soft keyword: legal as a method name and
        // as a binding — never rename it (the old `__kw` suffix broke
        // `pattern.match(line)` at runtime)
        let py = compile(
            "defmodule M do\n  def f(line) do\n    match = ~r/a/.match(line)\n    match\n  end\nend",
        );
        assert!(py.contains(".match(line)"), "{py}");
        assert!(!py.contains("match__kw"), "{py}");
    }

    #[test]
    fn hard_keyword_attributes_error_with_a_getattr_recipe() {
        let msg = compile_err("defmodule M do\n  def f(x), do: x.import()\nend");
        assert!(msg.contains("getattr"), "{msg}");
    }

    #[test]
    fn hard_keyword_bindings_still_mangle() {
        let py = compile("defmodule M do\n  def f() do\n    class = 1\n    class\n  end\nend");
        assert!(py.contains("class__kw"), "{py}");
    }

    #[test]
    fn captures_resolve_private_names_and_kernel_forms() {
        let py = compile(
            "defmodule M do\n  def run(xs), do: Enum.map(xs, &tag/1)\n  def strs(xs), do: Enum.map(xs, &to_string/1)\n  def reprs(xs), do: Enum.map(xs, &inspect/1)\n  defp tag(x), do: {:ok, x}\nend",
        );
        assert!(py.contains("_tag"), "defp capture must use the private name: {py}");
        assert!(!py.contains(", tag)"), "unmangled defp capture leaked: {py}");
        assert!(py.contains("str)") || py.contains("str,"), "&to_string/1 must become str: {py}");
        assert!(py.contains("repr)") || py.contains("repr,"), "&inspect/1 must become repr: {py}");
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
    fn params_render_into_the_docstring() {
        // GEP-0018-R001/R004
        let py = compile(
            "defmodule M do\n  @param a, \"the left side\"\n  @param b, \"the right side\"\n  @doc \"Adds.\"\n  def add(a, b), do: a + b\nend",
        );
        assert!(py.contains("## Parameters"), "{py}");
        assert!(py.contains("- a: the left side"), "{py}");
        assert!(py.contains("- b: the right side"), "{py}");
    }

    #[test]
    fn param_validation_errors() {
        // unknown name (GEP-0018-R002)
        let err = compile_err(
            "defmodule M do\n  @param nope, \"x\"\n  def f(a), do: a\nend",
        );
        assert!(err.contains("GEP-0018-R002"), "{err}");
        // duplicate (R001)
        let err2 = compile_err(
            "defmodule M do\n  @param a, \"x\"\n  @param a, \"y\"\n  def f(a), do: a\nend",
        );
        assert!(err2.contains("GEP-0018-R001"), "{err2}");
        // orphan translation (R003)
        let err3 = compile_err(
            "defmodule M do\n  @param_trans a, zh_CN: \"x\"\n  def f(a), do: a\nend",
        );
        assert!(err3.contains("GEP-0018-R003"), "{err3}");
    }

    #[test]
    fn comprehensions_compile_to_native_comprehensions() {
        // GEP-0020-R001/R003
        let py = compile(
            "defmodule M do\n  def f(xs), do: for x <- xs, x > 1, do: x * 10\nend",
        );
        assert!(py.contains("[x * 10 for x in xs if _gan_truthy(x > 1)]")
            || py.contains("[x * 10 for x in xs if x > 1]"), "{py}");
        let py2 = compile(
            "defmodule M do\n  def g(xs), do: for {k, v} <- xs, into: %{}, do: {k, v}\nend",
        );
        assert!(py2.contains("isinstance("), "{py2}");
        assert!(py2.contains("{"), "{py2}");
    }

    #[test]
    fn comprehension_pattern_skips_not_raises() {
        // GEP-0020-R002
        let py = compile(
            "defmodule M do\n  def f(xs), do: for {a, b} <- xs, do: a + b\nend",
        );
        assert!(py.contains("isinstance(") && py.contains("len("), "{py}");
        assert!(!py.contains("GanMatchError\" + repr"), "{py}");
    }

    #[test]
    fn loop_and_break_are_retired() {
        // GEP-0014-R007
        let err = compile_err(
            "defmodule M do\n  def f() do\n    loop n = 0 do\n      break(n)\n    end\n  end\nend",
        );
        assert!(err.contains("GEP-0014-R007"), "{err}");
    }

    #[test]
    fn tail_self_calls_compile_to_loops() {
        // GEP-0019-R002: dispatcher form
        let py = compile(
            "defmodule M do\n  def sum_to(0, acc), do: acc\n  def sum_to(n, acc), do: sum_to(n - 1, acc + n)\nend",
        );
        assert!(py.contains("while True:"), "{py}");
        assert!(py.contains("_gan_args = (n - 1, acc + n)"), "{py}");
        assert!(py.contains("continue"), "{py}");
        // simple form under an if branch
        let py2 = compile(
            "defmodule M do\n  def go(n) do\n    if n <= 0 do\n      :done\n    else\n      go(n - 1)\n    end\n  end\nend",
        );
        assert!(py2.contains("n = n - 1"), "{py2}");
        assert!(py2.contains("continue"), "{py2}");
    }

    #[test]
    fn closures_snapshot_captured_locals() {
        // GEP-0021-R001: rebinding after creation must not leak in
        let py = compile(
            "defmodule M do\n  def straight do\n    x = 1\n    f = fn -> x end\n    x = 2\n    {f.(), x}\n  end\nend",
        );
        assert!(py.contains("lambda *, x=x: x"), "{py}");
        // params stay positional; snapshots are keyword-only (R002)
        let py2 = compile(
            "defmodule M do\n  def make(n), do: fn x -> x + n end\nend",
        );
        assert!(py2.contains("lambda x, *, n=n: x + n"), "{py2}");
        // hoisted multi-clause fn carries the snapshot on its def line
        let py3 = compile(
            "defmodule M do\n  def pick(a) do\n    fn\n      0 -> a\n      y -> y + a\n    end\n  end\nend",
        );
        assert!(py3.contains("def _gan_fn0(*_gan_args, a=a):"), "{py3}");
        // & capture snapshots too
        let py4 = compile(
            "defmodule M do\n  def add(m), do: &(&1 + m)\nend",
        );
        assert!(py4.contains("lambda _gan_cap1, *, m=m: _gan_cap1 + m"), "{py4}");
        // module-level names are not locals: nothing to snapshot
        let py5 = compile(
            "defmodule M do\n  def helper(x), do: x\n  def use_it, do: fn v -> helper(v) end\nend",
        );
        assert!(py5.contains("lambda v: helper(v)"), "{py5}");
    }

    #[test]
    fn closures_in_tco_loops_capture_per_iteration() {
        // GEP-0021 + GEP-0019: the amplified case — snapshots make
        // rebind+continue frame-faithful
        let py = compile(
            "defmodule M do\n  def collect(0, acc), do: acc\n  def collect(n, acc), do: collect(n - 1, [fn -> n end] ++ acc)\nend",
        );
        assert!(py.contains("while True:"), "{py}");
        assert!(py.contains("[lambda *, n=n: n] + acc"), "{py}");
    }

    #[test]
    fn bounded_single_segment_pyref_locks_the_boundary() {
        // GEP-0003-R010: $(sys).stderr.write must import sys, not
        // sys.stderr — the explicit boundary survives the chain walk
        let py = compile(
            "defmodule M do\n  def w(s), do: $(sys).stderr.write(s)\nend",
        );
        assert!(py.contains("import sys\n"), "{py}");
        assert!(!py.contains("import sys.stderr"), "{py}");
        assert!(py.contains("sys.stderr.write(s)"), "{py}");
        // the unbounded spelling keeps the R010 heuristic
        let py2 = compile(
            "defmodule M do\n  def v(x), do: $importlib.metadata.version(x)\nend",
        );
        assert!(py2.contains("import importlib.metadata"), "{py2}");
    }

    fn warnings_of(src: &str) -> Vec<String> {
        let module = crate::parser::parse_file("t.gan", src).unwrap();
        let macros = collect_macros("t.gan", &module).unwrap();
        let mut ex = Expander::new("t.gan", macros);
        let expanded = ex.expand_module(&module).unwrap();
        let mut cg = Codegen::new("t.gan", vec![]);
        cg.compile(&expanded).unwrap();
        cg.warnings.iter().map(|w| w.message.clone()).collect()
    }

    #[test]
    fn undefined_variables_warn() {
        // GEP-0022-R001: reading a name nothing binds is a NameError
        let ws = warnings_of("defmodule M do\n  def f(x), do: x + y\nend");
        assert!(ws.iter().any(|w| w.contains("R001") && w.contains("y")), "{ws:?}");
        // pyimport names — aliased and bare — are known
        let ws2 = warnings_of(
            "defmodule M do\n  pyimport json\n  pyimport numpy, as: np\n  def f(x), do: json.dumps(np.array(x))\nend",
        );
        assert!(!ws2.iter().any(|w| w.contains("R001")), "{ws2:?}");
        // `import Mod` makes bare names unknowable: lint stands down
        let ws3 = warnings_of(
            "defmodule M do\n  import Enum\n  def f(x), do: x + y\nend",
        );
        assert!(!ws3.iter().any(|w| w.contains("R001")), "{ws3:?}");
    }

    #[test]
    fn unused_variables_warn_unless_underscored() {
        // GEP-0022-R002
        let ws = warnings_of("defmodule M do\n  def f(x, y), do: x\nend");
        assert!(ws.iter().any(|w| w.contains("R002") && w.contains("y")), "{ws:?}");
        let ws2 = warnings_of("defmodule M do\n  def f(x, _y), do: x\nend");
        assert!(!ws2.iter().any(|w| w.contains("R002")), "{ws2:?}");
        // sigil splices are reads
        let ws3 = warnings_of(
            "defmodule M do\n  def f(xs), do: $python([x for x in <%= xs %>])\nend",
        );
        assert!(!ws3.iter().any(|w| w.contains("R002")), "{ws3:?}");
        // a var read only in a later clause of the group still counts
        let ws4 = warnings_of(
            "defmodule M do\n  def f(0, y), do: y\n  def f(n, y), do: n + y\nend",
        );
        assert!(!ws4.iter().any(|w| w.contains("R002")), "{ws4:?}");
    }

    #[test]
    fn unreachable_clauses_warn() {
        // GEP-0022-R003: def group — catch-all before more clauses
        let ws = warnings_of(
            "defmodule M do\n  def f(0), do: 1\n  def f(n), do: n\n  def f(9), do: 2\nend",
        );
        assert!(ws.iter().any(|w| w.contains("R003")), "{ws:?}");
        // different arities do not shadow each other
        let ws2 = warnings_of(
            "defmodule M do\n  def f(n), do: n\n  def f(a, b), do: a + b\nend",
        );
        assert!(!ws2.iter().any(|w| w.contains("R003")), "{ws2:?}");
        // case: wildcard shadowing later clauses
        let ws3 = warnings_of(
            "defmodule M do\n  def f(x) do\n    case x do\n      _ -> :any\n      1 -> :one\n    end\n  end\nend",
        );
        assert!(ws3.iter().any(|w| w.contains("R003")), "{ws3:?}");
        // guarded catch-all is refutable: no warning
        let ws4 = warnings_of(
            "defmodule M do\n  def f(n) when n > 0, do: n\n  def f(n), do: -n\nend",
        );
        assert!(!ws4.iter().any(|w| w.contains("R003")), "{ws4:?}");
    }

    #[test]
    fn discarded_comprehensions_and_dead_defps_warn() {
        // GEP-0022-R004
        let ws = warnings_of(
            "defmodule M do\n  def f(xs) do\n    for x <- xs, do: IO.puts(x)\n    :ok\n  end\nend",
        );
        assert!(ws.iter().any(|w| w.contains("R004")), "{ws:?}");
        // GEP-0022-R005 + @allow :unused_function
        let ws2 = warnings_of(
            "defmodule M do\n  def f(x), do: x\n  defp helper(x), do: x\nend",
        );
        assert!(ws2.iter().any(|w| w.contains("R005") && w.contains("helper")), "{ws2:?}");
        let ws3 = warnings_of(
            "defmodule M do\n  def f(x), do: helper(x)\n  defp helper(x), do: x\nend",
        );
        assert!(!ws3.iter().any(|w| w.contains("R005")), "{ws3:?}");
        let ws4 = warnings_of(
            "defmodule M do\n  def f(x), do: x\n  @allow :unused_function\n  defp helper(x), do: x\nend",
        );
        assert!(!ws4.iter().any(|w| w.contains("R005")), "{ws4:?}");
    }

    #[test]
    fn stack_recursion_warns_unless_allowed() {
        // GEP-0019-R007: non-tail self-recursion gets a spanned warning
        let src = "defmodule M do\n  def fact(0), do: 1\n  def fact(n), do: n * fact(n - 1)\nend";
        let module = crate::parser::parse_file("t.gan", src).unwrap();
        let macros = collect_macros("t.gan", &module).unwrap();
        let mut ex = Expander::new("t.gan", macros);
        let expanded = ex.expand_module(&module).unwrap();
        let mut cg = Codegen::new("t.gan", vec![]);
        cg.compile(&expanded).unwrap();
        let w = cg
            .warnings
            .iter()
            .find(|w| w.message.contains("GEP-0019-R007"))
            .expect("expected a stack-recursion warning");
        assert_eq!(w.span.line, 2, "{w:?}");
        // acknowledged intent silences it
        let src2 = "defmodule M do\n  @allow :stack_recursion\n  def fact(0), do: 1\n  def fact(n), do: n * fact(n - 1)\nend";
        let module2 = crate::parser::parse_file("t.gan", src2).unwrap();
        let macros2 = collect_macros("t.gan", &module2).unwrap();
        let mut ex2 = Expander::new("t.gan", macros2);
        let expanded2 = ex2.expand_module(&module2).unwrap();
        let mut cg2 = Codegen::new("t.gan", vec![]);
        cg2.compile(&expanded2).unwrap();
        assert!(
            !cg2.warnings.iter().any(|w| w.message.contains("GEP-0019-R007")),
            "{:?}",
            cg2.warnings
        );
        // tail recursion never warns
        let src3 = "defmodule M do\n  def down(0), do: :done\n  def down(n), do: down(n - 1)\nend";
        let module3 = crate::parser::parse_file("t.gan", src3).unwrap();
        let macros3 = collect_macros("t.gan", &module3).unwrap();
        let mut ex3 = Expander::new("t.gan", macros3);
        let expanded3 = ex3.expand_module(&module3).unwrap();
        let mut cg3 = Codegen::new("t.gan", vec![]);
        cg3.compile(&expanded3).unwrap();
        assert!(cg3.warnings.is_empty(), "{:?}", cg3.warnings);
        // a typo in the allow target is an error, not a silent no-op
        let err = compile_err(
            "defmodule M do\n  @allow :stack_recursoin\n  def f(x), do: x\nend",
        );
        assert!(err.contains("does not recognize"), "{err}");
    }

    #[test]
    fn recursion_shape_is_reported() {
        // GEP-0019-R006
        use crate::parser::parse_file;
        let src = "defmodule M do\n  def down(0), do: :done\n  def down(n), do: down(n - 1)\n  def fact(0), do: 1\n  def fact(n), do: n * fact(n - 1)\n  def plain(x), do: x\nend";
        let term = parse_file("t.gan", src).unwrap();
        let shape = |name: &str, arity: usize| {
            let mut bodies = Vec::new();
            let arities: BTreeSet<usize> = [arity].into();
            for stmt in term.as_block() {
                let Term::Call(dm) = &stmt else { continue };
                let Some(body) = Term::keyword_arg(&dm.args, "do") else { continue };
                for inner in body.as_block() {
                    if let Term::Call(c) = &inner {
                        if inner.is_call_named("def") {
                            if let Some(Term::Call(h)) = c.args.first() {
                                if matches!(&h.callee, Callee::Name(n) if n == name) {
                                    bodies.push(
                                        Term::keyword_arg(&c.args, "do").unwrap().clone(),
                                    );
                                }
                            }
                        }
                    }
                }
            }
            let refs: Vec<&Term> = bodies.iter().collect();
            recursion_shape(name, &arities, &refs).map(|s| s.to_string())
        };
        assert_eq!(shape("down", 1).as_deref(), Some("loop"));
        assert_eq!(shape("fact", 1).as_deref(), Some("stack"));
        assert_eq!(shape("plain", 1), None);
    }

    #[test]
    fn function_level_recur_is_checked_and_compiled() {
        // GEP-0019-R005
        let py = compile(
            "defmodule M do\n  def go(n, acc) do\n    if n == 0 do\n      acc\n    else\n      recur(n - 1, acc + n)\n    end\n  end\nend",
        );
        assert!(py.contains("while True:"), "{py}");
        assert!(py.contains("n, acc = n - 1, acc + n"), "{py}");
        let err = compile_err(
            "defmodule M do\n  def go(n) do\n    if n == 0 do\n      :done\n    else\n      recur(n - 1, n)\n    end\n  end\nend",
        );
        assert!(err.contains("GEP-0019-R005"), "{err}");
    }

    #[test]
    fn non_tail_recursion_is_untouched() {
        // GEP-0019-R003
        let py = compile(
            "defmodule M do\n  def fact(0), do: 1\n  def fact(n), do: n * fact(n - 1)\nend",
        );
        assert!(!py.contains("while True:"), "{py}");
        assert!(py.contains("return n * fact(n - 1)"), "{py}");
    }

    #[test]
    fn tail_calls_inside_try_are_not_optimized() {
        // GEP-0019-R001: try is never a tail position
        let py = compile(
            "defmodule M do\n  def f(n) do\n    try do\n      f(n - 1)\n    rescue\n      _e -> :stop\n    end\n  end\nend",
        );
        assert!(!py.contains("continue"), "{py}");
    }

    #[test]
    fn type_variables_compile_to_typevars() {
        // GEP-0017-R005
        let py = compile(
            "defmodule M do\n  @spec pick(list(a), integer()) :: a | nil\n  def pick(xs, i), do: xs\nend",
        );
        assert!(py.contains("_T_a = typing.TypeVar(\"_T_a\")"), "{py}");
        assert!(py.contains("def pick(xs: list[_T_a], i: int) -> _T_a | None:"), "{py}");
        let err = compile_err(
            "defmodule M do\n  @spec f(integer) :: integer()\n  def f(x), do: x\nend",
        );
        assert!(err.contains("write integer()"), "{err}");
    }

    #[test]
    fn abstract_container_types() {
        // GEP-0017-R002 rev 3
        let py = compile(
            "defmodule M do\n  @spec total(sequence(number())) :: number()\n  def total(xs), do: xs\nend",
        );
        assert!(
            py.contains("def total(xs: collections.abc.Sequence[int | float]) -> int | float:"),
            "{py}"
        );
    }

    #[test]
    fn dotted_pyref_chains_import_the_module_path() {
        // GEP-0003-R010
        let py = compile(
            "defmodule M do\n  def v(d), do: $importlib.metadata.version(d)\nend",
        );
        assert!(py.contains("import importlib.metadata"), "{py}");
        assert!(py.contains("return importlib.metadata.version(d)"), "{py}");
        let py2 = compile(
            "defmodule M do\n  def c(x), do: $builtins.isinstance(x, $collections.abc.Sequence)\nend",
        );
        assert!(py2.contains("import collections.abc"), "{py2}");
    }

    #[test]
    fn quoted_pyref_locks_the_module_boundary() {
        // `$"a.b".c` — the quote is an explicit boundary the chain rule
        // must not extend (GEP-0003-R010)
        let py = compile(
            "defmodule M do\n  def f(), do: $(os.path).sep.join([\"a\"])\nend",
        );
        assert!(py.contains("import os.path"), "{py}");
        assert!(!py.contains("import os.path.sep"), "{py}");
        assert!(py.contains("os.path.sep.join"), "{py}");
    }

    #[test]
    fn parametrized_host_types() {
        // GEP-0017-R002 rev 2: $mod.Type(t) -> mod.Type[t]
        let py = compile(
            "defmodule M do\n  @spec total($(collections.abc).Sequence(number())) :: float()\n  def total(xs), do: xs\nend",
        );
        assert!(
            py.contains("def total(xs: collections.abc.Sequence[int | float]) -> float:"),
            "{py}"
        );
        assert!(py.contains("import collections.abc"), "{py}");
    }

    #[test]
    fn named_spec_args_compile_like_unnamed() {
        // GEP-0018-R006
        let py = compile(
            "defmodule M do\n  @spec add(a :: integer(), b :: integer()) :: integer()\n  def add(a, b), do: a + b\nend",
        );
        assert!(py.contains("def add(a: int, b: int) -> int:"), "{py}");
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
            "defmodule M do\n  @spec load(string()) :: $decimal.Decimal() | map()\n  def load(p), do: p\nend",
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
    fn async_def_and_await_emit_one_to_one() {
        let py = compile(
            "defmodule M do\n  async defp fetch(x) do\n    r = await g(x)\n    t = Task.async(g(x))\n    u = await Task.try_await(t, 1000)\n    {r, u}\n  end\n  async def g(x), do: x\nend",
        );
        assert!(py.contains("async def _fetch(x):"), "{py}");
        assert!(py.contains("async def g(x):"), "{py}");
        assert!(py.contains("r = (await g(x))"), "{py}");
        // Task calls are ordinary std calls everywhere (GEP-0030-R007)
        assert!(py.contains("task.async__kw(g(x))"), "{py}");
        assert!(py.contains("(await task.try_await(t, 1000))"), "{py}");
        assert!(!py.contains("wait_for"), "{py}");
    }

    #[test]
    fn await_binds_tighter_than_binary_operators() {
        let py = compile(
            "defmodule M do\n  async def f(a, b) do\n    x = await g(a) |> h()\n    y = await g(a) + await g(b)\n    {x, y}\n  end\n  async def g(x), do: x\n  def h(x), do: x\nend",
        );
        // `await g(a) |> h()` pipes the awaited value (GEP-0030-R002)
        assert!(py.contains("h((await g(a)))"), "{py}");
        assert!(py.contains("(await g(a)) + (await g(b))"), "{py}");
    }

    #[test]
    fn await_in_comprehension_bodies_is_native() {
        let py = compile(
            "defmodule M do\n  async def all(ts) do\n    for t <- ts, do: await t\n  end\nend",
        );
        assert!(py.contains("[(await t) for t in ts]"), "{py}");
    }

    #[test]
    fn await_outside_async_bodies_is_rejected() {
        let err = compile_err("defmodule M do\n  def f(t), do: await t\nend");
        assert!(err.contains("GEP-0030-R002"), "{err}");
    }

    #[test]
    fn await_inside_fn_closures_is_rejected() {
        let err = compile_err(
            "defmodule M do\n  async def f(t) do\n    g = fn -> await t end\n    g.()\n  end\nend",
        );
        assert!(err.contains("fn closure is synchronous"), "{err}");
    }

    #[test]
    fn mixed_sync_and_async_clauses_are_rejected() {
        let err = compile_err(
            "defmodule M do\n  async def f(0), do: 0\n  def f(x), do: x\nend",
        );
        assert!(err.contains("mix def and async def"), "{err}");
    }

    #[test]
    fn async_main_is_rejected() {
        let err = compile_err("defmodule M do\n  async def main(), do: 1\nend");
        assert!(err.contains("GEP-0030-R001"), "{err}");
    }

    #[test]
    fn async_examples_are_runnable_doctests() {
        let py = compile(
            "defmodule M do\n  @doc \"d\"\n  @example \"\"\"\n      gan> Task.run(M.f(1))\n      1\n  \"\"\"\n  async def f(x), do: x\nend",
        );
        // the example compiles to an ordinary doctest (GEP-0030-R005)
        assert!(py.contains(">>>"), "{py}");
        assert!(!py.contains("gan>"), "{py}");
    }

    #[test]
    fn async_and_await_stay_ordinary_names_elsewhere() {
        let py = compile(
            "defmodule M do\n  def f(t), do: Task.async(t)\n  def g() do\n    async = 1\n    async + 1\n  end\nend",
        );
        assert!(py.contains("task.async__kw(t)"), "{py}");
        assert!(py.contains("async__kw = 1"), "{py}");
    }

    #[test]
    fn grouped_or_keeps_its_parens_under_and() {
        let py = compile(
            "defmodule M do\n  def f(t) do\n    (Map.get(t, \"k\") == \"a\" or Map.get(t, \"k\") == \"b\") and Map.get(t, \"n\") > 1\n  end\nend",
        );
        // `(A or B) and C`, never `A or B and C` (Python would re-associate)
        assert!(py.contains(") and "), "{py}");
        let and_split = py.split(") and ").next().unwrap();
        assert!(and_split.ends_with("\"b\"))") || and_split.ends_with(')'), "{py}");
        assert!(!py.contains("\"a\") or (gandora_std.map.get(t, \"k\") == \"b\") and"), "{py}");
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
    fn cjk_in_a_default_doc_channel_is_an_error() {
        // GEP-0007-R011: the default channel is the retrieval index's
        // language; localized prose belongs in the _trans channels
        for src in [
            "defmodule M do\n  @doc \"对每个元素求值\"\n  def f(), do: 1\nend",
            "defmodule M do\n  @moduledoc \"模块文档\"\n  def f(), do: 1\nend",
            "defmodule M do\n  @doc \"d\"\n  @param x, \"输入\"\n  def f(x), do: x\nend",
        ] {
            let err = compile_err(src);
            assert!(err.contains("GEP-0007-R011"), "{err}");
            assert!(err.contains("_trans"), "{err}");
        }
    }

    #[test]
    fn cjk_stays_welcome_in_trans_channels() {
        let py = compile(
            "defmodule M do\n  @moduledoc \"module doc\"\n  @moduledoc_trans zh_CN: \"模块文档\"\n  @doc \"doubles\"\n  @doc_trans zh_CN: \"翻倍\"\n  @param x, \"the input\"\n  @param_trans x, zh_CN: \"输入\"\n  def f(x), do: x\nend",
        );
        // the trans channels compile without tripping R011
        assert!(py.contains("doubles"), "{py}");
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
            "defmodule M do\n  def squares(n) do\n    $python(sum(i * i for i in range(n)))\n  end\nend",
        );
        assert!(py.contains("return (sum(i * i for i in range(n)))"), "{py}");
    }

    #[test]
    fn py_sigil_composes_with_pipes() {
        let py = compile(
            "defmodule M do\n  def f(xs) do\n    xs |> $builtins.sorted() |> $python(list)()\n  end\nend",
        );
        assert!(py.contains("return (list)(builtins.sorted(xs))"), "{py}");
    }

    #[test]
    fn any_name_is_an_embedded_sigil_now() {
        // GEP-0005-R009 was repealed by GEP-0009-R001; short names are
        // the functional whitelist (GEP-0005-R010), 3+ chars are tags
        let py = compile("defmodule M do\n  def f(), do: ~zzz(nope)\nend");
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
            "defmodule M do\n  @doc \"\"\"\nSign check. # this is prose, not a comment\n\nThe # mark above stays put.\n\"\"\"\n  @example \"\"\"\n    gan> classify(-3) # 行尾中文注释会被词法器剥掉\n    'negative'\n\"\"\"\n  def classify(x), do: :negative\nend",
        );
        // prose with a # mark passes through verbatim (comments are a
        // lexer concept, not a docstring one); the Chinese trailing
        // comment in the @example is stripped before R011 ever looks
        assert!(py.contains("Sign check. # this is prose, not a comment"), "{py}");
        assert!(py.contains("The # mark above stays put."), "{py}");
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
        assert!(
            cg.warnings.iter().any(|w| w.message.contains("GEP-0007-R005")),
            "{:?}",
            cg.warnings
        );
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
            "defmodule M do\n  def evens(xs, limit) do\n    $python([x for x in <%= xs %> if x % 2 == 0][:<%= limit + 1 %>])\n  end\nend",
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
    fn hook_emitted_registrations_join_the_value_table() {
        // GEP-0008-R005: a hook returns "the reconstructed definition
        // plus registrations" — the emitted @table write is absorbed as
        // a registration, never a plain module attribute for codegen
        let py = compile(
            "defmodule M do\n  defattr :cmd\n  defattr :table, accumulate: true\n  defmacro on_def(kind, head, attrs, body) do\n    quote do\n      def unquote(head) do\n        unquote(body)\n      end\n      @table unquote(length(attrs))\n    end\n  end\n  @on_definition M.on_def\n  @cmd \"a\"\n  def a(), do: :ok\n  @cmd \"b\"\n  def b(), do: :ok\n  def table(), do: @table\nend",
        );
        assert!(py.contains("def a():"), "{py}");
        // the hook ran for all three defs; table() itself carried no @cmd
        assert!(py.contains("return [1, 1, 0]"), "{py}");
    }

    #[test]
    fn macro_emitted_write_to_non_accumulating_attr_still_errors() {
        let err = expand_err(
            "defmodule M do\n  defattr :one\n  defmacro reg(v) do\n    quote do\n      @one unquote(v)\n    end\n  end\n  @one 1\n  reg(2)\n  def f(), do: @one\nend",
        );
        assert!(err.contains("GEP-0008-R004"), "{err}");
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
    fn literal_defaults_fold_into_native_python_defaults() {
        // one real clause + immutable literal defaults: no dispatcher,
        // a signature ty can check arity against
        let py = compile(
            "defmodule M do\n  def greet(name, greeting \\\\ \"hello\", mark \\\\ \"!\") do\n    greeting <> \", \" <> name <> mark\n  end\nend",
        );
        assert!(
            py.contains("def greet(name, greeting=\"hello\", mark=\"!\"):"),
            "{py}"
        );
        assert!(!py.contains("_gan_args"), "{py}");
    }

    #[test]
    fn mutable_defaults_keep_the_dispatcher() {
        // a `[]` default must evaluate per call (Elixir semantics), so
        // the delegating-clause dispatcher stays
        let py = compile(
            "defmodule M do\n  def pad(xs, tail \\\\ []) do\n    xs ++ tail\n  end\nend",
        );
        assert!(py.contains("*_gan_args"), "{py}");
        assert!(py.contains("case (xs,):"), "{py}");
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
    // snippet top level is a scope too: closures over its bindings
    // snapshot like everywhere else (GEP-0021)
    let mut snippet_scope = std::collections::BTreeSet::new();
    scope_binders(term, &mut snippet_scope);
    cg.scope_bound.push(snippet_scope);
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
    fn recur_replaces_the_retired_loop() {
        // GEP-0014-R007 + GEP-0019: the loop shape now comes from recur
        let py = compile(
            "defmodule M do\n  defp tally(left, seen) do\n    case left do\n      [] -> seen\n      [h | t] -> recur(t, seen + h)\n    end\n  end\nend",
        );
        assert!(py.contains("while True:"), "{py}");
        assert!(py.contains("continue"), "{py}");
    }

    #[test]
    fn recur_outside_loop_or_function_tail_is_an_error() {
        // since GEP-0019-R005, a function-tail recur is legal (arity-checked);
        // inside try the jump would change rescue semantics, so it stays out
        let err = compile_err(
            "defmodule M do\n  def f(x) do\n    try do\n      recur(x)\n    rescue\n      _e -> :x\n    end\n  end\nend",
        );
        assert!(err.contains("GEP-0014-R005"), "{err}");
        // wrong arity on a function-tail recur
        let err2 = compile_err("defmodule M do\n  def f(), do: recur(1)\nend");
        assert!(err2.contains("GEP-0019-R005"), "{err2}");
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
