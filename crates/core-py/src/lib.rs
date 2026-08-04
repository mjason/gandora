//! The `gandora_core` Python extension module (GEP-0012).
//!
//! Quoted terms are exported in the Elixir encoding of GEP-0012-R004:
//! 3-tuples are syntax nodes, 2-tuples are data.

use std::collections::BTreeMap;
use std::path::Path;

use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyTuple};

use compiler::ast::{Callee, StrPart, Term};
use compiler::codegen::Codegen;
use compiler::diag::Diagnostic;
use compiler::expander::{collect_macros, Expander, MacroTable};
use compiler::parser::parse_file;
use compiler::project;

pyo3::create_exception!(gandora_core, CompileError, PyException);

fn raise(d: Diagnostic) -> PyErr {
    // args carry (message, path, line, col) per GEP-0012-R003
    CompileError::new_err((d.message, d.file, d.span.line, d.span.col))
}

fn meta<'py>(py: Python<'py>, line: u32, col: u32) -> Bound<'py, PyDict> {
    let m = PyDict::new_bound(py);
    let _ = m.set_item("line", line);
    let _ = m.set_item("col", col);
    m
}

fn node(py: Python, head: PyObject, line: u32, col: u32, args: Vec<PyObject>) -> PyObject {
    let m = meta(py, line, col);
    let list = PyList::new_bound(py, args);
    PyTuple::new_bound(py, [head, m.into_py(py), list.into_py(py)]).into_py(py)
}

fn term_to_py(py: Python, t: &Term) -> PyObject {
    match t {
        Term::Int(n) => n.into_py(py),
        Term::Float(f) => f.into_py(py),
        Term::Bool(b) => b.into_py(py),
        Term::Nil => py.None(),
        Term::Atom(a) => a.into_py(py),
        // {:__pyref__, [], ["math"]} — a $module reference (GEP-0003-R001)
        Term::PyRef(m, bounded) => {
            let name: PyObject = "__pyref__".into_py(py);
            let meta = PyList::empty_bound(py).into_py(py);
            let args: PyObject = if *bounded {
                PyList::new_bound(py, [m.into_py(py), true.into_py(py)]).into_py(py)
            } else {
                PyList::new_bound(py, [m.into_py(py)]).into_py(py)
            };
            PyTuple::new_bound(py, [name, meta, args]).into_py(py)
        }
        Term::Str(parts) => match t.as_plain_str() {
            Some(s) => s.into_py(py),
            None => {
                let items = parts
                    .iter()
                    .map(|p| match p {
                        StrPart::Text(s) => s.into_py(py),
                        StrPart::Interp(e) => term_to_py(py, e),
                    })
                    .collect();
                node(py, "__interp__".into_py(py), 0, 0, items)
            }
        },
        Term::Var(name, ctx) => {
            let c = match ctx {
                Some(id) => id.into_py(py),
                None => py.None(),
            };
            PyTuple::new_bound(py, [name.into_py(py), meta(py, 0, 0).into_py(py), c]).into_py(py)
        }
        Term::Alias(segs) => {
            let items = segs.iter().map(|s| s.into_py(py)).collect();
            node(py, "__aliases__".into_py(py), 0, 0, items)
        }
        Term::List(items) => {
            PyList::new_bound(py, items.iter().map(|i| term_to_py(py, i))).into_py(py)
        }
        Term::Tuple(items) => {
            if items.len() == 2 {
                PyTuple::new_bound(py, [term_to_py(py, &items[0]), term_to_py(py, &items[1])])
                    .into_py(py)
            } else {
                let elems = items.iter().map(|i| term_to_py(py, i)).collect();
                node(py, "{}".into_py(py), 0, 0, elems)
            }
        }
        Term::Map(entries) => {
            let pairs = entries
                .iter()
                .map(|(k, v)| {
                    PyTuple::new_bound(py, [term_to_py(py, k), term_to_py(py, v)]).into_py(py)
                })
                .collect();
            node(py, "%{}".into_py(py), 0, 0, pairs)
        }
        Term::Pair(k, v) => {
            PyTuple::new_bound(py, [k.into_py(py), term_to_py(py, v)]).into_py(py)
        }
        Term::Call(call) => {
            let (line, col) = (call.span.line, call.span.col);
            let args: Vec<PyObject> = call.args.iter().map(|a| term_to_py(py, a)).collect();
            match &call.callee {
                Callee::Name(n) => node(py, n.into_py(py), line, col, args),
                Callee::Dot { base, name, .. } => {
                    let head = node(
                        py,
                        ".".into_py(py),
                        line,
                        col,
                        vec![term_to_py(py, base), name.into_py(py)],
                    );
                    node(py, head, line, col, args)
                }
                Callee::Apply(f) => {
                    let head = node(py, ".".into_py(py), line, col, vec![term_to_py(py, f)]);
                    node(py, head, line, col, args)
                }
            }
        }
    }
}

/// Project + installed context for a given root, when provided.
fn context(
    root: Option<&str>,
) -> (
    std::collections::BTreeSet<String>,
    BTreeMap<String, String>,
    MacroTable,
) {
    let mut project_modules = std::collections::BTreeSet::new();
    let mut installed = BTreeMap::new();
    let mut macros = MacroTable::new();
    if let Some(root) = root {
        let config = match project::find_config(Path::new(root)) {
            Ok(Some(c)) => c,
            _ => project::Config::default_at(Path::new(root).to_path_buf()),
        };
        installed = project::installed_module_map(&config);
        if let Ok(sources) = project::discover_sources(&config) {
            for (path, module) in sources {
                project_modules.insert(module.join("."));
                if let Ok(text) = std::fs::read_to_string(&path) {
                    let name = path.display().to_string();
                    if let Ok(term) = parse_file(&name, &text) {
                        if let Ok(table) = collect_macros(&name, &term) {
                            for (k, v) in table {
                                macros.entry(k).or_insert(v);
                            }
                        }
                    }
                }
            }
        }
    }
    (project_modules, installed, macros)
}

fn expand_source(
    source: &str,
    path: &str,
    root: Option<&str>,
) -> Result<(Term, Vec<Diagnostic>), PyErr> {
    let term = parse_file(path, source).map_err(raise)?;
    let (_, _, mut macros) = context(root);
    for (k, v) in collect_macros(path, &term).map_err(raise)? {
        macros.insert(k, v);
    }
    let mut ex = Expander::new(path, macros);
    let expanded = ex.expand_module(&term).map_err(raise)?;
    Ok((expanded, std::mem::take(&mut ex.warnings)))
}

#[pyfunction]
fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[pyfunction]
#[pyo3(signature = (source, path = "nofile"))]
fn parse(py: Python, source: &str, path: &str) -> PyResult<PyObject> {
    let term = parse_file(path, source).map_err(raise)?;
    Ok(term_to_py(py, &term))
}

#[pyfunction]
#[pyo3(signature = (source, path = "nofile", root = None))]
fn expand(py: Python, source: &str, path: &str, root: Option<&str>) -> PyResult<PyObject> {
    let (expanded, _) = expand_source(source, path, root)?;
    Ok(term_to_py(py, &expanded))
}

#[pyfunction]
#[pyo3(signature = (source, path = "nofile", root = None))]
fn compile_string(source: &str, path: &str, root: Option<&str>) -> PyResult<String> {
    let (expanded, _) = expand_source(source, path, root)?;
    let (project_modules, installed, _) = context(root);
    let mut cg = Codegen::new(path, vec![]);
    cg.project_modules = project_modules;
    cg.installed_modules = installed;
    cg.compile(&expanded).map_err(raise)
}


/// The full lexical stream for tooling (GEP-0016-R001): comments kept,
/// newlines uncollapsed, end spans included.
#[pyfunction]
#[pyo3(signature = (source, path = "nofile"))]
fn tokens(py: Python, source: &str, path: &str) -> PyResult<PyObject> {
    use compiler::lexer::{Lexer, Tok};
    let toks = Lexer::new(path, source)
        .tokenize_full()
        .map_err(raise)?;
    let out = PyList::empty_bound(py);
    for (tok, start, end) in &toks {
        let (kind, value): (&str, String) = match tok {
            Tok::Int(n) => ("int", n.to_string()),
            Tok::Float(f) => ("float", f.to_string()),
            Tok::Str(parts) => ("str", format!("{parts:?}")),
            Tok::Sigil(name, parts) => (
                "sigil",
                if name.starts_with('$') || name.starts_with('%') {
                    format!("{name}{parts:?}")
                } else {
                    format!("~{name}{parts:?}")
                },
            ),
            Tok::Atom(a) => ("atom", a.clone()),
            Tok::PyRef(m, _) => ("pyref", m.clone()),
            Tok::Ident(i) => ("ident", i.clone()),
            Tok::UpIdent(i) => ("upident", i.clone()),
            Tok::KwKey(k) => ("kwkey", k.clone()),
            Tok::Comment(c) => ("comment", c.clone()),
            Tok::Op(o) => ("op", o.to_string()),
            Tok::Kw(k) => ("kw", k.to_string()),
            Tok::Newline => ("newline", String::new()),
            Tok::Eof => ("eof", String::new()),
        };
        let e = PyDict::new_bound(py);
        let _ = e.set_item("kind", kind);
        let _ = e.set_item("value", value);
        let _ = e.set_item("line", start.line);
        let _ = e.set_item("col", start.col);
        let _ = e.set_item("end_line", end.line);
        let _ = e.set_item("end_col", end.col);
        let _ = out.append(e);
    }
    Ok(out.into_py(py))
}


/// Documentation lookup (GEP-0007/GEP-0015): `doc("Stats.mean")` returns
/// {"label", "entries" (locale -> markdown), "examples", "meta", "hidden"}
/// or None when the module/function has no docs.
#[pyfunction]
#[pyo3(signature = (target, root = None))]
fn doc(py: Python, target: &str, root: Option<&str>) -> PyResult<PyObject> {
    let segs: Vec<&str> = target.split('.').collect();
    let (module_name, fun) = match segs.last() {
        Some(last) if last.chars().next().is_some_and(|c| c.is_lowercase()) => {
            (segs[..segs.len() - 1].join("."), Some((*last).to_string()))
        }
        _ => (target.to_string(), None),
    };
    if module_name.is_empty() {
        return Ok(py.None());
    }
    let root_dir = std::path::Path::new(root.unwrap_or("."));
    let config =
        compiler::project::load_config(&root_dir.join("gandora.jsonc")).map_err(raise)?;
    let info = compiler::project::find_doc(&config, &module_name, fun.as_deref()).map_err(raise)?;
    let Some(info) = info else {
        return Ok(py.None());
    };
    let out = PyDict::new_bound(py);
    let _ = out.set_item("label", target);
    let _ = out.set_item("hidden", info.hidden);
    let entries = PyDict::new_bound(py);
    for (loc, text) in &info.entries {
        let _ = entries.set_item(loc, text);
    }
    let _ = out.set_item("entries", entries);
    let _ = out.set_item("examples", PyList::new_bound(py, &info.examples));
    let meta = PyList::empty_bound(py);
    for (k, v) in &info.meta {
        let _ = meta.append(PyTuple::new_bound(py, [k, v]));
    }
    let _ = out.set_item("meta", meta);
    let _ = out.set_item("specs", PyList::new_bound(py, &info.specs));
    let _ = out.set_item("tco", info.tco.clone());
    let params = PyList::empty_bound(py);
    for (name, entries) in &info.params {
        let e = PyDict::new_bound(py);
        let _ = e.set_item("name", name);
        let locs = PyDict::new_bound(py);
        for (loc, text) in entries {
            let _ = locs.set_item(loc, text);
        }
        let _ = e.set_item("entries", locs);
        let _ = params.append(e);
    }
    let _ = out.set_item("params", params);
    let sigs: Vec<String> = match &fun {
        Some(f) => compiler::project::module_symbols(&config, &module_name)
            .unwrap_or_default()
            .into_iter()
            .filter(|s| s.name == *f)
            .map(|s| s.head)
            .collect(),
        None => vec![format!("defmodule {module_name}")],
    };
    let _ = out.set_item("signatures", PyList::new_bound(py, sigs));
    Ok(out.into_py(py))
}

/// Where a module or function is defined (GEP-0015-R006):
/// {"path", "line", "col"} or None.
#[pyfunction]
#[pyo3(signature = (target, root = None))]
fn definition(py: Python, target: &str, root: Option<&str>) -> PyResult<PyObject> {
    let segs: Vec<&str> = target.split('.').collect();
    let (module_name, fun) = match segs.last() {
        Some(last) if last.chars().next().is_some_and(|c| c.is_lowercase()) => {
            (segs[..segs.len() - 1].join("."), Some((*last).to_string()))
        }
        _ => (target.to_string(), None),
    };
    if module_name.is_empty() {
        return Ok(py.None());
    }
    let root_dir = std::path::Path::new(root.unwrap_or("."));
    let config =
        compiler::project::load_config(&root_dir.join("gandora.jsonc")).map_err(raise)?;
    match compiler::project::find_definition(&config, &module_name, fun.as_deref())
        .map_err(raise)?
    {
        None => Ok(py.None()),
        Some((path, line, col)) => {
            let out = PyDict::new_bound(py);
            let _ = out.set_item("path", path);
            let _ = out.set_item("line", line);
            let _ = out.set_item("col", col);
            Ok(out.into_py(py))
        }
    }
}

/// A developer-local preference from `gandora.local.jsonc`
/// (GEP-0015-R015), or None.
#[pyfunction]
#[pyo3(signature = (root, key))]
fn local_pref(root: &str, key: &str) -> Option<String> {
    compiler::project::local_pref(std::path::Path::new(root), key)
}

/// Every reference to `Module.fun` across the project (GEP-0015-R012):
/// [{"path","line","col","is_def"}] in source order.
#[pyfunction]
#[pyo3(signature = (target, root = None))]
fn references(py: Python, target: &str, root: Option<&str>) -> PyResult<PyObject> {
    let segs: Vec<&str> = target.split('.').collect();
    let (module_name, fun) = match segs.last() {
        Some(last) if last.chars().next().is_some_and(|c| c.is_lowercase()) => {
            (segs[..segs.len() - 1].join("."), (*last).to_string())
        }
        _ => return Ok(PyList::empty_bound(py).into_py(py)),
    };
    if module_name.is_empty() {
        return Ok(PyList::empty_bound(py).into_py(py));
    }
    let root_dir = std::path::Path::new(root.unwrap_or("."));
    let config =
        compiler::project::load_config(&root_dir.join("gandora.jsonc")).map_err(raise)?;
    let list = PyList::empty_bound(py);
    for (path, line, col, is_def) in
        compiler::project::find_references(&config, &module_name, &fun).map_err(raise)?
    {
        let e = PyDict::new_bound(py);
        let _ = e.set_item("path", path);
        let _ = e.set_item("line", line);
        let _ = e.set_item("col", col);
        let _ = e.set_item("is_def", is_def);
        let _ = list.append(e);
    }
    Ok(list.into_py(py))
}

/// Project-wide symbol search (GEP-0015-R013):
/// [{"module","path","name","kind","line","head","doc"}].
#[pyfunction]
#[pyo3(signature = (query, root = None))]
fn wsymbols(py: Python, query: &str, root: Option<&str>) -> PyResult<PyObject> {
    let root_dir = std::path::Path::new(root.unwrap_or("."));
    let config =
        compiler::project::load_config(&root_dir.join("gandora.jsonc")).map_err(raise)?;
    let list = PyList::empty_bound(py);
    for (module, path, s) in
        compiler::project::workspace_symbols(&config, query).map_err(raise)?
    {
        let e = PyDict::new_bound(py);
        let _ = e.set_item("module", module);
        let _ = e.set_item("path", path);
        let _ = e.set_item("name", s.name);
        let _ = e.set_item("kind", s.kind);
        let _ = e.set_item("line", s.line);
        let _ = e.set_item("head", s.head);
        let _ = e.set_item("doc", s.doc_head);
        let _ = list.append(e);
    }
    Ok(list.into_py(py))
}

/// Every definition of a module, for outlines and completion
/// (GEP-0015-R008): [{"name","kind","line","head","doc"}].
#[pyfunction]
#[pyo3(signature = (module, root = None))]
fn symbols(py: Python, module: &str, root: Option<&str>) -> PyResult<PyObject> {
    let root_dir = std::path::Path::new(root.unwrap_or("."));
    let config =
        compiler::project::load_config(&root_dir.join("gandora.jsonc")).map_err(raise)?;
    let list = PyList::empty_bound(py);
    for s in compiler::project::module_symbols(&config, module).map_err(raise)? {
        let e = PyDict::new_bound(py);
        let _ = e.set_item("name", s.name);
        let _ = e.set_item("kind", s.kind);
        let _ = e.set_item("line", s.line);
        let _ = e.set_item("head", s.head);
        let _ = e.set_item("doc", s.doc_head);
        let _ = list.append(e);
    }
    Ok(list.into_py(py))
}

#[pyfunction]
#[pyo3(signature = (source, path = "nofile", root = None))]
fn diagnostics(py: Python, source: &str, path: &str, root: Option<&str>) -> PyResult<PyObject> {
    let out = PyList::empty_bound(py);
    let push = |list: &Bound<PyList>, line: u32, col: u32, severity: &str, message: &str| {
        let e = PyDict::new_bound(py);
        let _ = e.set_item("message", message);
        let _ = e.set_item("line", line);
        let _ = e.set_item("col", col);
        let _ = e.set_item("severity", severity);
        let _ = list.append(e);
    };
    match expand_source(source, path, root) {
        Err(err) => {
            let value = err.value_bound(py);
            let (message, _p, line, col): (String, String, u32, u32) = value
                .getattr("args")
                .and_then(|a| a.extract())
                .unwrap_or((value.to_string(), path.to_string(), 0, 0));
            push(&out, line, col, "error", &message);
        }
        Ok((expanded, macro_warnings)) => {
            let (project_modules, installed, _) = context(root);
            let mut cg = Codegen::new(path, vec![]);
            cg.project_modules = project_modules;
            cg.installed_modules = installed;
            for w in &macro_warnings {
                push(&out, w.span.line, w.span.col, "warning", &w.message);
            }
            match cg.compile(&expanded) {
                Err(d) => push(&out, d.span.line, d.span.col, "error", &d.message),
                Ok(_) => {
                    for w in &cg.warnings {
                        push(&out, w.span.line, w.span.col, "warning", &w.message);
                    }
                }
            }
        }
    }
    Ok(out.into_py(py))
}

#[pyfunction]
#[pyo3(signature = (source, root = None))]
fn compile_snippet(source: &str, root: Option<&str>) -> PyResult<String> {
    let term = parse_file("<snippet>", source).map_err(raise)?;
    let (project_modules, installed, macros) = context(root);
    let mut ex = Expander::new("<snippet>", macros);
    let expanded = ex.expand_module(&term).map_err(raise)?;
    compiler::codegen::compile_snippet("<snippet>", &expanded, project_modules, installed)
        .map_err(raise)
}

#[pyfunction]
fn resolve(py: Python, root: &str, module_name: &str) -> PyResult<PyObject> {
    let config = match project::find_config(Path::new(root)) {
        Ok(Some(c)) => c,
        _ => project::Config::default_at(Path::new(root).to_path_buf()),
    };
    let out = PyDict::new_bound(py);
    let segs: Vec<String> = module_name.split('.').map(|s| s.to_string()).collect();
    let mechanical = compiler::codegen::module_py_path(&segs);
    let mut kind = "mechanical";
    let mut python = mechanical.clone();
    let mut source: Option<String> = None;
    if let Ok(sources) = project::discover_sources(&config) {
        for (path, module) in &sources {
            if module.join(".") == module_name {
                kind = "project";
                source = Some(path.display().to_string());
                if let Some(prefix) = &config.py_package {
                    python = format!("{prefix}.{mechanical}");
                }
            }
        }
    }
    if kind == "mechanical" {
        if let Some(p) = project::installed_module_map(&config).get(module_name) {
            kind = "installed";
            python = p.clone();
            source = project::find_installed_source(&config, module_name)
                .map(|p| p.display().to_string());
        }
    }
    out.set_item("kind", kind)?;
    out.set_item("python", python)?;
    out.set_item("source", source)?;
    Ok(out.into_py(py))
}

#[pymodule]
fn gandora_core(py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    m.add("CompileError", py.get_type_bound::<CompileError>())?;
    m.add_function(wrap_pyfunction!(version, m)?)?;
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    m.add_function(wrap_pyfunction!(expand, m)?)?;
    m.add_function(wrap_pyfunction!(compile_string, m)?)?;
    m.add_function(wrap_pyfunction!(diagnostics, m)?)?;
    m.add_function(wrap_pyfunction!(tokens, m)?)?;
    m.add_function(wrap_pyfunction!(doc, m)?)?;
    m.add_function(wrap_pyfunction!(definition, m)?)?;
    m.add_function(wrap_pyfunction!(symbols, m)?)?;
    m.add_function(wrap_pyfunction!(references, m)?)?;
    m.add_function(wrap_pyfunction!(wsymbols, m)?)?;
    m.add_function(wrap_pyfunction!(local_pref, m)?)?;
    m.add_function(wrap_pyfunction!(compile_snippet, m)?)?;
    m.add_function(wrap_pyfunction!(resolve, m)?)?;
    m.add_function(wrap_pyfunction!(build, m)?)?;
    m.add_function(wrap_pyfunction!(check, m)?)?;
    Ok(())
}

#[pyfunction]
#[pyo3(signature = (root, out = None))]
fn build(py: Python, root: &str, out: Option<&str>) -> PyResult<PyObject> {
    let config = match project::find_config(Path::new(root)) {
        Ok(Some(c)) => c,
        _ => project::Config::default_at(Path::new(root).to_path_buf()),
    };
    let modules = project::compile_project(&config).map_err(raise)?;
    let out_root = match out {
        Some(o) => Path::new(o).to_path_buf(),
        None => config.root.join(&config.out_dir),
    };
    project::write_outputs(&modules, &out_root).map_err(raise)?;
    if config.package && out.is_none() {
        project::write_package_artifacts(&modules, &out_root).map_err(raise)?;
    }
    let list = PyList::empty_bound(py);
    for m in &modules {
        let e = PyDict::new_bound(py);
        e.set_item("source", m.source.display().to_string())?;
        e.set_item("module", m.module.join("."))?;
        e.set_item(
            "python",
            if m.compile_time_only {
                None
            } else {
                Some(out_root.join(&m.py_path).display().to_string())
            },
        )?;
        let _ = list.append(e);
    }
    Ok(list.into_py(py))
}

#[pyfunction]
fn check(py: Python, root: &str) -> PyResult<PyObject> {
    let config = match project::find_config(Path::new(root)) {
        Ok(Some(c)) => c,
        _ => project::Config::default_at(Path::new(root).to_path_buf()),
    };
    let out = PyList::empty_bound(py);
    match project::compile_project(&config) {
        Err(d) => {
            let e = PyDict::new_bound(py);
            e.set_item("message", &d.message)?;
            e.set_item("path", &d.file)?;
            e.set_item("line", d.span.line)?;
            e.set_item("col", d.span.col)?;
            e.set_item("severity", "error")?;
            let _ = out.append(e);
        }
        Ok(modules) => {
            for m in &modules {
                for w in &m.warnings {
                    let e = PyDict::new_bound(py);
                    e.set_item("message", &w.message)?;
                    e.set_item("path", &w.file)?;
                    e.set_item("line", w.span.line)?;
                    e.set_item("col", w.span.col)?;
                    e.set_item("severity", "warning")?;
                    let _ = out.append(e);
                }
            }
        }
    }
    Ok(out.into_py(py))
}
