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

#[test]
fn pandas_and_numpy_chapters_run() {
    // these chapters need the tour's uv dev dependencies; skip cleanly when
    // the .venv is absent so stdlib-only environments still pass
    let tour = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/tour");
    let venv_py = tour.join(".venv/bin/python");
    if !venv_py.exists() {
        eprintln!("skipping: examples/tour/.venv not present (run `uv sync`)");
        return;
    }
    let deps = Command::new(&venv_py)
        .args(["-c", "import pandas, numpy"])
        .status()
        .unwrap();
    if !deps.success() {
        eprintln!("skipping: pandas/numpy not installed (run `uv sync`)");
        return;
    }
    let out = gan().current_dir(&tour).arg("build").output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));

    let run = |script: &str| -> String {
        let out = Command::new(&venv_py)
            .arg("-P")
            .arg(tour.join(script))
            .env("PYTHONPATH", tour.join("dist"))
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{script} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).to_string()
    };

    let stdout = run("dist/tour/dataframe.py");
    assert!(stdout.contains("total revenue = 6810.5"), "{stdout}");
    assert!(stdout.contains("east units    = 50"), "{stdout}");

    let stdout = run("dist/tour/numpy.py");
    assert!(stdout.contains("a * 10 + 1   = [11. 21. 31. 41.]"), "{stdout}");
    assert!(stdout.contains("norm([3,4])  = 5.0"), "{stdout}");
    assert!(stdout.contains("std          = 3.452"), "{stdout}");
}

#[test]
fn package_publication_round_trip() {
    // publisher side: scaffold, build, verify marker + shipped sources
    let dir = temp_dir("pkg");
    let pkg = dir.join("acme-demo");
    let out = gan().args(["init", "--package"]).arg(&pkg).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let out = gan().current_dir(&pkg).arg("build").output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let marker = std::fs::read_to_string(pkg.join("pkg/acme_demo/gandora.toml")).unwrap();
    assert!(marker.contains("schema = 1"), "{marker}");
    assert!(marker.contains("name = \"AcmeDemo.Core\""), "{marker}");
    assert!(pkg.join("pkg/acme_demo/_gan/acme_demo/core.gan").exists());

    // consumer side: simulate an installed wheel by copying the built
    // package into a fake site-packages, then use it from Gandora
    let consumer = dir.join("app");
    let out = gan().arg("init").arg(&consumer).output().unwrap();
    assert!(out.status.success());
    let site = consumer.join(".venv/lib/python3.12/site-packages");
    copy_tree(&pkg.join("pkg/acme_demo"), &site.join("acme_demo"));
    std::fs::write(
        consumer.join("src/main.gan"),
        "defmodule Main do\n  require AcmeDemo.Core\n  alias AcmeDemo.Core\n\n  def main() do\n    IO.puts(Core.hello(\"e2e\"))\n    IO.puts(inspect(AcmeDemo.Core.twice(1 + 2)))\n  end\nend\n",
    )
    .unwrap();
    let out = gan().current_dir(&consumer).arg("build").output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let main_py = std::fs::read_to_string(consumer.join("dist/main.py")).unwrap();
    // the macro expanded at compile time; the function is a plain import
    assert!(main_py.contains("import acme_demo.core"), "{main_py}");
    assert!(main_py.contains("((1 + 2), (1 + 2))") || main_py.contains("(1 + 2, 1 + 2)"), "{main_py}");
    assert!(!main_py.contains("gandora"), "no runtime import allowed: {main_py}");
    // and the program runs against the "installed" package
    let stdout = {
        let out = Command::new("python3")
            .arg("-P")
            .arg(consumer.join("dist/main.py"))
            .env(
                "PYTHONPATH",
                format!("{}:{}", consumer.join("dist").display(), site.display()),
            )
            .output()
            .unwrap();
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
        String::from_utf8_lossy(&out.stdout).to_string()
    };
    assert!(stdout.contains("Hello from acme-demo, e2e!"), "{stdout}");
    assert!(stdout.contains("(3, 3)"), "{stdout}");

    // an unresolvable require has a named diagnostic (GEP-0006-R008)
    std::fs::write(
        consumer.join("src/missing.gan"),
        "defmodule Missing do\n  require NoSuch.Package\n  def main(), do: nil\nend\n",
    )
    .unwrap();
    let out = gan().current_dir(&consumer).arg("build").output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("NoSuch.Package"), "{stderr}");
    assert!(stderr.contains("GEP-0006-R008"), "{stderr}");
    let _ = std::fs::remove_dir_all(&dir);
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let dest = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &dest);
        } else {
            std::fs::copy(entry.path(), &dest).unwrap();
        }
    }
}

#[test]
fn fastapi_chapter_runs() {
    let tour = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/tour");
    let venv_py = tour.join(".venv/bin/python");
    if !venv_py.exists() {
        eprintln!("skipping: examples/tour/.venv not present (run `uv sync`)");
        return;
    }
    let deps = Command::new(&venv_py)
        .args(["-c", "import fastapi, httpx"])
        .status()
        .unwrap();
    if !deps.success() {
        eprintln!("skipping: fastapi not installed (run `uv sync`)");
        return;
    }
    let out = gan().current_dir(&tour).arg("build").output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let out = Command::new(&venv_py)
        .arg("-P")
        .arg(tour.join("dist/tour/webapi.py"))
        .env("PYTHONPATH", tour.join("dist"))
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("-> 200 {'message': 'hello from gandora'"), "{stdout}");
    assert!(stdout.contains("'slug': 'hello-gandora-world'"), "{stdout}");
    assert!(stdout.contains("'fact': 3628800"), "{stdout}");
    assert!(stdout.contains("GET /nope             -> 404"), "{stdout}");
}

#[test]
fn doctests_and_localized_docs() {
    let tour = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/tour");
    // gan doc: default, exact locale, and prefix fallback
    let out = gan().current_dir(&tour).args(["doc", "App.Mathy.fact"]).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Factorial via multi-clause dispatch"), "{stdout}");
    let out = gan()
        .current_dir(&tour)
        .args(["doc", "App.Mathy.fact", "--locale", "zh-CN"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("多子句分派实现的阶乘"), "{stdout}");
    let out = gan()
        .current_dir(&tour)
        .args(["doc", "App.Mathy.classify", "--locale", "zh"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("以原子返回数字的符号"), "{stdout}");

    // a failing doctest is detected by gan test
    let dir = temp_dir("doctest");
    let proj = dir.join("app");
    let out = gan().arg("init").arg(&proj).output().unwrap();
    assert!(out.status.success());
    std::fs::write(
        proj.join("src/main.gan"),
        "defmodule Main do\n  @example \"\"\"\n      gan> broken(1)\n      999\n  \"\"\"\n  def broken(x), do: x + 1\n\n  def main(), do: nil\nend\n",
    )
    .unwrap();
    let out = gan().current_dir(&proj).arg("test").output().unwrap();
    assert_eq!(out.status.code(), Some(1), "{}", String::from_utf8_lossy(&out.stdout));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn builtin_docs_are_embedded_and_bilingual() {
    let out = gan().args(["doc", "IO.puts"]).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("print(value)"), "{stdout}");
    assert!(stdout.contains("gan> IO.puts"), "{stdout}");
    let out = gan().args(["doc", "rem", "--locale", "zh"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("截断除法"), "{stdout}");
    assert!(stdout.contains("rem(-7, 2)"), "{stdout}");
}

#[test]
fn marker_runtime_resolution_and_py_package() {
    // a fake installed package whose marker claims module name `Enum`
    // under the gandora_std prefix (GEP-0006-R005A, GEP-0010-R002/R003)
    let dir = temp_dir("stdres");
    let proj = dir.join("app");
    let out = gan().arg("init").arg(&proj).output().unwrap();
    assert!(out.status.success());
    let site = proj.join(".venv/lib/python3.12/site-packages/gandora_std");
    std::fs::create_dir_all(&site).unwrap();
    std::fs::write(
        site.join("enum.py"),
        "def map(xs, f):\n    return [f(x) for x in xs]\n",
    )
    .unwrap();
    std::fs::write(
        site.join("gandora.toml"),
        "schema = 1\ncompiler = \"0.1.0\"\n\n[[modules]]\nname = \"Enum\"\npython = \"gandora_std/enum.py\"\nsource = \"gandora_std/_gan/gandora_std/enum.gan\"\n",
    )
    .unwrap();
    std::fs::write(
        proj.join("src/main.gan"),
        "defmodule Main do\n  def main(), do: IO.puts(inspect(Enum.map([1, 2], fn x -> x * 2 end)))\nend\n",
    )
    .unwrap();
    let out = gan().current_dir(&proj).arg("build").output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let py = std::fs::read_to_string(proj.join("dist/main.py")).unwrap();
    assert!(py.contains("import gandora_std.enum"), "{py}");
    assert!(py.contains("gandora_std.enum.map("), "{py}");
    let _ = std::fs::remove_dir_all(&dir);
}
