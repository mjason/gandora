"""Artifact verification (GEP-0025-R008): after codegen, the generated
Python is checked with ty (resolution rules only) — undefined names,
dead imports, and missing module members are runtime-fatal facts, so
they join the build verdict as errors, mapped back to the .gan source.
"""

import builtins
import collections.abc
import difflib
import os
import pathlib
import re
import shutil
import subprocess
import sys
import gandora_std.enum
import gandora_std.map
import gandora_std.string


def _gan_truthy(value):
    return value is not None and value is not False

def _gan_and(value, then):
    return then() if _gan_truthy(value) else value

def _gan_or(value, then):
    return value if _gan_truthy(value) else then()

ty_config = "[tool.ty.src]\nrespect-ignore-files = false\n\n[tool.ty.rules]\nunsupported-operator = \"ignore\"\ninvalid-argument-type = \"ignore\"\ninvalid-return-type = \"ignore\"\ninvalid-assignment = \"ignore\"\npossibly-missing-attribute = \"ignore\"\ncall-non-callable = \"ignore\"\nno-matching-overload = \"ignore\"\nnot-iterable = \"ignore\"\nunsupported-bool-conversion = \"ignore\"\ninvalid-parameter-default = \"ignore\"\nredundant-cast = \"ignore\"\npossibly-unresolved-reference = \"ignore\"\n"

ty_config_strict = "[tool.ty.src]\nrespect-ignore-files = false\n"


def verify(root: str, cache: str, modules: collections.abc.Sequence[collections.abc.Mapping[str, object]], strict: bool = False) -> list[dict]:
    """Verifies compiled modules; returns diagnostics mapped to sources.
`modules` is the core.build result: maps with "python"/"source"/"module".
With `strict`, full type-flow findings join as warnings.

## Parameters

  - root: The project root (its .venv resolves third-party imports).
  - cache: The directory holding the compiled artifacts.
  - modules: The core.build module list.
  - strict: Also report type-flow findings (as warnings).
"""
    while True:
        ty = _ty_bin()
        if (ty is None) or not (_gan_truthy(os.path.isdir(cache))):
            return []
        else:
            if _gan_truthy(strict):
                _gan_tmp0 = ty_config_strict
            else:
                _gan_tmp0 = ty_config
            config = _gan_tmp0
            pathlib.Path(os.path.join(cache, "pyproject.toml")).write_text(config)
            py_env = _python_env(root)
            r = subprocess.run([ty, "check", "--output-format", "concise", "--exit-zero", "--project", cache, "--python", py_env, cache], capture_output=True, text=True, timeout=120, cwd=root)
            by_python = gandora_std.enum.reduce(modules, {}, lambda m, acc: gandora_std.map.put(acc, str(gandora_std.map.get(m, "python", "")), m))
            parsed = gandora_std.enum.map(r.stdout.strip().split("\n"), _parse_line)
            return [_map_to_source(d, by_python, root) for d in parsed if _gan_truthy(_gan_and(not ((d is None)), lambda: _gan_or(_gold_p(d), lambda: strict)))]


def _ty_bin():
    local = os.path.join(os.path.dirname(sys.executable), "ty")
    if _gan_truthy(os.path.exists(local)):
        return local
    else:
        return shutil.which("ty")


def _python_env(root):
    venv = os.path.join(root, ".venv")
    if _gan_truthy(os.path.exists(venv)):
        return venv
    else:
        return sys.prefix


def _parse_line(line):
    m = re.compile("^(.*?):(\\d+):(\\d+): (error|warning)\\[([a-z-]+)\\] (.*)$").search(line)
    if (m is None):
        return None
    else:
        return {"python": m.group(1), "pyline": builtins.int(m.group(2)), "severity": m.group(4), "rule": m.group(5), "message": m.group(6)}


def _gold_p(d):
    rule = gandora_std.map.get(d, "rule")
    msg = gandora_std.map.get(d, "message")
    if rule == "unresolved-reference":
        return True
    elif rule == "unresolved-import":
        return True
    elif rule == "unresolved-attribute":
        return gandora_std.string.starts_with_p(msg, "Module ")
    elif rule == "missing-argument":
        return True
    elif rule == "too-many-positional-arguments":
        return True
    elif rule == "unknown-argument":
        return True
    else:
        return False


def _map_to_source(d, by_python, root):
    p = str(gandora_std.map.get(d, "python"))
    if _gan_truthy(os.path.isabs(p)):
        _gan_tmp1 = p
    else:
        _gan_tmp1 = os.path.normpath(os.path.join(root, p))
    abs = _gan_tmp1
    mod = gandora_std.map.get(by_python, abs, {})
    source_path = str(gandora_std.map.get(mod, "source", gandora_std.map.get(d, "python")))
    source = _read_source(source_path)
    name = _offending_name(gandora_std.map.get(d, "message"))
    gan_name = demangle(name)
    line = _source_line(source, gan_name)
    if _gan_truthy(_gan_and(gandora_std.map.get(d, "rule") == "unresolved-import", lambda: builtins.hasattr(builtins, gan_name))):
        _gan_tmp2 = f" — `{gan_name}` is a Python builtin, not a module: write `$builtins.{gan_name}(...)`"
    elif gandora_std.map.get(d, "rule") == "unresolved-import":
        _gan_tmp2 = " — `$x`/`pyimport x` needs an importable module; see `gan lsc doc python`"
    else:
        _gan_tmp2 = _suggestion(source, gan_name)
    hint = _gan_tmp2
    if _gan_truthy(_gold_p(d)):
        _gan_tmp3 = "error"
    else:
        _gan_tmp3 = "warning"
    severity = _gan_tmp3
    if severity == "error":
        _gan_tmp4 = ""
    else:
        _gan_tmp4 = "[type] "
    tag = _gan_tmp4
    return {"severity": severity, "path": source_path, "line": line, "col": 0, "message": tag + (_rewrite(gandora_std.map.get(d, "message"), name, gan_name) + (hint + " (GEP-0025-R008, from the compiled artifact)"))}


def _read_source(path):
    try:
        return pathlib.Path(path).read_text()
    except Exception as _e:
        return ""


def _offending_name(msg):
    names = re.compile("`([A-Za-z_][A-Za-z0-9_.]*)`").findall(msg)
    if _gan_truthy(gandora_std.enum.empty_p(names)):
        return ""
    else:
        return gandora_std.enum.at(names, -1)


def demangle(name: str) -> str:
    """The Gandora spelling of a compiled Python identifier.

## Parameters

  - name: The Python identifier from a ty message.

    >>> demangle("member_p")
    'member?'
"""
    return gandora_std.string.replace(gandora_std.string.replace(name, "_bang", "!"), "_p", "?")


def _source_line(source, name):
    if name == "":
        return 0
    else:
        bare = gandora_std.string.replace(gandora_std.string.replace(name, "?", "\\?"), "!", "!")
        m = re.compile("\\b" + bare).search(source)
        if (m is None):
            return 0
        else:
            return source.count("\n", 0, m.start()) + 1


def _suggestion(source, name):
    if name == "":
        return ""
    else:
        idents = gandora_std.enum.uniq(re.compile("[a-z_][A-Za-z0-9_]*[?!]?").findall(source))
        near = gandora_std.enum.filter(difflib.get_close_matches(name, idents, n=3, cutoff=0.75), lambda n, *, name=name: n != name)
        if _gan_truthy(gandora_std.enum.empty_p(near)):
            return ""
        else:
            return f" — did you mean `{gandora_std.enum.at(near, 0)}`?"


def _rewrite(msg, name, gan_name):
    if (name == gan_name) or (name == ""):
        return msg
    else:
        return gandora_std.string.replace(msg, "`" + (name + "`"), "`" + (gan_name + "`"))
