"""Artifact verification (GEP-0025-R008): after codegen, the generated
Python is checked with ty (resolution rules only) — undefined names,
dead imports, and missing module members are runtime-fatal facts, so
they join the build verdict as errors, mapped back to the .gan source.
"""

import builtins
import collections.abc
import difflib
import re
import sys
import gandora_std.enum
import gandora_std.file
import gandora_std.map
import gandora_std.path
import gandora_std.string
import gandora_std.system


def _gan_truthy(value):
    return value is not None and value is not False

def _gan_and(value, then):
    return then() if _gan_truthy(value) else value

def _gan_or(value, then):
    return value if _gan_truthy(value) else then()

class GanMatchError(Exception):
    pass

lenient_rules = ["unsupported-operator", "invalid-argument-type", "invalid-return-type", "invalid-assignment", "possibly-missing-attribute", "call-non-callable", "no-matching-overload", "not-iterable", "unsupported-bool-conversion", "invalid-parameter-default", "redundant-cast", "possibly-unresolved-reference"]

ty_config_strict = "[tool.ty.src]\nrespect-ignore-files = false\n"

gold_rules = ["unresolved-reference", "unresolved-import", "missing-argument", "too-many-positional-arguments", "unknown-argument"]


def _config(strict):
    if _gan_truthy(strict):
        return ty_config_strict
    else:
        ignores = gandora_std.enum.join(gandora_std.enum.map(lenient_rules, lambda r: r + " = \"ignore\""), "\n")
        return ty_config_strict + ("\n[tool.ty.rules]\n" + (ignores + "\n"))


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
        if (ty is None) or not (_gan_truthy(gandora_std.file.dir_p(cache))):
            return []
        else:
            gandora_std.file.write_bang(gandora_std.path.join(cache, "pyproject.toml"), _config(strict))
            _gan_val0 = gandora_std.system.cmd(ty, ["check", "--output-format", "concise", "--exit-zero", "--project", cache, "--python", _python_env(root), cache], [("cd", root), ("timeout", 120000)])
            match _gan_val0:
                case (out, _status) as _gan_t1 if isinstance(_gan_t1, tuple):
                    pass
                case _:
                    raise GanMatchError("no match of right-hand side value: " + repr(_gan_val0))
            by_python = gandora_std.enum.reduce(modules, {}, lambda m, acc: gandora_std.map.put(acc, str(gandora_std.map.get(m, "python", "")), m))
            parsed = gandora_std.enum.map(gandora_std.string.split_on(gandora_std.string.trim(out), "\n"), _parse_line)
            return [_map_to_source(d, by_python, root) for d in parsed if _gan_truthy(_gan_and(not ((d is None)), lambda: _gan_or(_gold_p(d), lambda: strict)))]


def _ty_bin():
    local = gandora_std.path.join(gandora_std.path.dirname(sys.executable), "ty")
    if _gan_truthy(gandora_std.file.exists_p(local)):
        return local
    else:
        return gandora_std.system.find_executable("ty")


def _python_env(root):
    venv = gandora_std.path.join(root, ".venv")
    if _gan_truthy(gandora_std.file.exists_p(venv)):
        return venv
    else:
        return sys.prefix


def _parse_line(line):
    m = re.compile("^(.*?):(\\d+):(\\d+): (error|warning)\\[([a-z-]+)\\] (.*)$").match(line)
    if (m is None):
        return None
    else:
        return {"python": m.group(1), "pyline": builtins.int(m.group(2)), "severity": m.group(4), "rule": m.group(5), "message": m.group(6)}


def _gold_p(d):
    return _gan_or(gandora_std.enum.member_p(gold_rules, gandora_std.map.get(d, "rule")), lambda: _gan_and(gandora_std.map.get(d, "rule") == "unresolved-attribute", lambda: gandora_std.string.starts_with_p(gandora_std.map.get(d, "message"), "Module ")))


def _map_to_source(d, by_python, root):
    p = str(gandora_std.map.get(d, "python"))
    if _gan_truthy(gandora_std.path.absolute_p(p)):
        _gan_tmp2 = p
    else:
        _gan_tmp2 = gandora_std.path.expand(gandora_std.path.join(root, p))
    abs = _gan_tmp2
    mod = gandora_std.map.get(by_python, abs, {})
    source_path = str(gandora_std.map.get(mod, "source", gandora_std.map.get(d, "python")))
    source = _read_source(source_path)
    name = _offending_name(gandora_std.map.get(d, "message"))
    gan_name = demangle(name)
    line = _source_line(source, gan_name)
    if _gan_truthy(_gan_and(gandora_std.map.get(d, "rule") == "unresolved-import", lambda: builtins.hasattr(builtins, gan_name))):
        _gan_tmp3 = f" — `{gan_name}` is a Python builtin, not a module: write `$builtins.{gan_name}(...)`"
    elif gandora_std.map.get(d, "rule") == "unresolved-import":
        _gan_tmp3 = " — `$x`/`pyimport x` needs an importable module; see `gan lsc doc python`"
    else:
        _gan_tmp3 = _suggestion(source, gan_name)
    hint = _gan_tmp3
    if _gan_truthy(_gold_p(d)):
        _gan_tmp4 = "error"
    else:
        _gan_tmp4 = "warning"
    severity = _gan_tmp4
    if severity == "error":
        _gan_tmp5 = ""
    else:
        _gan_tmp5 = "[type] "
    tag = _gan_tmp5
    return {"severity": severity, "path": source_path, "line": line, "col": 0, "message": tag + (_rewrite(gandora_std.map.get(d, "message"), name, gan_name) + (hint + " (GEP-0025-R008, from the compiled artifact)"))}


def _read_source(path):
    _gan_case6 = gandora_std.file.read(path)
    match _gan_case6:
        case ("ok", text) as _gan_t7 if isinstance(_gan_t7, tuple):
            return text
        case ("error", _why) as _gan_t8 if isinstance(_gan_t8, tuple):
            return ""
        case _:
            raise GanMatchError("no case clause matched: " + repr(_gan_case6))


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
        idents = gandora_std.enum.uniq(re.compile("(?<![A-Za-z0-9_])[a-z_][A-Za-z0-9_]*[?!]?").findall(source))
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
