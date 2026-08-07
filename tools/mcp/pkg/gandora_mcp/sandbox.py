"""The verified-example engine (GEP-0028): a snippet becomes a throwaway
project, faces the same verdict a user's `gan build` would give it,
and its `@example` doctests are actually executed. Nothing returned
from here has gone unrun — that is the whole point of the surface.
"""

import gandora_core as core
import json
import os
import pathlib
import re
import shutil
import subprocess
import sys
import tempfile
import gandora_std.enum
import gandora_std.list
import gandora_std.map
import gandora_std.string


def _gan_truthy(value):
    return value is not None and value is not False

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
        shutil.rmtree(dir, ignore_errors=True)


def venv(root: str) -> str:
    """The environment a snippet is judged in: the project's own `.venv`
when it has one, otherwise the server's. A verdict must be about the
toolchain the project actually runs, not the one the server happens
to be installed in.

## Parameters

  - root: The project root.
"""
    local = os.path.join(root, ".venv")
    if _gan_truthy(os.path.isdir(local)):
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
    _gan_tmp0 = [_snake_char(c, i) for _gan_for1 in gandora_std.enum.with_index(gandora_std.string.codepoints(seg)) if isinstance(_gan_for1, tuple) and len(_gan_for1) == 2 for (c, i,) in [(_gan_for1[0], _gan_for1[1],)]]
    chars = _gan_tmp0
    return gandora_std.enum.join(chars, "")


def _snake_char(c, i):
    if ((i > 0) and (c == gandora_std.string.upcase(c))) and (c != gandora_std.string.downcase(c)):
        return "_" + gandora_std.string.downcase(c)
    else:
        return gandora_std.string.downcase(c)


def _write_project(dir, source, env):
    target = os.path.join(dir, "src", module_path(module_name(source)))
    os.makedirs(os.path.dirname(target), exist_ok=True)
    pathlib.Path(os.path.join(dir, "gandora.jsonc")).write_text(config)
    pathlib.Path(target).write_text(source)
    try:
        return os.symlink(env, os.path.join(dir, ".venv"))
    except Exception as _e:
        return None


def _check(dir):
    lsc = _tool("gan-lsc")
    if (lsc is None):
        return {"ok": False, "clean": False, "diagnostics": [], "suggestions": [], "why": "gan-lsc not found — install gandora-lsp"}
    else:
        r = subprocess.run([lsc, "check", "--root", dir], capture_output=True, text=True, timeout=180)
        try:
            return json.loads(r.stdout)
        except Exception as _e:
            return {"ok": False, "clean": False, "diagnostics": [], "suggestions": [], "why": gandora_std.string.trim(r.stderr)}


def _doctests(dir, source, env):
    if not (_gan_truthy(gandora_std.string.contains_p(source, "gan> "))):
        return skipped("no gan> lines in @example")
    else:
        cache = os.path.join(dir, "dist")
        try:
            files = gandora_std.enum.flat_map(core.build(dir, cache), lambda m: gandora_std.list.wrap(gandora_std.map.get(m, "python")))
            r = subprocess.run([_python(env), "-m", "doctest"] + files, capture_output=True, text=True, timeout=120, cwd=cache)
            return {"ran": True, "passed": r.returncode == 0, "why": "", "output": gandora_std.string.trim(r.stdout + r.stderr)}
        except Exception as e:
            return skipped("the artifact could not be run: " + str(e))


def _python(env):
    local = os.path.join(env, "bin", "python")
    if _gan_truthy(os.path.exists(local)):
        return local
    else:
        return sys.executable


def _tool(bin):
    path = os.path.join(os.path.dirname(sys.executable), bin)
    if _gan_truthy(os.path.exists(path)):
        return path
    else:
        return shutil.which(bin)
