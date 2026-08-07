"""gan — the Gandora task runner (GEP-0013), written in Gandora.
Native commands drive gandora-core in-process; unknown subcommands
delegate to gan-<name> plugins, then to the stage-0 compiler ganc.
"""

import builtins
import gandora_core as core
import importlib.metadata
import os
import pathlib
import subprocess
import sys
import gandora_std.enum
import gandora_std.file
import gandora_std.map
import gandora_std.path
import gandora_std.string
import gandora_std.system
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

usage = "gan - the Gandora task runner\n\nUsage:\n  gan build [--strict]    the verdict + compile: diagnostics, advice,\n                          artifact verification; errors stop artifacts\n                          (--strict adds type-flow warnings via ty)\n  gan run <file> [args...] | exec <code> | repl\n  gan fmt [--check] [path...] | version\n  gan init [--package] <path>   scaffold a project (or fill an existing one)\n  gan test                doctests + tests/*.gan (GEP-0024)\n  gan agent [--json]      the AI-session briefing: working loop +\n                          context pack in one output (GEP-0026)\n  gan <plugin> ...        delegates to gan-<plugin>, then to ganc\n"

labels = {"error": ("error", "1;31"), "warning": ("warning", "1;33"), "practice": ("practice", "1;36"), "migration": ("migration", "1;35"), "did_you_mean": ("did you mean", "1;34"), "type": ("type", "2;37")}

gandora_jsonc = "{\n  \"source\": [\"src\"],\n  \"outDir\": \"dist\",\n  \"targetPython\": \"3.11\"\n}\n"

gitignore = "__pycache__/\n*.py[oc]\ndist/\n.gandora/\n.venv/\n"

hello_gan = "defmodule Main do\n  @moduledoc \"Your project's entry point — `gan run src/main.gan`.\"\n\n  @doc \"Prints a greeting.\"\n  @spec main() :: nil\n  def main() do\n    IO.puts(\"Hello from Gandora!\")\n  end\nend\n"

mcp_claude = "{\n  \"mcpServers\": {\n    \"gandora\": {\n      \"type\": \"stdio\",\n      \"command\": \"uv\",\n      \"args\": [\"run\", \"gan\", \"mcp\"]\n    }\n  }\n}\n"

mcp_codex = "# Codex reads this for a trusted project; `codex mcp list` shows it.\n[mcp_servers.gandora]\ncommand = \"uv\"\nargs = [\"run\", \"gan\", \"mcp\"]\n"

mcp_opencode = "{\n  \"$schema\": \"https://opencode.ai/config.json\",\n  \"mcp\": {\n    \"gandora\": {\n      \"type\": \"local\",\n      \"command\": [\"uv\", \"run\", \"gan\", \"mcp\"],\n      \"enabled\": true\n    }\n  }\n}\n"

package_jsonc = "{\n  // Gandora package project (GEP-0006): `gan build` also emits the\n  // gandora.toml marker and ships .gan sources for macro consumers.\n  \"source\": [\"src\"],\n  \"outDir\": \"pkg\",\n  \"targetPython\": \"3.11\",\n  \"package\": true\n}\n"


def main() -> None:
    """The task-runner entry: one command per invocation (GEP-0013)."""
    args = gandora_std.system.argv()
    _gan_case0 = args
    match _gan_case0:
        case [] as _gan_l1 if isinstance(_gan_l1, list):
            print(usage)
            return gandora_std.system.halt(2)
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
        case ["init", *rest] as _gan_l12 if isinstance(_gan_l12, list):
            return _init_cmd(rest)
        case [cmd, *rest] as _gan_l13 if isinstance(_gan_l13, list):
            return _delegate(cmd, rest)
        case _:
            raise GanMatchError("no case clause matched: " + repr(_gan_case0))


def _die_usage(msg):
    print("gan: " + msg)
    print(usage)
    return gandora_std.system.halt(2)


def _root():
    return gandora_std.file.cwd_bang()


def _project_python():
    venv = gandora_std.path.join(_root(), ".venv/bin/python")
    if _gan_truthy(gandora_std.file.exists_p(venv)):
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
            gandora_std.system.halt(1)
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
            cache = gandora_std.path.join(_root(), ".gandora/cache")
            try:
                modules = core.build(_root(), cache)
                _gan_tmp15 = gandora_tool.verifier.verify(_root(), cache, modules, strict)
            except Exception as _e:
                _gan_tmp15 = []
            verification = _gan_tmp15
            _gan_tmp14 = diags + verification
        else:
            _gan_tmp14 = diags
        diags = _gan_tmp14
        gandora_std.enum.each(diags, lambda d: _print_finding(gandora_std.map.get(d, "severity"), gandora_std.map.get(d, "path"), gandora_std.map.get(d, "line", 0), gandora_std.map.get(d, "message")))
        if _gan_truthy(with_suggestions):
            _gan_tmp16 = _collect_files(["src", "tests"])
        else:
            _gan_tmp16 = []
        sources = _gan_tmp16
        def _gan_fn0(path, *, diags=diags):
            _gan_case18 = gandora_std.file.read(path)
            match _gan_case18:
                case ("ok", t) as _gan_t19 if isinstance(_gan_t19, tuple):
                    _gan_tmp17 = t
                case ("error", _why) as _gan_t20 if isinstance(_gan_t20, tuple):
                    _gan_tmp17 = ""
                case _:
                    raise GanMatchError("no case clause matched: " + repr(_gan_case18))
            text = _gan_tmp17
            per_file = gandora_std.enum.filter(diags, lambda d, *, path=path: gandora_std.map.get(d, "path") == path)
            return gandora_std.enum.map(gandora_tool.advisor.analyze(text, _root()) + gandora_tool.advisor.lint_hints(text, per_file), lambda h, *, path=path: gandora_std.map.put(h, "path", path))
        hints = gandora_std.enum.flat_map(sources, _gan_fn0)
        consolidated = gandora_tool.advisor.consolidate(hints)
        gandora_std.enum.each(consolidated, lambda h: _print_finding(gandora_std.map.get(h, "kind"), gandora_std.map.get(h, "path"), gandora_std.map.get(h, "line", 0), gandora_std.map.get(h, "message")))
        errors = gandora_std.enum.filter(diags, lambda d: gandora_std.map.get(d, "severity") == "error")
        _print_summary(gandora_std.enum.count(errors), gandora_std.enum.count(diags) - gandora_std.enum.count(errors), gandora_std.enum.count(consolidated), with_suggestions)
        return gandora_std.enum.empty_p(errors)


def _color_p():
    return _gan_and(sys.stdout.isatty(), lambda: (gandora_std.system.get_env("NO_COLOR") is None))


def _paint(text, code):
    if _gan_truthy(_color_p()):
        esc = builtins.chr(27)
        return esc + ("[" + (code + ("m" + (text + (esc + "[0m")))))
    else:
        return text


def _print_finding(kind, path, line, message):
    _gan_val21 = gandora_std.map.get(labels, kind, (kind, "1"))
    match _gan_val21:
        case (label, code) as _gan_t22 if isinstance(_gan_t22, tuple):
            pass
        case _:
            raise GanMatchError("no match of right-hand side value: " + repr(_gan_val21))
    if line > 0:
        _gan_tmp23 = f":{line}"
    else:
        _gan_tmp23 = ""
    at = _gan_tmp23
    rel = gandora_std.string.replace(str(path), _root() + "/", "")
    print("")
    _gan_fstr24 = _paint(rel + at, "2")
    print(f"{_paint(label, code)} {_gan_fstr24}")
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
        if not (_gan_truthy(gandora_std.file.dir_p(r))):
            return []
        elif r == "tests":
            return gandora_std.path.wildcard(gandora_std.path.join(r, "*.gan"))
        else:
            return gandora_std.path.wildcard(gandora_std.path.join(gandora_std.path.join(r, "**"), "*.gan"))
    return gandora_std.enum.flat_map(roots, _gan_fn1)


def run(file: str, args: list[str]) -> None:
    """Compiles and executes `file` with the project Python (GEP-0013-R002).

## Parameters

  - file: The .gan entry file.
  - args: Arguments passed through to the program.
"""
    if not (_gan_truthy(_run_check(False))):
        print("run aborted: check failed")
        gandora_std.system.halt(1)
    cache = gandora_std.path.join(_root(), ".gandora/cache")
    try:
        modules = core.build(_root(), cache)
        abs = str(pathlib.Path(file).resolve())
        target = gandora_std.enum.find(modules, lambda m, *, abs=abs: gandora_std.map.get(m, "source") == abs)
        if (target is None):
            print(f"gan: {file} is not a module of this project")
            return gandora_std.system.halt(1)
        elif (gandora_std.map.get(target, "python") is None):
            _gan_fstr26 = gandora_std.map.get(target, "module")
            print(f"gan: {_gan_fstr26} defines only macros; nothing to run")
            return gandora_std.system.halt(1)
        else:
            code = subprocess.call([_project_python(), "-P", gandora_std.map.get(target, "python")] + args, env=gandora_std.map.put(builtins.dict(os.environ), "PYTHONPATH", cache))
            return gandora_std.system.halt(code)
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
            _gan_tmp27 = builtins.input("gan> ")
        except builtins.EOFError as _e:
            _gan_tmp27 = "eof"
        except builtins.KeyboardInterrupt as _e:
            _gan_tmp27 = "eof"
        line = _gan_tmp27
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
    return f"[project]\nname = \"{name}\"\nversion = \"0.1.0\"\nrequires-python = \">=3.11\"\ndependencies = [\"gandora-std>={core.version()}\"]\n\n[dependency-groups]\ndev = [\"gandora-tool[dev]>={core.version()}\", \"gandora-mcp>={core.version()}\"]\n"


def write_mcp_configs(p: str) -> None:
    """Writes the project-level MCP configuration each agent reads on its own
(GEP-0028-R012): `.mcp.json` for Claude Code, `.codex/config.toml` for
Codex, `opencode.json` for opencode. Never overwrites what is there.

## Parameters

  - p: The project directory.
"""
    _write_if_absent(gandora_std.path.join(p, ".mcp.json"), mcp_claude)
    _write_if_absent(gandora_std.path.join(p, "opencode.json"), mcp_opencode)
    gandora_std.file.mkdir_p_bang(gandora_std.path.join(p, ".codex"))
    return _write_if_absent(gandora_std.path.join(gandora_std.path.join(p, ".codex"), "config.toml"), mcp_codex)


def _init_cmd(args):
    paths = gandora_std.enum.filter(args, lambda a: not (_gan_truthy(gandora_std.string.starts_with_p(a, "--"))))
    if _gan_truthy(gandora_std.enum.empty_p(paths)):
        _gan_tmp28 = "."
    else:
        _gan_tmp28 = gandora_std.enum.at(paths, 0)
    path = _gan_tmp28
    if _gan_truthy(gandora_std.enum.member_p(args, "--package")):
        if _gan_truthy(gandora_std.enum.empty_p(paths)):
            return _die_usage("init --package requires a package name")
        else:
            return init_package(path)
    else:
        return init(path)


def init(path: str) -> None:
    """Creates a project, or fills in what an existing one is missing.

Since the scaffold also writes the agent wiring (GEP-0028-R012),
re-running it on a project that already exists is how that wiring
arrives — so this never refuses and never overwrites: every file it
would write, it writes only when absent.

## Parameters

  - path: Where to create the project.
"""
    if _gan_truthy(_gan_and(gandora_std.file.exists_p(path), lambda: not (_gan_truthy(gandora_std.file.dir_p(path))))):
        print(f"gan: {path} is a file")
        gandora_std.system.halt(1)
    gandora_std.file.mkdir_p_bang(gandora_std.path.join(path, "src"))
    name = gandora_std.path.basename(gandora_std.path.expand(path))
    _write_if_absent(gandora_std.path.join(path, "gandora.jsonc"), gandora_jsonc)
    pyproject = gandora_std.path.join(path, "pyproject.toml")
    if _gan_truthy(gandora_std.file.exists_p(pyproject)):
        if not (_gan_truthy(gandora_std.string.contains_p(gandora_std.file.read_bang(pyproject), "gandora-mcp"))):
            print("kept existing pyproject.toml — add these so the toolchain and the MCP wiring resolve:")
            print("  dependencies: \"gandora-std\"        (generated code imports it)")
            print("  dev group:    \"gandora-tool[dev]\", \"gandora-mcp\"  (`uv run gan mcp`)")
    else:
        gandora_std.file.write_bang(pyproject, _pyproject_toml(name))
    _write_if_absent(gandora_std.path.join(path, ".gitignore"), gitignore)
    _write_if_absent(gandora_std.path.join(path, ".python-version"), "3.11\n")
    _write_if_absent(gandora_std.path.join(gandora_std.path.join(path, "src"), "main.gan"), hello_gan)
    write_mcp_configs(path)
    print(f"Initialized Gandora project in {path}")
    return print("MCP wired for Claude Code (.mcp.json), Codex (.codex/config.toml), opencode (opencode.json)")


def _write_if_absent(file, content):
    if not (_gan_truthy(gandora_std.file.exists_p(file))):
        return gandora_std.file.write_bang(file, content)
    else:
        return None


def _package_pyproject(dist_name, py_pkg):
    return f"[project]\nname = \"{dist_name}\"\nversion = \"0.1.0\"\ndescription = \"A Gandora package\"\nrequires-python = \">=3.11\"\ndependencies = [\"gandora-std>={core.version()}\"]\n\n[build-system]\nrequires = [\"hatchling\"]\nbuild-backend = \"hatchling.build\"\n\n# pkg/ is gitignored build output but must enter the distribution\n[tool.hatch.build]\nignore-vcs = true\n\n# the sdist carries sources and the compiled output of `gan build`\n[tool.hatch.build.targets.sdist]\ninclude = [\"pkg\", \"src\", \"gandora.jsonc\"]\n\n# the wheel packages the compiled output (marker and .gan sources included)\n[tool.hatch.build.targets.wheel]\npackages = [\"pkg/{py_pkg}\"]\n\n# the toolchain and the MCP surface are development-time, never runtime\n[dependency-groups]\ndev = [\"gandora-tool[dev]>={core.version()}\", \"gandora-mcp>={core.version()}\"]\n"


def _package_starter(module, dist_name):
    fence = "\"\"\""
    hole = "#" + "{name}"
    return f"defmodule {module}.Core do\n  @moduledoc \"Public surface of {dist_name}: functions run anywhere, macros expand in consumers.\"\n\n  @doc \"Greets `name` in this package's voice.\"\n  @param name, \"Who to greet.\"\n  @spec hello(string()) :: string()\n  @example {fence}\n      gan> {module}.Core.hello(\"world\")\n      'Hello from {dist_name}, world!'\n  {fence}\n  def hello(name), do: \"Hello from {dist_name}, {hole}!\"\n\n  @doc \"Evaluates `expr` twice and pairs the results — a macro consumers expand.\"\n  @param expr, \"The expression to duplicate.\"\n  @spec twice(term()) :: tuple()\n  defmacro twice(expr) do\n    quote do\n      {{unquote(expr), unquote(expr)}}\n    end\n  end\n\n  @doc \"Proof the macro expands where it is used — a macro's own example is displayed, never run.\"\n  @spec twice_demo() :: tuple()\n  @example {fence}\n      gan> {module}.Core.twice_demo()\n      (2, 2)\n  {fence}\n  def twice_demo(), do: twice(1 + 1)\nend\n"


def init_package(path: str) -> None:
    """Creates a publishable Gandora package (GEP-0006-R001): sources under
`src/<py_pkg>/`, compiled output in `pkg/` shipped in the wheel.

## Parameters

  - path: The package directory to create; its name is the distribution name.
"""
    if _gan_truthy(gandora_std.file.exists_p(path)):
        print(f"gan: {path} already exists")
        gandora_std.system.halt(1)
    dist_name = gandora_std.path.basename(gandora_std.path.expand(path))
    py_pkg = gandora_std.string.replace(dist_name, "-", "_")
    module = camelize(py_pkg)
    src = gandora_std.path.join(gandora_std.path.join(path, "src"), py_pkg)
    gandora_std.file.mkdir_p_bang(src)
    gandora_std.file.write_bang(gandora_std.path.join(path, "gandora.jsonc"), package_jsonc)
    gandora_std.file.write_bang(gandora_std.path.join(path, "pyproject.toml"), _package_pyproject(dist_name, py_pkg))
    gandora_std.file.write_bang(gandora_std.path.join(path, ".gitignore"), gitignore + "pkg/\n")
    gandora_std.file.write_bang(gandora_std.path.join(path, ".python-version"), "3.11\n")
    gandora_std.file.write_bang(gandora_std.path.join(src, "core.gan"), _package_starter(module, dist_name))
    write_mcp_configs(path)
    print(f"Initialized Gandora package {dist_name} in {path}")
    print("Publish with:")
    print(f"  cd {path}")
    return print("  gan build && uv build && uv publish")


def camelize(name: str) -> str:
    """A snake_case package name as its Gandora module name.

## Parameters

  - name: The snake_case name.

    >>> camelize("gan_coin")
    'GanCoin'
"""
    _gan_tmp29 = [gandora_std.string.capitalize(part) for part in gandora_std.string.split_on(name, "_") if part != ""]
    parts = _gan_tmp29
    return gandora_std.enum.join(parts, "")


def plugin_name(cmd: str) -> str:
    """The executable name a plugin subcommand delegates to (GEP-0013-R003).

## Parameters

  - cmd: The subcommand.

    >>> plugin_name("fmt")
    'gan-fmt'
"""
    return "gan-" + cmd


def _find_plugin(cmd):
    local = gandora_std.path.join(gandora_std.path.join(_root(), ".venv/bin"), plugin_name(cmd))
    if _gan_truthy(gandora_std.file.exists_p(local)):
        return local
    else:
        return gandora_std.system.find_executable(plugin_name(cmd))


def _delegate(cmd, rest):
    plugin = _find_plugin(cmd)
    ganc = _ganc_bin()
    if not ((plugin is None)):
        return gandora_std.system.halt(subprocess.call([plugin] + rest))
    elif not ((ganc is None)):
        return gandora_std.system.halt(subprocess.call([ganc, cmd] + rest))
    else:
        return _die_usage(f"unknown command '{cmd}' (no gan-{cmd} plugin, no ganc)")


def _ganc_bin():
    local = gandora_std.path.join(gandora_std.path.dirname(sys.executable), "ganc")
    if _gan_truthy(gandora_std.file.exists_p(local)):
        return local
    else:
        return gandora_std.system.find_executable("ganc")


def _compile_error(e):
    args = builtins.list(e.args)
    print(f"{gandora_std.enum.at(args, 1)}:{gandora_std.enum.at(args, 2)}:{gandora_std.enum.at(args, 3)}: error: {gandora_std.enum.at(args, 0)}")
    return gandora_std.system.halt(1)


if __name__ == "__main__":
    main()
