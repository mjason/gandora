//! Project configuration and build orchestration (GEP-0001-R013, R018-R020).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::ast::Term;
use crate::codegen::{camel_to_snake, module_py_path, Codegen};
use crate::diag::{Diagnostic, Result, Span};
use crate::expander::{collect_macros, Expander, MacroTable};
use crate::jsonc::{parse_jsonc, JsonValue};
use crate::parser::parse_file;

pub struct Config {
    pub root: PathBuf,
    pub source: Vec<String>,
    pub out_dir: String,
    pub target_python: String,
    pub exclude: Vec<String>,
    /// package project: `gan build` also emits marker + shipped sources
    /// (GEP-0006-R001/R002)
    pub package: bool,
    /// python package prefix for compiled output, e.g. "gandora_std"
    /// (GEP-0010-R002)
    pub py_package: Option<String>,
}

impl Config {
    pub fn default_at(root: PathBuf) -> Self {
        Config {
            root,
            source: vec!["src".into()],
            out_dir: "dist".into(),
            target_python: "3.11".into(),
            exclude: Vec::new(),
            package: false,
            py_package: None,
        }
    }
}

/// Find the nearest ancestor `gandora.jsonc` (GEP-0001-R018).
pub fn find_config(start: &Path) -> Result<Option<Config>> {
    let mut dir = Some(start.to_path_buf());
    while let Some(d) = dir {
        let candidate = d.join("gandora.jsonc");
        if candidate.exists() {
            return load_config(&candidate).map(Some);
        }
        dir = d.parent().map(|p| p.to_path_buf());
    }
    Ok(None)
}

pub fn load_config(path: &Path) -> Result<Config> {
    let text = std::fs::read_to_string(path).map_err(|e| {
        Diagnostic::new(path.display().to_string(), Span::default(), e.to_string())
    })?;
    let file = path.display().to_string();
    let value = parse_jsonc(&file, &text)?;
    let JsonValue::Object(entries) = value else {
        return Err(Diagnostic::new(
            &file,
            Span::default(),
            "gandora.jsonc must contain a JSON object",
        ));
    };
    let mut config = Config::default_at(path.parent().unwrap().to_path_buf());
    for (key, val) in entries {
        match key.as_str() {
            "source" => {
                config.source = string_array(&file, &key, val)?;
            }
            "outDir" => {
                config.out_dir = string_value(&file, &key, val)?;
            }
            "targetPython" => {
                config.target_python = string_value(&file, &key, val)?;
            }
            "exclude" => {
                config.exclude = string_array(&file, &key, val)?;
            }
            "pyPackage" => {
                config.py_package = Some(string_value(&file, &key, val)?);
            }
            "package" => match val {
                JsonValue::Bool(b) => config.package = b,
                _ => {
                    return Err(Diagnostic::new(
                        &file,
                        Span::default(),
                        "'package' must be a boolean",
                    ))
                }
            },
            "$schema" => {}
            other => {
                return Err(Diagnostic::new(
                    &file,
                    Span::default(),
                    format!("unknown gandora.jsonc field '{other}' (GEP-0001-R018)"),
                ))
            }
        }
    }
    Ok(config)
}

fn string_value(file: &str, key: &str, val: JsonValue) -> Result<String> {
    match val {
        JsonValue::String(s) => Ok(s),
        _ => Err(Diagnostic::new(
            file,
            Span::default(),
            format!("'{key}' must be a string"),
        )),
    }
}

fn string_array(file: &str, key: &str, val: JsonValue) -> Result<Vec<String>> {
    match val {
        JsonValue::Array(items) => items
            .into_iter()
            .map(|i| string_value(file, key, i))
            .collect(),
        _ => Err(Diagnostic::new(
            file,
            Span::default(),
            format!("'{key}' must be an array of strings"),
        )),
    }
}

/// Discover `.gan` sources and derive their module names (GEP-0001-R013).
pub fn discover_sources(config: &Config) -> Result<Vec<(PathBuf, Vec<String>)>> {
    let mut out = Vec::new();
    for root in &config.source {
        let root_path = config.root.join(root);
        if !root_path.exists() {
            continue;
        }
        let mut stack = vec![root_path.clone()];
        while let Some(dir) = stack.pop() {
            let entries = std::fs::read_dir(&dir).map_err(|e| {
                Diagnostic::new(dir.display().to_string(), Span::default(), e.to_string())
            })?;
            let mut items: Vec<PathBuf> = entries
                .filter_map(|e| e.ok().map(|e| e.path()))
                .collect();
            items.sort();
            for item in items {
                let rel = item
                    .strip_prefix(&config.root)
                    .unwrap_or(&item)
                    .to_string_lossy()
                    .replace('\\', "/");
                if rel.starts_with(&config.out_dir) || rel.starts_with(".gandora") {
                    continue;
                }
                if config.exclude.iter().any(|pat| glob_match(pat, &rel)) {
                    continue;
                }
                if item.is_dir() {
                    stack.push(item);
                } else if item.extension().is_some_and(|e| e == "gan") {
                    let module = module_name_for(&item, &root_path)?;
                    out.push((item, module));
                }
            }
        }
    }
    out.sort();
    Ok(out)
}

/// `src/app/hello_web.gan` under root `src` -> `["App", "HelloWeb"]`.
pub fn module_name_for(path: &Path, source_root: &Path) -> Result<Vec<String>> {
    let rel = path.strip_prefix(source_root).map_err(|_| {
        Diagnostic::new(
            path.display().to_string(),
            Span::default(),
            "source file is outside every configured source root",
        )
    })?;
    let mut segs = Vec::new();
    let parts: Vec<_> = rel.iter().collect();
    for (i, part) in parts.iter().enumerate() {
        let mut s = part.to_string_lossy().to_string();
        if i + 1 == parts.len() {
            s = s.trim_end_matches(".gan").to_string();
        }
        segs.push(snake_to_camel(&s));
    }
    Ok(segs)
}

pub fn snake_to_camel(s: &str) -> String {
    s.split('_')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let mut cs = p.chars();
            match cs.next() {
                Some(first) => first.to_uppercase().collect::<String>() + cs.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

fn glob_match(pattern: &str, path: &str) -> bool {
    // A pattern without glob syntax also excludes its descendants.
    if !pattern.contains('*') {
        return path == pattern || path.starts_with(&format!("{pattern}/"));
    }
    glob_match_inner(
        &pattern.split('/').collect::<Vec<_>>(),
        &path.split('/').collect::<Vec<_>>(),
    )
}

fn glob_match_inner(pat: &[&str], path: &[&str]) -> bool {
    match (pat.first(), path.first()) {
        (None, None) => true,
        (Some(&"**"), _) => {
            glob_match_inner(&pat[1..], path)
                || (!path.is_empty() && glob_match_inner(pat, &path[1..]))
        }
        (Some(p), Some(s)) => segment_match(p, s) && glob_match_inner(&pat[1..], &path[1..]),
        _ => false,
    }
}

fn segment_match(pat: &str, s: &str) -> bool {
    // '*' within one path segment
    let parts: Vec<&str> = pat.split('*').collect();
    if parts.len() == 1 {
        return pat == s;
    }
    let mut rest = s;
    for (i, part) in parts.iter().enumerate() {
        if i == 0 {
            if !rest.starts_with(part) {
                return false;
            }
            rest = &rest[part.len()..];
        } else if i + 1 == parts.len() {
            return rest.ends_with(part);
        } else if let Some(idx) = rest.find(part) {
            rest = &rest[idx + part.len()..];
        } else {
            return false;
        }
    }
    true
}

pub struct CompiledModule {
    pub source: PathBuf,
    pub module: Vec<String>,
    pub py_path: String,
    pub python: String,
    /// macros-only module: no Python file is written (GEP-0002-R009)
    pub compile_time_only: bool,
    /// non-fatal notices, printed by check/build (GEP-0007-R005)
    pub warnings: Vec<String>,
}

/// Parse, expand, and compile every module of a project.
pub fn compile_project(config: &Config) -> Result<Vec<CompiledModule>> {
    let sources = discover_sources(config)?;
    compile_files(&sources, Some(config))
}

pub fn compile_files(
    sources: &[(PathBuf, Vec<String>)],
    config: Option<&Config>,
) -> Result<Vec<CompiledModule>> {
    // 1. parse everything and collect local macro tables
    struct Parsed {
        path: PathBuf,
        module: Vec<String>,
        term: Term,
        macros: MacroTable,
        deps: Vec<String>,
    }
    let mut parsed: Vec<Parsed> = Vec::new();
    for (path, module) in sources {
        let file = path.display().to_string();
        let text = std::fs::read_to_string(path)
            .map_err(|e| Diagnostic::new(&file, Span::default(), e.to_string()))?;
        let term = parse_file(&file, &text)?;
        let macros = collect_macros(&file, &term)?;
        let deps = macro_deps(&term);
        parsed.push(Parsed {
            path: path.clone(),
            module: module.clone(),
            term,
            macros,
            deps,
        });
    }
    let by_name: BTreeMap<String, usize> = parsed
        .iter()
        .enumerate()
        .map(|(i, p)| (p.module.join("."), i))
        .collect();

    // 2. detect macro-dependency cycles (GEP-0002-R006)
    for (i, p) in parsed.iter().enumerate() {
        let mut stack = vec![(i, vec![i])];
        while let Some((cur, chain)) = stack.pop() {
            for dep in &parsed[cur].deps {
                if let Some(&j) = by_name.get(dep) {
                    if j == i && chain.len() > 0 && cur != i {
                        return Err(Diagnostic::new(
                            p.path.display().to_string(),
                            Span::default(),
                            format!(
                                "macro dependency cycle involving {}",
                                p.module.join(".")
                            ),
                        ));
                    }
                    if !chain.contains(&j) {
                        let mut c = chain.clone();
                        c.push(j);
                        stack.push((j, c));
                    }
                }
            }
        }
    }

    // 2b. resolve require/import deps that are not project modules against
    //     installed package markers (GEP-0006-R006/R008)
    let mut external: BTreeMap<String, MacroTable> = BTreeMap::new();
    for p in &parsed {
        for dep in &p.deps {
            if by_name.contains_key(dep) || external.contains_key(dep) {
                continue;
            }
            let Some(cfg) = config else {
                return Err(Diagnostic::new(
                    p.path.display().to_string(),
                    Span::default(),
                    format!("module {dep} named by require/import is not among the compiled files"),
                ));
            };
            let Some(source) = find_installed_source(cfg, dep) else {
                return Err(Diagnostic::new(
                    p.path.display().to_string(),
                    Span::default(),
                    format!(
                        "module {dep} named by require/import was found neither in project \
                         sources nor in an installed package under {} (GEP-0006-R008)",
                        cfg.root.join(".venv").display()
                    ),
                ));
            };
            let file = source.display().to_string();
            let text = std::fs::read_to_string(&source)
                .map_err(|e| Diagnostic::new(&file, Span::default(), e.to_string()))?;
            let term = parse_file(&file, &text)?;
            external.insert(dep.clone(), collect_macros(&file, &term)?);
        }
    }

    let project_modules: BTreeSet<String> =
        parsed.iter().map(|p| p.module.join(".")).collect();
    let installed = match config {
        Some(cfg) => installed_module_map(cfg),
        None => BTreeMap::new(),
    };
    let py_prefix = config.and_then(|c| c.py_package.clone());

    // 3. expand and compile each module with its visible macros
    let mut out = Vec::new();
    for p in &parsed {
        let file = p.path.display().to_string();
        let mut table = p.macros.clone();
        for dep in &p.deps {
            if let Some(&j) = by_name.get(dep) {
                for (k, v) in &parsed[j].macros {
                    table.entry(k.clone()).or_insert_with(|| v.clone());
                }
            } else if let Some(ext) = external.get(dep) {
                for (k, v) in ext {
                    table.entry(k.clone()).or_insert_with(|| v.clone());
                }
            }
        }
        let mut expander = Expander::new(&file, table);
        let expanded = expander.expand_module(&p.term)?;
        let mut cg = Codegen::new(&file, p.module.clone());
        cg.py_prefix = py_prefix.clone();
        cg.project_modules = project_modules.clone();
        cg.installed_modules = installed.clone();
        let python = cg.compile(&expanded)?;
        out.push(CompiledModule {
            source: p.path.clone(),
            module: p.module.clone(),
            py_path: match &py_prefix {
                Some(prefix) => format!(
                    "{prefix}/{}.py",
                    module_py_path(&p.module).replace('.', "/")
                ),
                None => module_py_path(&p.module).replace('.', "/") + ".py",
            },
            python,
            compile_time_only: cg.compile_time_only,
            warnings: {
                let mut warnings = std::mem::take(&mut cg.warnings);
                let py_path = match &py_prefix {
                    Some(prefix) => format!(
                        "{prefix}/{}.py",
                        module_py_path(&p.module).replace('.', "/")
                    ),
                    None => module_py_path(&p.module).replace('.', "/") + ".py",
                };
                if let Some(stem) = py_path.strip_suffix(".py") {
                    if !stem.contains('/') && PY_STDLIB_MODULES.contains(&stem) {
                        warnings.push(format!(
                            "module {} compiles to {stem}.py, shadowing Python's \
                             standard-library module '{stem}' for everything on the \
                             project path; rename the module or set pyPackage",
                            p.module.join(".")
                        ));
                    }
                }
                warnings
            },
        });
    }
    Ok(out)
}

/// Python 3.11 standard-library module names (sys.stdlib_module_names,
/// underscore-prefixed entries dropped): a top-level project module
/// compiling to one of these file names shadows the stdlib on
/// PYTHONPATH and breaks any package that imports it.
const PY_STDLIB_MODULES: &[&str] = &[
    "abc", "aifc", "antigravity", "argparse", "array", "ast",
    "asynchat", "asyncio", "asyncore", "atexit", "audioop", "base64",
    "bdb", "binascii", "bisect", "builtins", "bz2", "cProfile",
    "calendar", "cgi", "cgitb", "chunk", "cmath", "cmd",
    "code", "codecs", "codeop", "collections", "colorsys", "compileall",
    "concurrent", "configparser", "contextlib", "contextvars", "copy", "copyreg",
    "crypt", "csv", "ctypes", "curses", "dataclasses", "datetime",
    "dbm", "decimal", "difflib", "dis", "distutils", "doctest",
    "email", "encodings", "ensurepip", "enum", "errno", "faulthandler",
    "fcntl", "filecmp", "fileinput", "fnmatch", "fractions", "ftplib",
    "functools", "gc", "genericpath", "getopt", "getpass", "gettext",
    "glob", "graphlib", "grp", "gzip", "hashlib", "heapq",
    "hmac", "html", "http", "idlelib", "imaplib", "imghdr",
    "imp", "importlib", "inspect", "io", "ipaddress", "itertools",
    "json", "keyword", "lib2to3", "linecache", "locale", "logging",
    "lzma", "mailbox", "mailcap", "marshal", "math", "mimetypes",
    "mmap", "modulefinder", "msilib", "msvcrt", "multiprocessing", "netrc",
    "nis", "nntplib", "nt", "ntpath", "nturl2path", "numbers",
    "opcode", "operator", "optparse", "os", "ossaudiodev", "pathlib",
    "pdb", "pickle", "pickletools", "pipes", "pkgutil", "platform",
    "plistlib", "poplib", "posix", "posixpath", "pprint", "profile",
    "pstats", "pty", "pwd", "py_compile", "pyclbr", "pydoc",
    "pydoc_data", "pyexpat", "queue", "quopri", "random", "re",
    "readline", "reprlib", "resource", "rlcompleter", "runpy", "sched",
    "secrets", "select", "selectors", "shelve", "shlex", "shutil",
    "signal", "site", "smtpd", "smtplib", "sndhdr", "socket",
    "socketserver", "spwd", "sqlite3", "sre_compile", "sre_constants", "sre_parse",
    "ssl", "stat", "statistics", "string", "stringprep", "struct",
    "subprocess", "sunau", "symtable", "sys", "sysconfig", "syslog",
    "tabnanny", "tarfile", "telnetlib", "tempfile", "termios", "textwrap",
    "this", "threading", "time", "timeit", "tkinter", "token",
    "tokenize", "tomllib", "trace", "traceback", "tracemalloc", "tty",
    "turtle", "turtledemo", "types", "typing", "unicodedata", "unittest",
    "urllib", "uu", "uuid", "venv", "warnings", "wave",
    "weakref", "webbrowser", "winreg", "winsound", "wsgiref", "xdrlib",
    "xml", "xmlrpc", "zipapp", "zipfile", "zipimport", "zlib",
    "zoneinfo",
];


/// Locate a module's source (project first, then installed packages) and
/// extract the documentation of the module or one of its functions
/// (GEP-0007). Returns Ok(None) when the module or doc is absent.

/// The function name of a def head, looking through a `when` guard.
fn def_head_name(head: &crate::ast::Term) -> Option<String> {
    use crate::ast::{Callee, Term};
    match head {
        Term::Call(hc) => match &hc.callee {
            Callee::Name(n) if n == "when" => hc.args.first().and_then(def_head_name),
            Callee::Name(n) => Some(n.clone()),
            _ => None,
        },
        Term::Var(n, _) => Some(n.clone()),
        _ => None,
    }
}

pub struct SymbolInfo {
    pub name: String,
    pub kind: String,
    pub line: u32,
    pub head: String,
    pub doc_head: Option<String>,
}

/// Every definition of a module, in source order, with rendered heads and
/// the first line of each `@doc` (GEP-0015-R008).
pub fn module_symbols(
    config: &Config,
    module_name: &str,
) -> crate::diag::Result<Vec<SymbolInfo>> {
    let Some((_, term)) = load_module_term(config, module_name)? else {
        return Ok(Vec::new());
    };
    use crate::ast::{Callee, Term};
    let mut out = Vec::new();
    let mut pending_doc: Option<String> = None;
    for stmt in term.as_block() {
        if !stmt.is_call_named("defmodule") {
            continue;
        }
        let Term::Call(dm) = &stmt else { continue };
        let Some(body) = Term::keyword_arg(&dm.args, "do") else { continue };
        for inner in body.as_block() {
            let Term::Call(c) = &inner else { continue };
            let Callee::Name(name) = &c.callee else { continue };
            match name.as_str() {
                "@doc" => {
                    if let Some(Term::Str(_)) = c.args.first() {
                        if let Some(Term::Str(parts)) = c.args.first() {
                            if let Some(crate::ast::StrPart::Text(t)) = parts.first() {
                                pending_doc =
                                    t.lines().find(|l| !l.trim().is_empty()).map(String::from);
                            }
                        }
                    }
                }
                "def" | "defp" | "defmacro" => {
                    if let Some(head) = c.args.first() {
                        let head_name = def_head_name(head);
                        if let Some(h) = head_name {
                            out.push(SymbolInfo {
                                name: h,
                                kind: name.to_string(),
                                line: c.span.line,
                                head: format!("{name} {}", crate::printer::print_expr(head)),
                                doc_head: pending_doc.take(),
                            });
                        }
                    }
                    pending_doc = None;
                }
                _ => {}
            }
        }
    }
    Ok(out)
}

/// Where a module or one of its functions is defined: (path, line, col)
/// of the `defmodule` or the first matching clause (GEP-0015-R006).
pub fn find_definition(
    config: &Config,
    module_name: &str,
    fun: Option<&str>,
) -> crate::diag::Result<Option<(String, u32, u32)>> {
    let Some((file, term)) = load_module_term(config, module_name)? else {
        return Ok(None);
    };
    use crate::ast::{Callee, Term};
    for stmt in term.as_block() {
        if !stmt.is_call_named("defmodule") {
            continue;
        }
        let Term::Call(dm) = &stmt else { continue };
        let Some(f) = fun else {
            return Ok(Some((file, dm.span.line, dm.span.col)));
        };
        let Some(body) = Term::keyword_arg(&dm.args, "do") else { continue };
        for inner in body.as_block() {
            let Term::Call(c) = &inner else { continue };
            let Callee::Name(name) = &c.callee else { continue };
            if !matches!(name.as_str(), "def" | "defp" | "defmacro") {
                continue;
            }
            let head_name = c.args.first().and_then(def_head_name);
            if head_name.as_deref() == Some(f) {
                return Ok(Some((file, c.span.line, c.span.col)));
            }
        }
    }
    Ok(None)
}

/// Locate and parse a module's source: project first, then installed.
fn load_module_term(
    config: &Config,
    module_name: &str,
) -> crate::diag::Result<Option<(String, crate::ast::Term)>> {
    let mut source: Option<std::path::PathBuf> = None;
    for (path, module) in discover_sources(config)? {
        if module.join(".") == module_name {
            source = Some(path);
            break;
        }
    }
    if source.is_none() {
        source = find_installed_source(config, module_name);
    }
    let Some(source) = source else {
        return Ok(None);
    };
    let file = source.display().to_string();
    let text = std::fs::read_to_string(&source).map_err(|e| {
        crate::diag::Diagnostic::new(&file, crate::diag::Span::default(), e.to_string())
    })?;
    let term = crate::parser::parse_file(&file, &text)?;
    Ok(Some((file, term)))
}

pub fn find_doc(
    config: &Config,
    module_name: &str,
    fun: Option<&str>,
) -> crate::diag::Result<Option<crate::codegen::DocInfo>> {
    use crate::ast::{Callee, Term};
    use crate::codegen;
    let mut source: Option<std::path::PathBuf> = None;
    for (path, module) in discover_sources(config)? {
        if module.join(".") == module_name {
            source = Some(path);
            break;
        }
    }
    if source.is_none() {
        source = find_installed_source(config, module_name);
    }
    let Some(source) = source else {
        return Ok(None);
    };
    let file = source.display().to_string();
    let text = std::fs::read_to_string(&source)
        .map_err(|e| crate::diag::Diagnostic::new(&file, crate::diag::Span::default(), e.to_string()))?;
    let term = crate::parser::parse_file(&file, &text)?;

    let mut module_doc: Option<codegen::DocInfo> = None;
    let mut fun_doc: Option<codegen::DocInfo> = None;
    let mut pending: Option<codegen::DocInfo> = None;
    for stmt in term.as_block() {
        if !stmt.is_call_named("defmodule") {
            continue;
        }
        let Term::Call(dm) = &stmt else { continue };
        let Some(body) = Term::keyword_arg(&dm.args, "do") else { continue };
        for inner in body.as_block() {
            let Term::Call(c) = &inner else { continue };
            let Callee::Name(name) = &c.callee else { continue };
            match name.as_str() {
                "@moduledoc" => {
                    let info = module_doc.get_or_insert_with(codegen::DocInfo::default);
                    codegen::merge_doc_value(&file, c, info, "@moduledoc")?;
                }
                "@doc" => {
                    let info = pending.get_or_insert_with(codegen::DocInfo::default);
                    codegen::merge_doc_value(&file, c, info, "@doc")?;
                }
                "@doc_trans" => {
                    if let Some(info) = pending.as_mut() {
                        codegen::merge_doc_trans(&file, c, info, "@doc_trans")?;
                    }
                }
                "@example" => {
                    let ex = codegen::example_from_args(&file, c)?;
                    pending
                        .get_or_insert_with(codegen::DocInfo::default)
                        .examples
                        .push(ex);
                }
                "@moduledoc_trans" => {
                    if let Some(info) = module_doc.as_mut() {
                        codegen::merge_doc_trans(&file, c, info, "@moduledoc_trans")?;
                    }
                }
                "def" | "defp" | "defmacro" => {
                    let head_name = c.args.first().and_then(def_head_name);
                    if let (Some(f), Some(h)) = (&fun, &head_name) {
                        if *f == h.as_str() && fun_doc.is_none() {
                            fun_doc = pending.take();
                        }
                    }
                    pending = None;
                }
                _ => {}
            }
        }
    }
    Ok(match fun {
        Some(_) => fun_doc,
        None => module_doc,
    })
}

/// Modules named by `require`/`import`, whose macros become visible.
fn macro_deps(term: &Term) -> Vec<String> {
    let mut deps = Vec::new();
    for stmt in term.as_block() {
        if stmt.is_call_named("defmodule") {
            if let Term::Call(dm) = &stmt {
                if let Some(body) = Term::keyword_arg(&dm.args, "do") {
                    for inner in body.as_block() {
                        if inner.is_call_named("require")
                            || inner.is_call_named("import")
                            || inner.is_call_named("use")
                        {
                            if let Term::Call(c) = &inner {
                                if let Some(Term::Alias(segs)) = c.args.first() {
                                    deps.push(segs.join("."));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    deps
}

/// Write compiled modules under `out_root`, only touching changed files.
pub fn write_outputs(modules: &[CompiledModule], out_root: &Path) -> Result<Vec<PathBuf>> {
    let mut written = Vec::new();
    for m in modules {
        let target = out_root.join(&m.py_path);
        if m.compile_time_only {
            // remove a stale file left by an earlier compiler version
            let _ = std::fs::remove_file(&target);
            continue;
        }
        if let Some(dir) = target.parent() {
            std::fs::create_dir_all(dir).map_err(|e| {
                Diagnostic::new(dir.display().to_string(), Span::default(), e.to_string())
            })?;
        }
        let unchanged = std::fs::read_to_string(&target)
            .map(|old| old == m.python)
            .unwrap_or(false);
        if !unchanged {
            std::fs::write(&target, &m.python).map_err(|e| {
                Diagnostic::new(target.display().to_string(), Span::default(), e.to_string())
            })?;
        }
        written.push(target);
    }
    Ok(written)
}

/// Marker + shipped sources for package projects (GEP-0006-R002/R004).
pub fn write_package_artifacts(
    modules: &[CompiledModule],
    out_root: &Path,
) -> Result<Vec<String>> {
    let mut by_top: BTreeMap<String, Vec<&CompiledModule>> = BTreeMap::new();
    for m in modules {
        let top = m.py_path.split('/').next().unwrap_or_default().to_string();
        by_top.entry(top).or_default().push(m);
    }
    let mut tops = Vec::new();
    for (top, mods) in by_top {
        let mut marker = format!(
            "schema = 1\ncompiler = \"{}\"\n",
            env!("CARGO_PKG_VERSION")
        );
        for m in &mods {
            let gan_rel = format!(
                "{top}/_gan/{}.gan",
                m.py_path.trim_end_matches(".py")
            );
            let dest = out_root.join(&gan_rel);
            if let Some(dir) = dest.parent() {
                std::fs::create_dir_all(dir).map_err(|e| {
                    Diagnostic::new(dir.display().to_string(), Span::default(), e.to_string())
                })?;
            }
            std::fs::copy(&m.source, &dest).map_err(|e| {
                Diagnostic::new(dest.display().to_string(), Span::default(), e.to_string())
            })?;
            marker.push_str("\n[[modules]]\n");
            marker.push_str(&format!("name = \"{}\"\n", m.module.join(".")));
            if !m.compile_time_only {
                marker.push_str(&format!("python = \"{}\"\n", m.py_path));
            }
            marker.push_str(&format!("source = \"{gan_rel}\"\n"));
        }
        let path = out_root.join(&top).join("gandora.toml");
        std::fs::write(&path, marker).map_err(|e| {
            Diagnostic::new(path.display().to_string(), Span::default(), e.to_string())
        })?;
        tops.push(top);
    }
    Ok(tops)
}

/// Parse a marker into (module name, source path) entries; unknown schema
/// yields nothing (GEP-0006-R004).
fn parse_marker(text: &str) -> Vec<(String, String)> {
    let mut entries = Vec::new();
    let mut name: Option<String> = None;
    let mut source: Option<String> = None;
    let mut schema_ok = false;
    let flush = |name: &mut Option<String>, source: &mut Option<String>,
                     entries: &mut Vec<(String, String)>| {
        if let (Some(n), Some(s)) = (name.take(), source.take()) {
            entries.push((n, s));
        }
    };
    for line in text.lines() {
        let line = line.trim();
        if line == "[[modules]]" {
            flush(&mut name, &mut source, &mut entries);
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            let v = v.trim().trim_matches('"');
            match k.trim() {
                "schema" => schema_ok = v == "1",
                "name" => name = Some(v.to_string()),
                "source" => source = Some(v.to_string()),
                _ => {}
            }
        }
    }
    flush(&mut name, &mut source, &mut entries);
    if schema_ok {
        entries
    } else {
        Vec::new()
    }
}

fn site_packages_dirs(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for lib in ["lib", "Lib"] {
        let lib_dir = root.join(".venv").join(lib);
        let direct = lib_dir.join("site-packages");
        if direct.is_dir() {
            out.push(direct);
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(&lib_dir) {
            for e in entries.flatten() {
                let sp = e.path().join("site-packages");
                if sp.is_dir() {
                    out.push(sp);
                }
            }
        }
    }
    out
}

/// All installed marker modules: gandora name -> dotted python path
/// (GEP-0006-R005A). Static reads only.
pub fn installed_module_map(config: &Config) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for sp in site_packages_dirs(&config.root) {
        let Ok(entries) = std::fs::read_dir(&sp) else { continue };
        let mut dirs: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
        dirs.sort();
        for dir in dirs {
            let Ok(text) = std::fs::read_to_string(dir.join("gandora.toml")) else {
                continue;
            };
            for (name, python) in parse_marker_python(&text) {
                out.entry(name)
                    .or_insert_with(|| python.trim_end_matches(".py").replace('/', "."));
            }
        }
    }
    out
}

/// Marker (name, python) entries for runtime resolution.
fn parse_marker_python(text: &str) -> Vec<(String, String)> {
    let mut entries = Vec::new();
    let mut name: Option<String> = None;
    let mut python: Option<String> = None;
    let mut schema_ok = false;
    let flush = |name: &mut Option<String>, python: &mut Option<String>,
                     entries: &mut Vec<(String, String)>| {
        if let (Some(n), Some(p)) = (name.take(), python.take()) {
            entries.push((n, p));
        } else {
            name.take();
            python.take();
        }
    };
    for line in text.lines() {
        let line = line.trim();
        if line == "[[modules]]" {
            flush(&mut name, &mut python, &mut entries);
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            let v = v.trim().trim_matches('"');
            match k.trim() {
                "schema" => schema_ok = v == "1",
                "name" => name = Some(v.to_string()),
                "python" => python = Some(v.to_string()),
                _ => {}
            }
        }
    }
    flush(&mut name, &mut python, &mut entries);
    if schema_ok {
        entries
    } else {
        Vec::new()
    }
}

/// Locate a module shipped by an installed package by scanning markers in
/// the project's site-packages (GEP-0006-R006). Static reads only.
pub fn find_installed_source(config: &Config, module_name: &str) -> Option<PathBuf> {
    for sp in site_packages_dirs(&config.root) {
        let Ok(entries) = std::fs::read_dir(&sp) else { continue };
        let mut dirs: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
        dirs.sort();
        for dir in dirs {
            let marker_path = dir.join("gandora.toml");
            let Ok(text) = std::fs::read_to_string(&marker_path) else { continue };
            for (name, source) in parse_marker(&text) {
                if name == module_name {
                    let path = sp.join(source);
                    if path.exists() {
                        return Some(path);
                    }
                }
            }
        }
    }
    None
}

/// The interpreter for `gan run`: `.venv/bin/python`, `uv run python`, or python3.
pub fn interpreter_command(project_root: &Path) -> (String, Vec<String>) {
    let venv = project_root.join(".venv/bin/python");
    if venv.exists() {
        return (venv.display().to_string(), vec![]);
    }
    if project_root.join("pyproject.toml").exists() && which("uv") {
        return (
            "uv".to_string(),
            vec!["run".into(), "--project".into(), project_root.display().to_string(), "python".into()],
        );
    }
    ("python3".to_string(), vec![])
}

fn which(bin: &str) -> bool {
    std::env::var_os("PATH")
        .map(|paths| {
            std::env::split_paths(&paths).any(|p| {
                let c = p.join(bin);
                c.exists()
            })
        })
        .unwrap_or(false)
}

/// Modules that `camel_to_snake` would map to the same Python path must be
/// rejected (GEP-0001-R015 collision rule).
pub fn check_collisions(sources: &[(PathBuf, Vec<String>)]) -> Result<()> {
    let mut seen: BTreeMap<String, &PathBuf> = BTreeMap::new();
    let mut names: BTreeSet<String> = BTreeSet::new();
    for (path, module) in sources {
        let name = module.join(".");
        if !names.insert(name.clone()) {
            return Err(Diagnostic::new(
                path.display().to_string(),
                Span::default(),
                format!("duplicate module {name}"),
            ));
        }
        let py = module_py_path(module);
        if let Some(other) = seen.get(&py) {
            return Err(Diagnostic::new(
                path.display().to_string(),
                Span::default(),
                format!(
                    "modules {} and {} map to the same Python module {}",
                    path.display(),
                    other.display(),
                    py
                ),
            ));
        }
        seen.insert(py, path);
    }
    Ok(())
}

pub fn snake_path_for_camel(s: &str) -> String {
    camel_to_snake(s)
}
