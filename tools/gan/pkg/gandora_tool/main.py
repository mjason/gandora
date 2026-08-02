"""gan — the Gandora task runner (GEP-0013), written in Gandora.
Native commands drive gandora-core in-process; unknown subcommands
delegate to gan-<name> plugins, then to the stage-0 compiler ganc.
"""

import builtins
import gandora_core as core
import importlib.metadata
import os
import os.path
import pathlib
import shutil
import subprocess
import sys
import gandora_std.enum
import gandora_std.map
import gandora_std.string


def _gan_truthy(value):
    return value is not None and value is not False

class GanMatchError(Exception):
    pass

usage = "gan - the Gandora task runner\n\nUsage:\n  gan build | check | run <file> [args...] | exec <code> | repl\n  gan init <path> | version\n  gan <plugin> ...        delegates to gan-<plugin>, then to ganc\n"

gandora_jsonc = "{\n  \"source\": [\"src\"],\n  \"outDir\": \"dist\",\n  \"targetPython\": \"3.11\"\n}\n"

gitignore = "__pycache__/\n*.py[oc]\ndist/\n.gandora/\n.venv/\n"

hello_gan = "defmodule Main do\n  def main(), do: IO.puts(\"Hello from Gandora!\")\nend\n"


def main():
    args = gandora_std.enum.drop(builtins.list(sys.argv), 1)
    _gan_case0 = args
    match _gan_case0:
        case [] as _gan_l1 if isinstance(_gan_l1, list):
            print(usage)
            return sys.exit(2)
        case ["version", *_] as _gan_l2 if isinstance(_gan_l2, list):
            return version()
        case ["build", *_] as _gan_l3 if isinstance(_gan_l3, list):
            return build()
        case ["check", *_] as _gan_l4 if isinstance(_gan_l4, list):
            return check()
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
        case ["init", path, *_] as _gan_l10 if isinstance(_gan_l10, list):
            return init(path)
        case ["init"] as _gan_l11 if isinstance(_gan_l11, list):
            return _die_usage("init requires a path")
        case [cmd, *rest] as _gan_l12 if isinstance(_gan_l12, list):
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


def version():
    print(f"gan {_runner_version()} (runner) / gandora-core {core.version()}")
    if _runner_version() != core.version():
        return print("warning: runner and gandora-core versions differ (GEP-0012-R007)")
    else:
        return None


def _runner_version():
    return importlib.metadata.version("gandora-tool")


def build():
    try:
        modules = core.build(_root())
        return print(f"compiled {gandora_std.enum.count(modules)} module(s)")
    except core.CompileError as e:
        return _compile_error(e)


def check():
    diags = core.check(_root())
    def _gan_fn0(d):
        _gan_fstr17 = gandora_std.map.get(d, "severity")
        _gan_fstr18 = gandora_std.map.get(d, "path")
        _gan_fstr19 = gandora_std.map.get(d, "line")
        _gan_fstr20 = gandora_std.map.get(d, "message")
        return print(f"{_gan_fstr17}: {_gan_fstr18}:{_gan_fstr19}: {_gan_fstr20}")
    gandora_std.enum.each(diags, _gan_fn0)
    errors = gandora_std.enum.filter(diags, lambda d: gandora_std.map.get(d, "severity") == "error")
    if _gan_truthy(gandora_std.enum.empty_p(errors)):
        return print("check passed")
    else:
        return sys.exit(1)


def run(file, args):
    cache = _root() + "/.gandora/cache"
    try:
        modules = core.build(_root(), cache)
        abs = str(pathlib.Path(file).resolve())
        target = gandora_std.enum.find(modules, lambda m: gandora_std.map.get(m, "source") == abs)
        if _gan_truthy((target is None)):
            print(f"gan: {file} is not a module of this project")
            return sys.exit(1)
        elif _gan_truthy((gandora_std.map.get(target, "python") is None)):
            _gan_fstr21 = gandora_std.map.get(target, "module")
            print(f"gan: {_gan_fstr21} defines only macros; nothing to run")
            return sys.exit(1)
        else:
            code = subprocess.call([_project_python(), "-P", gandora_std.map.get(target, "python")] + args, env=gandora_std.map.put(builtins.dict(os.environ), "PYTHONPATH", cache))
            return sys.exit(code)
    except core.CompileError as e:
        return _compile_error(e)


def exec_code(code):
    return _eval_line(code, builtins.dict())


def repl():
    print(f"gan repl (gandora-core {core.version()}) — Ctrl-D to exit")
    ns = builtins.dict()
    _gan_loop22 = 0
    _gan_res23 = None
    while True:
        n = _gan_loop22
        try:
            _gan_tmp24 = builtins.input("gan> ")
        except builtins.EOFError as _e:
            _gan_tmp24 = "eof"
        except builtins.KeyboardInterrupt as _e:
            _gan_tmp24 = "eof"
        line = _gan_tmp24
        if line == "eof":
            _gan_res23 = n
            break
        elif gandora_std.string.trim(line) == "":
            _gan_loop22 = n
            continue
        else:
            _eval_line(line, ns)
            _gan_loop22 = n + 1
            continue
        break
    return print("")


def _eval_line(code, ns):
    try:
        compiled = core.compile_snippet(code, _root())
        builtins.exec(compiled, ns)
        result = gandora_std.map.get(ns, "_")
        if not (_gan_truthy((result is None))):
            return print(repr(result))
        else:
            return None
    except core.CompileError as e:
        return print(f"error: {gandora_std.enum.at(builtins.list(e.args), 0)}")
    except Exception as e:
        return print(f"{builtins.type(e).__name__}: {str(e)}")


def _pyproject_toml(name):
    return f"[project]\nname = \"{name}\"\nversion = \"0.1.0\"\nrequires-python = \">=3.11\"\ndependencies = [\"gandora-std>={core.version()}\"]\n"


def init(path):
    p = pathlib.Path(path)
    if _gan_truthy(p.exists()):
        print(f"gan: {path} already exists")
        sys.exit(1)
    (p / "src").mkdir(parents=True)
    name = str(p.resolve().name)
    (p / "gandora.jsonc").write_text(gandora_jsonc)
    (p / "pyproject.toml").write_text(_pyproject_toml(name))
    (p / ".gitignore").write_text(gitignore)
    (p / ".python-version").write_text("3.11\n")
    ((p / "src") / "main.gan").write_text(hello_gan)
    return print(f"Initialized Gandora project in {path}")


def _find_plugin(cmd):
    venv_bin = _root() + "/.venv/bin"
    local = shutil.which("gan-" + cmd, path=venv_bin)
    if _gan_truthy((local is None)):
        return shutil.which("gan-" + cmd)
    else:
        return local


def _delegate(cmd, rest):
    plugin = _find_plugin(cmd)
    ganc = shutil.which("ganc")
    if not (_gan_truthy((plugin is None))):
        return sys.exit(subprocess.call([plugin] + rest))
    elif not (_gan_truthy((ganc is None))):
        return sys.exit(subprocess.call([ganc, cmd] + rest))
    else:
        return _die_usage(f"unknown command '{cmd}' (no gan-{cmd} plugin, no ganc)")


def _compile_error(e):
    args = builtins.list(e.args)
    print(f"{gandora_std.enum.at(args, 1)}:{gandora_std.enum.at(args, 2)}:{gandora_std.enum.at(args, 3)}: error: {gandora_std.enum.at(args, 0)}")
    return sys.exit(1)


if __name__ == "__main__":
    main()
