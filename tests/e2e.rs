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
