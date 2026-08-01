//! End-to-end tests driving the real `gan` binary (GEP-0001 Conformance).

use std::path::{Path, PathBuf};
use std::process::Command;

fn gan() -> Command {
    Command::new(env!("CARGO_BIN_EXE_gan"))
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("gandora-e2e-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn python3() -> Option<&'static str> {
    if Command::new("python3").arg("--version").output().is_ok() {
        Some("python3")
    } else {
        None
    }
}

fn run_generated(root: &Path, script: &Path) -> String {
    let Some(py) = python3() else {
        panic!("python3 is required for e2e tests");
    };
    let out = Command::new(py)
        .arg(script)
        .env("PYTHONPATH", root)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "python failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

#[test]
fn init_creates_a_runnable_project() {
    let dir = temp_dir("init");
    let project = dir.join("app");
    let out = gan().arg("init").arg(&project).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    for f in [
        "gandora.jsonc",
        "pyproject.toml",
        ".python-version",
        ".gitignore",
        "src/main.gan",
    ] {
        assert!(project.join(f).exists(), "missing {f}");
    }
    // compile and execute the starter module without uv
    let out = gan()
        .current_dir(&project)
        .args(["build"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let stdout = run_generated(&project.join("dist"), &project.join("dist/main.py"));
    assert!(stdout.contains("Hello from Gandora!"), "{stdout}");
    assert!(stdout.contains("2 + 2 * 2 = 6"), "{stdout}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn tour_example_produces_expected_output() {
    let tour = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/tour");
    let out = gan().current_dir(&tour).arg("build").output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let dist = tour.join("dist");
    let stdout = run_generated(&dist, &dist.join("app/cli.py"));
    assert!(stdout.contains("fact(10) = 3628800"), "{stdout}");
    assert!(stdout.contains("classify(-3) = negative"), "{stdout}");
    assert!(stdout.contains("norm([3, 4]) = 5.0"), "{stdout}");
    assert!(stdout.contains("unless_nil(nil, :fallback) = fallback"), "{stdout}");
    assert!(stdout.contains("destructured: 1 2 10 [20, 30]"), "{stdout}");
}

#[test]
fn build_is_deterministic() {
    let tour = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/tour");
    let out = gan().current_dir(&tour).arg("build").output().unwrap();
    assert!(out.status.success());
    let first = std::fs::read_to_string(tour.join("dist/app/mathy.py")).unwrap();
    let out = gan().current_dir(&tour).arg("build").output().unwrap();
    assert!(out.status.success());
    let second = std::fs::read_to_string(tour.join("dist/app/mathy.py")).unwrap();
    assert_eq!(first, second, "builds must be byte-identical (GEP-0001-R024)");
}

#[test]
fn check_reports_diagnostics_with_spans() {
    let dir = temp_dir("diag");
    let bad = dir.join("bad.gan");
    std::fs::write(&bad, "defmodule Bad do\n  def f(x) do\n    receive(x)\n  end\nend\n").unwrap();
    let out = gan().arg("check").arg(&bad).output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("bad.gan:3:"), "{stderr}");
    assert!(stderr.contains("GEP-0001-R007"), "{stderr}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn expand_prints_surface_syntax() {
    let tour = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/tour");
    let out = gan()
        .current_dir(&tour)
        .args(["expand", "src/app/cli.gan"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("case nil do"), "{stdout}");
    let _ = stdout;
}

#[test]
fn cli_misuse_exits_2() {
    let out = gan().arg("frobnicate").output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    let out = gan().output().unwrap();
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn structs_and_module_attributes_run() {
    let tour = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/tour");
    let out = gan().current_dir(&tour).arg("build").output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let dist = tour.join("dist");
    let stdout = run_generated(&dist, &dist.join("app/shop.py"));
    assert!(stdout.contains("expensive: 100"), "{stdout}");
    assert!(stdout.contains("keyboard now 50, tags ['sale']"), "{stdout}");
    assert!(stdout.contains("registered routes: ['/sale']"), "{stdout}");
}

#[test]
fn sigils_run() {
    let tour = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/tour");
    let out = gan().current_dir(&tour).arg("build").output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let dist = tour.join("dist");
    let stdout = run_generated(&dist, &dist.join("sigils.py"));
    assert!(stdout.contains("['gandora', 'elixir', 'python']"), "{stdout}");
    assert!(stdout.contains("no need to escape \"quotes\" here"), "{stdout}");
    assert!(stdout.contains("['hello', '世界', 'world']"), "{stdout}");
    assert!(stdout.contains("sum of squares: 285"), "{stdout}");
    assert!(stdout.contains("[2, 4, 6]"), "{stdout}");
}

#[test]
fn generated_snapshot_is_current() {
    // examples/tour/generated/ is the checked-in compilation result shown in
    // the docs; this test fails when the compiler output drifts from it.
    // Refresh with: cd examples/tour && gan build && cp -r dist/* generated/
    let tour = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/tour");
    let out = gan().current_dir(&tour).arg("build").output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let mut stack = vec![tour.join("generated")];
    let mut checked = 0;
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "py") {
                let rel = path.strip_prefix(tour.join("generated")).unwrap();
                let dist = tour.join("dist").join(rel);
                let want = std::fs::read_to_string(&path).unwrap();
                let got = std::fs::read_to_string(&dist).unwrap();
                assert_eq!(
                    got,
                    want,
                    "stale snapshot for {} — refresh examples/tour/generated/",
                    rel.display()
                );
                checked += 1;
            }
        }
    }
    assert!(checked >= 9, "expected at least 9 snapshot files, found {checked}");
}

#[test]
fn macro_only_module_emits_no_file() {
    let tour = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/tour");
    let out = gan().current_dir(&tour).arg("build").output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("macro-only"), "{stdout}");
    assert!(
        !tour.join("dist/app/macros.py").exists(),
        "macro-only modules must not produce a runtime file (GEP-0002-R009)"
    );
    // running a macro-only module is a clear error
    let out = gan()
        .current_dir(&tour)
        .args(["run", "src/app/macros.gan"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("defines only macros"), "{stderr}");
}
