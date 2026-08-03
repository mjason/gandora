"""Python-side intelligence for gan-lsp (GEP-0015): jedi answers hover,
completion, definition, and signatures for `$module` references and
`pyimport` aliases, resolved in the project's own environment.
"""

import builtins
import jedi
import re
import gandora_std.enum
import gandora_std.map
import gandora_std.string


def _gan_truthy(value):
    return value is not None and value is not False

def _gan_and(value, then):
    return then() if _gan_truthy(value) else value

class GanMatchError(Exception):
    pass

pyimport_re = re.compile("pyimport\\s+([A-Za-z0-9_.]+)\\s*,\\s*as:\\s*([a-z_][A-Za-z0-9_]*)")


def aliases(source: str) -> dict[str, str]:
    """## Parameters

  - source: The Gandora source holding pyimport declarations.
"""
    def _gan_fn0(*_gan_args):
        match _gan_args:
            case ((mod, name) as _gan_t0, acc,) if isinstance(_gan_t0, tuple):
                return gandora_std.map.put(acc, name, mod)
        raise GanMatchError("no clause of _gan_fn0/2 matched " + repr(_gan_args))
    return gandora_std.enum.reduce(pyimport_re.findall(source), {}, _gan_fn0)


def target(source: str, token: str, pyref: bool) -> tuple[str, str] | None:
    """## Parameters

  - source: The Gandora source, for alias resolution.
  - token: The dotted reference under the cursor.
  - pyref: Whether the token was written with a leading $.
"""
    first = gandora_std.enum.at(token.split("."), 0)
    if _gan_truthy(pyref):
        return ("import " + first, token)
    elif _gan_truthy(gandora_std.map.has_key_p(aliases(source), first)):
        mod = gandora_std.map.get(aliases(source), first)
        return ("import " + (mod + (" as " + first)), token)
    else:
        return None


def _script(root, import_line, expr):
    return jedi.Script(f"{import_line}\n{expr}", project=jedi.Project(root))


def hover_markdown(root: str, import_line: str, expr: str) -> str | None:
    """## Parameters

  - root: The project root jedi resolves in.
  - import_line: The synthesized import statement.
  - expr: The dotted expression to document.
"""
    try:
        names = builtins.list(_script(root, import_line, expr).help(2, gandora_std.string.length(expr)))
        _gan_case1 = names
        match _gan_case1:
            case [] as _gan_l2 if isinstance(_gan_l2, list):
                return None
            case [n, *_] as _gan_l3 if isinstance(_gan_l3, list):
                doc = n.docstring()
                if doc.strip() == "":
                    return f"`{expr}` — Python `{n.type}`"
                else:
                    shown = gandora_std.enum.join(gandora_std.enum.take(doc.split("\n"), 40), "\n")
                    return f"**`{expr}`** — Python {n.type}\n\n```text\n{shown.strip()}\n```"
            case _:
                raise GanMatchError("no case clause matched: " + repr(_gan_case1))
    except Exception as _e:
        return None


def complete(root: str, import_line: str, expr: str) -> list[dict]:
    """## Parameters

  - root: The project root jedi resolves in.
  - import_line: The synthesized import statement.
  - expr: The dotted prefix to complete.
"""
    try:
        return gandora_std.enum.map(gandora_std.enum.take(builtins.list(_script(root, import_line, expr).complete(2, gandora_std.string.length(expr))), 120), lambda c: {"name": c.name, "kind": c.type})
    except Exception as _e:
        return []


def goto(root: str, import_line: str, expr: str) -> dict | None:
    """## Parameters

  - root: The project root jedi resolves in.
  - import_line: The synthesized import statement.
  - expr: The dotted reference to locate.
"""
    try:
        names = builtins.list(_script(root, import_line, expr).goto(2, gandora_std.string.length(expr), follow_imports=True))
        _gan_case4 = names
        match _gan_case4:
            case [] as _gan_l5 if isinstance(_gan_l5, list):
                return None
            case [n, *_] as _gan_l6 if isinstance(_gan_l6, list):
                if (n.module_path is None) or (n.line is None):
                    return None
                else:
                    return {"path": str(n.module_path), "line0": n.line - 1, "col0": _or_zero(n.column)}
            case _:
                raise GanMatchError("no case clause matched: " + repr(_gan_case4))
    except Exception as _e:
        return None


def signatures(root: str, import_line: str, callee: str) -> list[dict]:
    """## Parameters

  - root: The project root jedi resolves in.
  - import_line: The synthesized import statement.
  - callee: The callable whose signatures to fetch.
"""
    try:
        def _gan_fn1(s):
            params = gandora_std.enum.map(builtins.list(s.params), lambda p: p.to_string())
            return {"label": s.to_string(), "params": params, "doc": _to_first_line(s.docstring(raw=True))}
        return gandora_std.enum.map(builtins.list(_script(root, import_line, callee + "()").get_signatures(2, gandora_std.string.length(callee) + 1)), _gan_fn1)
    except Exception as _e:
        return []


def infer_type(source_py, fn_names, var):
    try:
        lines = source_py.split("\n")
        start = gandora_std.enum.find_index(lines, lambda l: gandora_std.enum.any_p(fn_names, lambda n: l.startswith("def " + (n + "("))))
        if (start is None):
            return None
        else:
            var_re = _gan_and(_gan_and(jedi, lambda: re.compile("\\b")), lambda: None)
            return _find_and_infer(source_py, lines, start, var)
    except Exception as _e:
        return None


def _find_and_infer(source_py, lines, start, var):
    pattern = re.compile("\\b" + (re.escape(var) + "\\b"))
    total = gandora_std.enum.count(lines)
    hit = _scan_lines(start, start, lines, pattern, total)
    if (hit is None):
        return None
    else:
        _gan_val7 = hit
        match _gan_val7:
            case (line1, col) as _gan_t8 if isinstance(_gan_t8, tuple):
                pass
            case _:
                raise GanMatchError("no match of right-hand side value: " + repr(_gan_val7))
        names = builtins.list(jedi.Script(source_py).infer(line1, col))
        _gan_case9 = names
        match _gan_case9:
            case [] as _gan_l10 if isinstance(_gan_l10, list):
                return None
            case [n, *_] as _gan_l11 if isinstance(_gan_l11, list):
                return n.name
            case _:
                raise GanMatchError("no case clause matched: " + repr(_gan_case9))


def _scan_lines(i, start, lines, pattern, total):
    while True:
        if i >= total:
            return None
        elif _gan_truthy(_gan_and(i > start, lambda: gandora_std.enum.at(lines, i).startswith("def "))):
            return None
        else:
            m = pattern.search(gandora_std.enum.at(lines, i))
            if (m is None):
                i, start, lines, pattern, total = i + 1, start, lines, pattern, total
                continue
            else:
                _gan_val12 = m.span()
                match _gan_val12:
                    case (s, _e) as _gan_t13 if isinstance(_gan_t13, tuple):
                        pass
                    case _:
                        raise GanMatchError("no match of right-hand side value: " + repr(_gan_val12))
                return (i + 1, s)


def _to_first_line(text):
    line = gandora_std.enum.at(text.strip().split("\n"), 0)
    if (line is None):
        return ""
    else:
        return line


def _or_zero(v):
    if (v is None):
        return 0
    else:
        return v
