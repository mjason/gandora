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
}

impl Config {
    pub fn default_at(root: PathBuf) -> Self {
        Config {
            root,
            source: vec!["src".into()],
            out_dir: "dist".into(),
            target_python: "3.12".into(),
            exclude: Vec::new(),
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
}

/// Parse, expand, and compile every module of a project.
pub fn compile_project(config: &Config) -> Result<Vec<CompiledModule>> {
    let sources = discover_sources(config)?;
    compile_files(&sources)
}

pub fn compile_files(sources: &[(PathBuf, Vec<String>)]) -> Result<Vec<CompiledModule>> {
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
            }
        }
        let mut expander = Expander::new(&file, table);
        let expanded = expander.expand_module(&p.term)?;
        let mut cg = Codegen::new(&file, p.module.clone());
        let python = cg.compile(&expanded)?;
        out.push(CompiledModule {
            source: p.path.clone(),
            module: p.module.clone(),
            py_path: module_py_path(&p.module).replace('.', "/") + ".py",
            python,
            compile_time_only: cg.compile_time_only,
        });
    }
    Ok(out)
}

/// Modules named by `require`/`import`, whose macros become visible.
fn macro_deps(term: &Term) -> Vec<String> {
    let mut deps = Vec::new();
    for stmt in term.as_block() {
        if stmt.is_call_named("defmodule") {
            if let Term::Call(dm) = &stmt {
                if let Some(body) = Term::keyword_arg(&dm.args, "do") {
                    for inner in body.as_block() {
                        if inner.is_call_named("require") || inner.is_call_named("import") {
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
