"""The AI sandbox (GEP-0023): one query that answers "is this Gandora
code right, and if not, what did I probably mean?" — parse, compile,
lint, spell-check module members with fuzzy search, flag common
cross-language mistakes, then execute with a timeout. One JSON value
out; nothing touches the project.
"""

import builtins
import difflib
import gandora_core as core
import os
import pathlib
import re
import subprocess
import sys
import tempfile
import gandora_std.enum
import gandora_std.map
import gandora_std.string


def _gan_truthy(value):
    return value is not None and value is not False

def _gan_and(value, then):
    return then() if _gan_truthy(value) else value

def _gan_or(value, then):
    return value if _gan_truthy(value) else then()

class GanMatchError(Exception):
    pass

mistakes = [(re.compile("^\\s*return\\b"), "Gandora has no `return` — a function's value is its last expression."), (re.compile("^\\s*while\\b"), "There is no `while` — use tail recursion (constant stack) or `for`/Enum (GEP-0019/0020)."), (re.compile("\\belif\\b"), "`elif` is spelled as another `cond do` branch, or a `case` clause."), (re.compile("^\\s*def .*\\):\\s*$"), "Python `def ...():` — a Gandora head has no colon: `def f(x) do ... end` or `def f(x), do: expr`."), (re.compile("\\blambda\\b"), "`lambda` is spelled `fn x -> ... end`, called with `f.(x)`."), (re.compile("\\bNone\\b"), "`None` is spelled `nil`."), (re.compile("\\bTrue\\b|\\bFalse\\b"), "Booleans are lowercase: `true` / `false` (only `false` and `nil` are falsy)."), (re.compile("^\\s*import\\s+[a-z]"), "Python `import x` is `pyimport x` at module top, or an inline `$x` reference (GEP-0003)."), (re.compile("^\\s*from\\s+\\w+\\s+import\\b"), "`from x import y` has no direct spelling — `pyimport x` then `x.y`, or `$x.y` inline."), (re.compile("\\bprint\\("), "Prefer `IO.puts(...)` (falls through to Python print, but IO.puts is the idiom)."), (re.compile("\\bself\\."), "There is no `self` — Gandora functions are module functions over plain data."), (re.compile("&&|\\|\\|(?!>)"), "Boolean operators are the words `and` / `or` / `not`."), (re.compile("\\+=|-=|\\*="), "No augmented assignment — rebind: `x = x + 1`."), (re.compile("\\bf\""), "f-strings are plain strings — interpolation is built in: \"total " + ("#" + "{n}\".")), (re.compile("\\bswitch\\b"), "`switch` is `case ... do pattern -> ... end`."), (re.compile("\\bnil\\s*==|==\\s*nil\\b"), "Prefer `is_nil(x)` over comparing with nil."), (re.compile("\\$\"[A-Za-z0-9_.]+\""), "Quoted module refs were retired: write $(a.b) (GEP-0003-R010).")]

keywords = ["defmodule", "def", "defp", "defmacro", "defstruct", "case", "cond", "with", "for", "fn", "try", "rescue", "after", "else", "end", "do", "quote", "unquote", "pyimport", "import", "require", "alias", "use", "recur", "when", "true", "false", "nil", "and", "or", "not", "raise", "if", "unless"]


def try_source(source: str, root: str, run: bool) -> dict:
    """Runs the whole verdict pipeline over `source`. Returns a map:
ok / stage (parse|compile|lint|run|ok) / diagnostics / suggestions /
python / stdout / value.

## Parameters

  - source: The Gandora code to check — a full module or bare statements.
  - root: Project root for module resolution (std is always known).
  - run: Execute after a clean compile (with a timeout) when true.
"""
    suggestions = _common_mistakes(source) + _practice_hints(source)
    module_mode = gandora_std.string.match_p(source, re.compile("(?m)^\\s*defmodul\\w*\\s"))
    try:
        if _gan_truthy(module_mode):
            _gan_tmp0 = ("ok", core.compile_string(source, "sandbox.gan", root))
        else:
            _gan_tmp0 = ("ok", core.compile_snippet(source, root))
    except core.CompileError as e:
        _gan_tmp0 = ("error", builtins.list(e.args))
    compiled = _gan_tmp0
    _gan_case1 = compiled
    match _gan_case1:
        case ("error", args) as _gan_t2 if isinstance(_gan_t2, tuple):
            msg = gandora_std.enum.at(args, 0)
            line = gandora_std.enum.at(args, 2)
            return {"ok": False, "stage": "compile", "diagnostics": [{"severity": "error", "line": line, "col": gandora_std.enum.at(args, 3), "message": msg}], "suggestions": suggestions + _error_suggestions(source, msg, line), "python": None, "stdout": None, "value": None}
        case ("ok", python) as _gan_t3 if isinstance(_gan_t3, tuple):
            if _gan_truthy(module_mode):
                try:
                    _gan_tmp4 = core.diagnostics(source, "sandbox.gan", root)
                except Exception as _e:
                    _gan_tmp4 = []
            else:
                _gan_tmp4 = []
            lints = _gan_tmp4
            member_sugg = _member_suggestions(source, root)
            lint_sugg = gandora_std.enum.flat_map(lints, lambda d, *, source=source: _lint_suggestions(source, d))
            all_sugg = suggestions + (member_sugg + lint_sugg)
            if _gan_truthy(run):
                _gan_val5 = _execute(python, module_mode, source, root)
                match _gan_val5:
                    case (out, value, err) as _gan_t6 if isinstance(_gan_t6, tuple):
                        pass
                    case _:
                        raise GanMatchError("no match of right-hand side value: " + repr(_gan_val5))
                ok = _gan_and((err is None), lambda: gandora_std.enum.empty_p(lints))
                if (err is None):
                    _gan_tmp7 = "ok"
                else:
                    _gan_tmp7 = "run"
                return {"ok": ok, "stage": _gan_tmp7, "diagnostics": lints + _run_diag(err), "suggestions": all_sugg, "python": python, "stdout": out, "value": value}
            else:
                if _gan_truthy(gandora_std.enum.empty_p(lints)):
                    _gan_tmp8 = "ok"
                else:
                    _gan_tmp8 = "lint"
                return {"ok": gandora_std.enum.empty_p(lints), "stage": _gan_tmp8, "diagnostics": lints, "suggestions": all_sugg, "python": python, "stdout": None, "value": None}
        case _:
            raise GanMatchError("no case clause matched: " + repr(_gan_case1))


def _run_diag(err):
    if (err is None):
        return []
    else:
        return [{"severity": "error", "line": 0, "col": 0, "message": err}]


def _common_mistakes(source):
    lines = source.split("\n")
    def _gan_fn0(*_gan_args, lines=lines):
        match _gan_args:
            case ((pattern, advice) as _gan_t9,) if isinstance(_gan_t9, tuple):
                hit = gandora_std.enum.find_index(lines, lambda l, *, pattern=pattern: not ((pattern.search(l) is None)))
                if (hit is None):
                    return []
                else:
                    return [{"kind": "migration", "line": hit + 1, "message": advice}]
        raise GanMatchError("no clause of _gan_fn0/1 matched " + repr(_gan_args))
    return gandora_std.enum.flat_map(mistakes, _gan_fn0)


def _practice_hints(source):
    heads = re.compile("(?m)^\\s*def ([a-z_][A-Za-z0-9_]*[?!]?)").findall(source)
    specs = re.compile("(?m)^\\s*@spec ([a-z_][A-Za-z0-9_]*[?!]?)").findall(source)
    missing = gandora_std.enum.uniq(gandora_std.enum.filter(heads, lambda h, *, specs=specs: not (_gan_truthy(gandora_std.enum.member_p(specs, h)))))
    if _gan_truthy(_gan_or(gandora_std.enum.empty_p(missing), lambda: not (_gan_truthy(gandora_std.string.match_p(source, re.compile("defmodule")))))):
        _gan_tmp10 = []
    else:
        _gan_fstr11 = gandora_std.enum.join(missing, ", ")
        _gan_tmp10 = [{"kind": "practice", "line": 0, "message": f"Public defs without @spec: {_gan_fstr11} — the standard is @doc + @spec on every public def (docs/syntax.md)."}]
    spec_hint = _gan_tmp10
    refs = re.compile("\\$([a-z_][a-z0-9_]*)").findall(source)
    counts = gandora_std.enum.reduce(refs, {}, lambda m, acc: gandora_std.map.put(acc, m, gandora_std.map.get(acc, m, 0) + 1))
    def _gan_fn1(*_gan_args):
        match _gan_args:
            case ((_, n) as _gan_t12,) if isinstance(_gan_t12, tuple):
                return n >= 3
        raise GanMatchError("no clause of _gan_fn1/1 matched " + repr(_gan_args))
    def _gan_fn2(*_gan_args):
        match _gan_args:
            case ((m, n) as _gan_t13,) if isinstance(_gan_t13, tuple):
                return {"kind": "practice", "line": 0, "message": f"`${m}` appears {n} times — declare `pyimport {m}` once and use the bare name (GEP-0003 rev 6)."}
        raise GanMatchError("no clause of _gan_fn2/1 matched " + repr(_gan_args))
    py_hint = gandora_std.enum.map(gandora_std.enum.filter(counts.items(), _gan_fn1), _gan_fn2)
    return spec_hint + py_hint


def _error_suggestions(source, msg, line):
    from_msg = re.compile("'([A-Za-z_][A-Za-z0-9_.!?]*)'").findall(str(msg))
    if (line is None) or (line == 0):
        _gan_tmp14 = re.compile("[A-Za-z_][A-Za-z0-9_]*").findall(source)
    else:
        l = gandora_std.enum.at(source.split("\n"), line - 1)
        if (l is None):
            _gan_tmp14 = []
        else:
            _gan_tmp14 = re.compile("[A-Za-z_][A-Za-z0-9_]*").findall(l)
    at_line = _gan_tmp14
    def _gan_fn3(word, *, line=line):
        near = difflib.get_close_matches(word, keywords, n=1, cutoff=0.8)
        if _gan_truthy(_gan_or(gandora_std.enum.empty_p(near), lambda: gandora_std.enum.member_p(keywords, word))):
            return []
        else:
            return [{"kind": "did_you_mean", "line": line, "message": f"`{word}` — did you mean `{gandora_std.enum.at(near, 0)}`?"}]
    return gandora_std.enum.flat_map(gandora_std.enum.uniq(from_msg + at_line), _gan_fn3)


def _member_suggestions(source, root):
    calls = re.compile("\\b([A-Z][A-Za-z0-9_.]*)\\.([a-z_][A-Za-z0-9_]*[?!]?)\\(").findall(source)
    def _gan_fn4(*_gan_args, root=root):
        match _gan_args:
            case ((mod, fun) as _gan_t15,) if isinstance(_gan_t15, tuple):
                names = _module_functions(mod, root)
                if _gan_truthy(gandora_std.enum.empty_p(names)):
                    return []
                elif _gan_truthy(gandora_std.enum.member_p(names, fun)):
                    return []
                else:
                    near = difflib.get_close_matches(fun, names, n=3, cutoff=0.6)
                    if _gan_truthy(gandora_std.enum.empty_p(near)):
                        _gan_tmp16 = f"no close match; see `gan lsc symbols {mod}`"
                    else:
                        _gan_fstr17 = gandora_std.enum.join(gandora_std.enum.map(near, lambda n, *, mod=mod: f"`{mod}.{n}`"), " / ")
                        _gan_tmp16 = f"did you mean {_gan_fstr17}?"
                    hint = _gan_tmp16
                    return [{"kind": "did_you_mean", "line": 0, "message": f"`{mod}.{fun}` is not a function of {mod} — {hint}"}]
        raise GanMatchError("no clause of _gan_fn4/1 matched " + repr(_gan_args))
    return gandora_std.enum.flat_map(gandora_std.enum.uniq(calls), _gan_fn4)


def _module_functions(mod, root):
    try:
        _gan_tmp18 = core.symbols(mod, root)
    except Exception as _e:
        _gan_tmp18 = []
    syms = _gan_tmp18
    return gandora_std.enum.map(syms, lambda s: gandora_std.map.get(s, "name"))


def _lint_suggestions(source, d):
    m = re.compile("variable ([A-Za-z0-9_?!]+) is never bound").search(gandora_std.map.get(d, "message", ""))
    if (m is None):
        return []
    else:
        word = m.group(1)
        idents = gandora_std.enum.uniq(re.compile("[a-z_][A-Za-z0-9_]*").findall(source))
        near = gandora_std.enum.filter(difflib.get_close_matches(word, idents, n=3, cutoff=0.7), lambda n, *, word=word: n != word)
        if _gan_truthy(gandora_std.enum.empty_p(near)):
            return []
        else:
            _gan_fstr19 = gandora_std.enum.join(gandora_std.enum.map(near, lambda n: f"`{n}`"), " / ")
            return [{"kind": "did_you_mean", "line": gandora_std.map.get(d, "line"), "message": f"`{word}` — did you mean {_gan_fstr19}?"}]


def _sandbox_python(root):
    venv = os.path.abspath(os.path.join(root, ".venv", "bin", "python"))
    if _gan_truthy(os.path.exists(venv)):
        return venv
    else:
        return sys.executable


def _execute(python, module_mode, source, root):
    dir = tempfile.mkdtemp(prefix="gan-sandbox-")
    path = os.path.join(dir, "sandbox_mod.py")
    has_main = _gan_and(module_mode, lambda: gandora_std.string.match_p(source, re.compile("(?m)^\\s*def main\\(")))
    has_guard = gandora_std.string.contains_p(python, "if __name__ ==")
    if _gan_truthy(_gan_and(has_main, lambda: not (_gan_truthy(has_guard)))):
        _gan_tmp20 = python + "\n\nmain()\n"
    elif _gan_truthy(module_mode):
        _gan_tmp20 = python
    else:
        _gan_tmp20 = python + "\ntry:\n    print(\"__gan_value__\", repr(_))\nexcept NameError:\n    pass\n"
    code = _gan_tmp20
    pathlib.Path(path).write_text(code)
    try:
        _gan_tmp21 = ("ok", subprocess.run([_sandbox_python(root), path], capture_output=True, text=True, timeout=10, cwd=dir))
    except subprocess.TimeoutExpired as _e:
        _gan_tmp21 = "timeout"
    result = _gan_tmp21
    _gan_case22 = result
    match _gan_case22:
        case "timeout":
            return (None, None, "execution timed out after 10s")
        case ("ok", r) as _gan_t23 if isinstance(_gan_t23, tuple):
            _gan_val24 = _split_value(r.stdout)
            match _gan_val24:
                case (out, value) as _gan_t25 if isinstance(_gan_t25, tuple):
                    pass
                case _:
                    raise GanMatchError("no match of right-hand side value: " + repr(_gan_val24))
            if r.returncode == 0:
                return (out, value, None)
            else:
                return (out, value, _last_error_line(r.stderr))
        case _:
            raise GanMatchError("no case clause matched: " + repr(_gan_case22))


def _split_value(stdout):
    lines = stdout.split("\n")
    marked = gandora_std.enum.find(lines, lambda l: gandora_std.string.starts_with_p(l, "__gan_value__ "))
    plain = gandora_std.enum.join(gandora_std.enum.reject(lines, lambda l: gandora_std.string.starts_with_p(l, "__gan_value__ ")), "\n")
    if (marked is None):
        _gan_tmp26 = None
    else:
        _gan_tmp26 = gandora_std.string.replace(marked, "__gan_value__ ", "")
    value = _gan_tmp26
    return (plain, value)


def _last_error_line(stderr):
    return gandora_std.enum.at(gandora_std.enum.reject(stderr.strip().split("\n"), lambda l: l.strip() == ""), -1)
