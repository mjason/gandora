"""The verified-example engine (GEP-0028): a snippet becomes a throwaway
project, faces the same verdict a user's `gan build` would give it,
and its `@example` doctests are actually executed. Nothing returned
from here has gone unrun — that is the whole point of the surface.
"""

import gandora_core as core
import json
import os
import re
import sys
import tempfile
import gandora_std.enum
import gandora_std.file
import gandora_std.list
import gandora_std.map
import gandora_std.path
import gandora_std.string
import gandora_std.system


def _gan_truthy(value):
    return value is not None and value is not False

class GanMatchError(Exception):
    pass

config = "{\"source\": [\"src\"], \"outDir\": \"dist\", \"targetPython\": \"3.11\"}\n"


def verdict(source: str, root: str) -> dict:
    """The full verdict for one snippet: errors, practice suggestions, and —
when the code compiles — the result of really running its doctests.

## Parameters

  - source: The Gandora source text of one module.
  - root: The project the snippet is judged for; its toolchain is used.
"""
    dir = tempfile.mkdtemp(prefix="gan-mcp-")
    env = venv(root)
    try:
        _write_project(dir, source, env)
        base = _check(dir)
        if _gan_truthy(gandora_std.map.get(base, "ok", False)):
            return gandora_std.map.put(base, "doctests", _doctests(dir, source, env))
        else:
            return gandora_std.map.put(base, "doctests", skipped("the code did not compile"))
    finally:
        gandora_std.file.rm_rf_bang(dir)


def venv(root: str) -> str:
    """The environment a snippet is judged in: the project's own `.venv`
when it has one, otherwise the server's. A verdict must be about the
toolchain the project actually runs, not the one the server happens
to be installed in.

## Parameters

  - root: The project root.
"""
    local = gandora_std.path.join(root, ".venv")
    if _gan_truthy(gandora_std.file.dir_p(local)):
        return local
    else:
        return sys.prefix


def skipped(why: str) -> dict:
    """The doctest report for source that carries no `gan>` lines.

## Parameters

  - why: Why nothing ran.

    >>> skipped("no gan> lines")
    {'ran': False, 'passed': False, 'why': 'no gan> lines', 'output': ''}
"""
    return {"ran": False, "passed": False, "why": why, "output": ""}


def module_path(name: str) -> str:
    """The file path a module must live at (GEP-0001-R013): `App.HelloWeb`
belongs in `app/hello_web.gan`, and a snippet whose file disagrees
fails the verdict for a reason that teaches nothing.

## Parameters

  - name: The module name, or nil when the source declares none.

    >>> module_path("App.DMagicFactor")
    'app/d_magic_factor.gan'
"""
    if (name is None):
        return "example.gan"
    else:
        path = gandora_std.enum.join(gandora_std.enum.map(gandora_std.string.split_on(name, "."), _snake), "/")
        return path + ".gan"


def module_name(source: str) -> str:
    """The module a source declares, or nil when it declares none.

## Parameters

  - source: The Gandora source text.

    >>> module_name("defmodule App.Shop do\\nend")
    'App.Shop'
"""
    m = re.compile("defmodule\\s+([A-Z][A-Za-z0-9_.]*)").match(source)
    if (m is None):
        return None
    else:
        return m.group(1)


def _snake(seg):
    def _gan_fn0(*_gan_args):
        match _gan_args:
            case ((c, i) as _gan_t0,) if isinstance(_gan_t0, tuple):
                return _snake_char(c, i)
        raise GanMatchError("no clause of _gan_fn0/1 matched " + repr(_gan_args))
    return gandora_std.enum.join(gandora_std.enum.map(gandora_std.enum.with_index(gandora_std.string.codepoints(seg)), _gan_fn0), "")


def _snake_char(c, i):
    if ((i > 0) and (c == gandora_std.string.upcase(c))) and (c != gandora_std.string.downcase(c)):
        return "_" + gandora_std.string.downcase(c)
    else:
        return gandora_std.string.downcase(c)


def _write_project(dir, source, env):
    relative = module_path(module_name(source))
    target = gandora_std.path.join(gandora_std.path.join(dir, "src"), relative)
    gandora_std.file.mkdir_p_bang(gandora_std.path.dirname(target))
    gandora_std.file.write_bang(gandora_std.path.join(dir, "gandora.jsonc"), config)
    gandora_std.file.write_bang(target, source)
    try:
        return os.symlink(env, gandora_std.path.join(dir, ".venv"))
    except Exception as _e:
        return None


def _check(dir):
    lsc = _tool("gan-lsc")
    if (lsc is None):
        return _no_verdict("gan-lsc not found — install gandora-lsp")
    else:
        _gan_val1 = gandora_std.system.cmd(lsc, ["check", "--root", dir], [("timeout", 180000), ("stderr_to_stdout", True)])
        match _gan_val1:
            case (out, _status) as _gan_t2 if isinstance(_gan_t2, tuple):
                pass
            case _:
                raise GanMatchError("no match of right-hand side value: " + repr(_gan_val1))
        try:
            return json.loads(out)
        except Exception as _e:
            return _no_verdict(gandora_std.string.trim(out))


def _no_verdict(why):
    return {"ok": False, "clean": False, "diagnostics": [], "suggestions": [], "why": why}


def _doctests(dir, source, env):
    if not (_gan_truthy(gandora_std.string.contains_p(source, "gan> "))):
        return skipped("no gan> lines in @example")
    else:
        cache = gandora_std.path.join(dir, "dist")
        try:
            files = gandora_std.enum.flat_map(core.build(dir, cache), lambda m: gandora_std.list.wrap(gandora_std.map.get(m, "python")))
            _gan_val3 = gandora_std.system.cmd(_python(env), ["-m", "doctest"] + files, [("cd", cache), ("timeout", 120000), ("stderr_to_stdout", True)])
            match _gan_val3:
                case (out, status) as _gan_t4 if isinstance(_gan_t4, tuple):
                    pass
                case _:
                    raise GanMatchError("no match of right-hand side value: " + repr(_gan_val3))
            return {"ran": True, "passed": status == 0, "why": "", "output": gandora_std.string.trim(out)}
        except Exception as e:
            return skipped("the artifact could not be run: " + str(e))


def _python(env):
    local = gandora_std.path.join(gandora_std.path.join(env, "bin"), "python")
    if _gan_truthy(gandora_std.file.exists_p(local)):
        return local
    else:
        return sys.executable


def _tool(bin):
    local = gandora_std.path.join(gandora_std.path.dirname(sys.executable), bin)
    if _gan_truthy(gandora_std.file.exists_p(local)):
        return local
    else:
        return gandora_std.system.find_executable(bin)
