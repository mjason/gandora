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
        Term::PyRef(m) => {
            let name: PyObject = "__pyref__".into_py(py);
            let meta = PyList::empty_bound(py).into_py(py);
            let args: PyObject = PyList::new_bound(py, [m.into_py(py)]).into_py(py);
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
) -> Result<Term, PyErr> {
    let term = parse_file(path, source).map_err(raise)?;
    let (_, _, mut macros) = context(root);
    for (k, v) in collect_macros(path, &term).map_err(raise)? {
        macros.insert(k, v);
    }
    let mut ex = Expander::new(path, macros);
    ex.expand_module(&term).map_err(raise)
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
    let expanded = expand_source(source, path, root)?;
    Ok(term_to_py(py, &expanded))
}

#[pyfunction]
#[pyo3(signature = (source, path = "nofile", root = None))]
fn compile_string(source: &str, path: &str, root: Option<&str>) -> PyResult<String> {
    let expanded = expand_source(source, path, root)?;
    let (project_modules, installed, _) = context(root);
    let mut cg = Codegen::new(path, vec![]);
    cg.project_modules = project_modules;
    cg.installed_modules = installed;
    cg.compile(&expanded).map_err(raise)
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
        Ok(expanded) => {
            let (project_modules, installed, _) = context(root);
            let mut cg = Codegen::new(path, vec![]);
            cg.project_modules = project_modules;
            cg.installed_modules = installed;
            match cg.compile(&expanded) {
                Err(d) => push(&out, d.span.line, d.span.col, "error", &d.message),
                Ok(_) => {
                    for w in &cg.warnings {
                        push(&out, 0, 0, "warning", w);
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
                    e.set_item("message", w)?;
                    e.set_item("path", m.source.display().to_string())?;
                    e.set_item("line", 0u32)?;
                    e.set_item("col", 0u32)?;
                    e.set_item("severity", "warning")?;
                    let _ = out.append(e);
                }
            }
        }
    }
    Ok(out.into_py(py))
}
