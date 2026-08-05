"""gan — the Gandora task runner (GEP-0013), written in Gandora.
Native commands drive gandora-core in-process; unknown subcommands
delegate to gan-<name> plugins, then to the stage-0 compiler ganc.
"""

import builtins
import gandora_core as core
import importlib.metadata
import os
import pathlib
import shutil
import subprocess
import sys
import gandora_std.enum
import gandora_std.map
import gandora_std.string
import gandora_tool.advisor
import gandora_tool.agent
import gandora_tool.fmt
import gandora_tool.verifier


def _gan_truthy(value):
    return value is not None and value is not False

def _gan_and(value, then):
    return then() if _gan_truthy(value) else value

class GanMatchError(Exception):
    pass

usage = "gan - the Gandora task runner\n\nUsage:\n  gan build [--strict]    the verdict + compile: diagnostics, advice,\n                          artifact verification; errors stop artifacts\n                          (--strict adds type-flow warnings via ty)\n  gan run <file> [args...] | exec <code> | repl\n  gan fmt [--check] [path...] | init <path> | version\n  gan test                doctests + tests/*.gan (GEP-0024)\n  gan agent [--json]      the AI-session briefing: working loop +\n                          context pack in one output (GEP-0026)\n  gan <plugin> ...        delegates to gan-<plugin>, then to ganc\n"

labels = {"error": ("error", "1;31"), "warning": ("warning", "1;33"), "practice": ("practice", "1;36"), "migration": ("migration", "1;35"), "did_you_mean": ("did you mean", "1;34"), "type": ("type", "2;37")}

gandora_jsonc = "{\n  \"source\": [\"src\"],\n  \"outDir\": \"dist\",\n  \"targetPython\": \"3.11\"\n}\n"

gitignore = "__pycache__/\n*.py[oc]\ndist/\n.gandora/\n.venv/\n"

hello_gan = "defmodule Main do\n  @moduledoc \"Your project's entry point — `gan run src/main.gan`.\"\n\n  @doc \"Prints a greeting.\"\n  @spec main() :: nil\n  def main() do\n    IO.puts(\"Hello from Gandora!\")\n  end\nend\n"


def main() -> None:
    """The task-runner entry: one command per invocation (GEP-0013)."""
    args = gandora_std.enum.drop(builtins.list(sys.argv), 1)
    _gan_case0 = args
    match _gan_case0:
        case [] as _gan_l1 if isinstance(_gan_l1, list):
            print(usage)
            return sys.exit(2)
        case ["version", *_] as _gan_l2 if isinstance(_gan_l2, list):
            return version()
        case ["build", *rest] as _gan_l3 if isinstance(_gan_l3, list):
            return build(gandora_std.enum.member_p(rest, "--strict"))
        case ["check", *rest] as _gan_l4 if isinstance(_gan_l4, list):
            print("note: `gan check` merged into `gan build` (GEP-0025 rev 3)")
            return build(gandora_std.enum.member_p(rest, "--strict"))
        case ["run", file, *rest] as _gan_l5 if isinstance(_gan_l5, list):
            return run(file, rest)
        case ["run"] as _gan_l6 if isinstance(_gan_l6, list):
            return _die_usage("run requires a file")
        case ["exec", code, *_] as _gan_l7 if isinstance(_gan_l7, list):
            return exec_code(code)
        case ["exec"] as _gan_l8 if isinstance(_gan_l8, list):
            return _die_usage("exec requires code")
        case ["repl", *_] as _gan_l9 if isinstance(_gan_l9, list):
            return repl()
        case ["agent", *rest] as _gan_l10 if isinstance(_gan_l10, list):
            return gandora_tool.agent.run(rest)
        case ["fmt", *rest] as _gan_l11 if isinstance(_gan_l11, list):
            return gandora_tool.fmt.run(rest)
        case ["init", path, *_] as _gan_l12 if isinstance(_gan_l12, list):
            return init(path)
        case ["init"] as _gan_l13 if isinstance(_gan_l13, list):
            return _die_usage("init requires a path")
        case [cmd, *rest] as _gan_l14 if isinstance(_gan_l14, list):
            return _delegate(cmd, rest)
        case _:
            raise GanMatchError("no case clause matched: " + repr(_gan_case0))


def _die_usage(msg):
    print("gan: " + msg)
    print(usage)
    return sys.exit(2)


def _root():
    return os.getcwd()


def _project_python():
    venv = _root() + "/.venv/bin/python"
    if _gan_truthy(os.path.exists(venv)):
        return venv
    else:
        return sys.executable


def version() -> None:
    """Runner and compiler-library versions."""
    print(f"gan {_runner_version()} (runner) / gandora-core {core.version()}")
    if _runner_version() != core.version():
        return print("warning: runner and gandora-core versions differ (GEP-0012-R007)")
    else:
        return None


def _runner_version():
    return importlib.metadata.version("gandora-tool")


def build(strict: bool = False) -> None:
    """The verdict and the artifacts in one command (GEP-0025 rev 3):
diagnostics, Advisor suggestions, and artifact verification; any
error stops before artifacts are written.

## Parameters

  - strict: Also surface full type-flow findings as warnings.
"""
    while True:
        if not (_gan_truthy(_run_check(True, strict))):
            print(_paint("build aborted: errors in the verdict", "1;31"))
            sys.exit(1)
        try:
            modules = core.build(_root())
            return print(_paint("✓", "1;32") + f" compiled {gandora_std.enum.count(modules)} module(s)")
        except core.CompileError as e:
            return _compile_error(e)


def _run_check(with_suggestions=True, strict=False):
    while True:
        diags = core.check(_root())
        errors0 = gandora_std.enum.filter(diags, lambda d: gandora_std.map.get(d, "severity") == "error")
        if _gan_truthy(gandora_std.enum.empty_p(errors0)):
            cache = _root() + "/.gandora/cache"
            try:
                modules = core.build(_root(), cache)
                _gan_tmp16 = gandora_tool.verifier.verify(_root(), cache, modules, strict)
            except Exception as _e:
                _gan_tmp16 = []
            verification = _gan_tmp16
            _gan_tmp15 = diags + verification
        else:
            _gan_tmp15 = diags
        diags = _gan_tmp15
        gandora_std.enum.each(diags, lambda d: _print_finding(gandora_std.map.get(d, "severity"), gandora_std.map.get(d, "path"), gandora_std.map.get(d, "line", 0), gandora_std.map.get(d, "message")))
        if _gan_truthy(with_suggestions):
            _gan_tmp17 = _collect_files(["src", "tests"])
        else:
            _gan_tmp17 = []
        sources = _gan_tmp17
        def _gan_fn0(path, *, diags=diags):
            try:
                _gan_tmp18 = pathlib.Path(path).read_text()
            except Exception as _e:
                _gan_tmp18 = ""
            text = _gan_tmp18
            per_file = gandora_std.enum.filter(diags, lambda d, *, path=path: gandora_std.map.get(d, "path") == path)
            return gandora_std.enum.map(gandora_tool.advisor.analyze(text, _root()) + gandora_tool.advisor.lint_hints(text, per_file), lambda h, *, path=path: gandora_std.map.put(h, "path", path))
        hints = gandora_std.enum.flat_map(sources, _gan_fn0)
        consolidated = gandora_tool.advisor.consolidate(hints)
        gandora_std.enum.each(consolidated, lambda h: _print_finding(gandora_std.map.get(h, "kind"), gandora_std.map.get(h, "path"), gandora_std.map.get(h, "line", 0), gandora_std.map.get(h, "message")))
        errors = gandora_std.enum.filter(diags, lambda d: gandora_std.map.get(d, "severity") == "error")
        _print_summary(gandora_std.enum.count(errors), gandora_std.enum.count(diags) - gandora_std.enum.count(errors), gandora_std.enum.count(consolidated), with_suggestions)
        return gandora_std.enum.empty_p(errors)


def _color_p():
    return _gan_and(sys.stdout.isatty(), lambda: (os.environ.get("NO_COLOR") is None))


def _paint(text, code):
    if _gan_truthy(_color_p()):
        esc = builtins.chr(27)
        return esc + ("[" + (code + ("m" + (text + (esc + "[0m")))))
    else:
        return text


def _print_finding(kind, path, line, message):
    _gan_val19 = gandora_std.map.get(labels, kind, (kind, "1"))
    match _gan_val19:
        case (label, code) as _gan_t20 if isinstance(_gan_t20, tuple):
            pass
        case _:
            raise GanMatchError("no match of right-hand side value: " + repr(_gan_val19))
    if line > 0:
        _gan_tmp21 = f":{line}"
    else:
        _gan_tmp21 = ""
    at = _gan_tmp21
    rel = gandora_std.string.replace(str(path), _root() + "/", "")
    print("")
    _gan_fstr22 = _paint(rel + at, "2")
    print(f"{_paint(label, code)} {_gan_fstr22}")
    return gandora_std.enum.each(message.split("\n"), lambda l: print("  " + l))


def _print_summary(errors, warnings, suggestions, with_suggestions):
    if errors > 0:
        print("")
        return print(_paint(f"✗ {errors} error(s)", "1;31") + (_plural(warnings, "warning") + _plural(suggestions, "suggestion")))
    elif _gan_truthy(_gan_and((warnings + suggestions) > 0, lambda: with_suggestions)):
        print("")
        return print(_paint("◦ almost", "1;33") + (_plural(warnings, "warning") + (_plural(suggestions, "suggestion") + " — apply them for a clean verdict")))
    else:
        return None


def _plural(n, word):
    if n > 0:
        return f", {n} {word}(s)"
    else:
        return ""


def _collect_files(roots):
    def _gan_fn1(r):
        p = pathlib.Path(r)
        if not (_gan_truthy(p.is_dir())):
            return []
        elif r == "tests":
            return gandora_std.enum.sort(gandora_std.enum.map(builtins.list(p.glob("*.gan")), str))
        else:
            return gandora_std.enum.sort(gandora_std.enum.map(builtins.list(p.rglob("*.gan")), str))
    return gandora_std.enum.flat_map(roots, _gan_fn1)


def run(file: str, args: list[str]) -> None:
    """Compiles and executes `file` with the project Python (GEP-0013-R002).

## Parameters

  - file: The .gan entry file.
  - args: Arguments passed through to the program.
"""
    if not (_gan_truthy(_run_check(False))):
        print("run aborted: check failed")
        sys.exit(1)
    cache = _root() + "/.gandora/cache"
    try:
        modules = core.build(_root(), cache)
        abs = str(pathlib.Path(file).resolve())
        target = gandora_std.enum.find(modules, lambda m, *, abs=abs: gandora_std.map.get(m, "source") == abs)
        if (target is None):
            print(f"gan: {file} is not a module of this project")
            return sys.exit(1)
        elif (gandora_std.map.get(target, "python") is None):
            _gan_fstr23 = gandora_std.map.get(target, "module")
            print(f"gan: {_gan_fstr23} defines only macros; nothing to run")
            return sys.exit(1)
        else:
            code = subprocess.call([_project_python(), "-P", gandora_std.map.get(target, "python")] + args, env=gandora_std.map.put(builtins.dict(os.environ), "PYTHONPATH", cache))
            return sys.exit(code)
    except core.CompileError as e:
        return _compile_error(e)


def exec_code(code: str) -> None:
    """Compiles and runs one expression given on the command line.

## Parameters

  - code: The Gandora source text.
"""
    return _eval_line(code, builtins.dict())


def repl() -> None:
    """An interactive loop over compile_snippet — state carries across lines."""
    print(f"gan repl (gandora-core {core.version()}) — Ctrl-D to exit")
    ns = builtins.dict()
    _repl_walk(ns)
    return print("")


def _repl_walk(ns):
    while True:
        try:
            _gan_tmp24 = builtins.input("gan> ")
        except builtins.EOFError as _e:
            _gan_tmp24 = "eof"
        except builtins.KeyboardInterrupt as _e:
            _gan_tmp24 = "eof"
        line = _gan_tmp24
        if line == "eof":
            return "ok"
        elif gandora_std.string.trim(line) == "":
            ns = ns
            continue
        else:
            _eval_line(line, ns)
            ns = ns
            continue


def _eval_line(code, ns):
    try:
        compiled = core.compile_snippet(code, _root())
        builtins.exec(compiled, ns)
        result = gandora_std.map.get(ns, "_")
        if not ((result is None)):
            return print(repr(result))
        else:
            return None
    except core.CompileError as e:
        return print(f"error: {gandora_std.enum.at(builtins.list(e.args), 0)}")
    except Exception as e:
        return print(f"{builtins.type(e).__name__}: {str(e)}")


def _pyproject_toml(name):
    return f"[project]\nname = \"{name}\"\nversion = \"0.1.0\"\nrequires-python = \">=3.11\"\ndependencies = [\"gandora-std>={core.version()}\"]\n\n[dependency-groups]\ndev = [\"gandora-tool[dev]>={core.version()}\"]\n"


def init(path: str) -> None:
    """Creates a new project (or fills an existing directory) in place.

## Parameters

  - path: Where to create the project.
"""
    p = pathlib.Path(path)
    if _gan_truthy(_gan_and(p.exists(), lambda: not (_gan_truthy(p.is_dir())))):
        print(f"gan: {path} is a file")
        sys.exit(1)
    elif _gan_truthy((p / "gandora.jsonc").exists()):
        print(f"gan: {path} is already a Gandora project")
        sys.exit(1)
    else:
        pass
    (p / "src").mkdir(parents=True, exist_ok=True)
    name = str(p.resolve().name)
    (p / "gandora.jsonc").write_text(gandora_jsonc)
    pyproject = p / "pyproject.toml"
    if _gan_truthy(pyproject.exists()):
        print("kept existing pyproject.toml — add \"gandora-std\" to its dependencies")
    else:
        pyproject.write_text(_pyproject_toml(name))
    _write_if_absent(p / ".gitignore", gitignore)
    _write_if_absent(p / ".python-version", "3.11\n")
    _write_if_absent((p / "src") / "main.gan", hello_gan)
    return print(f"Initialized Gandora project in {path}")


def _write_if_absent(file, content):
    if not (_gan_truthy(file.exists())):
        return file.write_text(content)
    else:
        return None


def plugin_name(cmd: str) -> str:
    """The executable name a plugin subcommand delegates to (GEP-0013-R003).

## Parameters

  - cmd: The subcommand.

    >>> plugin_name("fmt")
    'gan-fmt'
"""
    return "gan-" + cmd


def _find_plugin(cmd):
    venv_bin = _root() + "/.venv/bin"
    local = shutil.which(plugin_name(cmd), path=venv_bin)
    if (local is None):
        return shutil.which(plugin_name(cmd))
    else:
        return local


def _delegate(cmd, rest):
    plugin = _find_plugin(cmd)
    ganc = _ganc_bin()
    if not ((plugin is None)):
        return sys.exit(subprocess.call([plugin] + rest))
    elif not ((ganc is None)):
        return sys.exit(subprocess.call([ganc, cmd] + rest))
    else:
        return _die_usage(f"unknown command '{cmd}' (no gan-{cmd} plugin, no ganc)")


def _ganc_bin():
    local = os.path.join(os.path.dirname(sys.executable), "ganc")
    if _gan_truthy(os.path.exists(local)):
        return local
    else:
        return shutil.which("ganc")


def _compile_error(e):
    args = builtins.list(e.args)
    print(f"{gandora_std.enum.at(args, 1)}:{gandora_std.enum.at(args, 2)}:{gandora_std.enum.at(args, 3)}: error: {gandora_std.enum.at(args, 0)}")
    return sys.exit(1)


if __name__ == "__main__":
    main()
