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

mistakes = [(re.compile("^\\s*return\\b"), "Gandora has no `return` — a function's value is its last expression."), (re.compile("^\\s*while\\b"), "There is no `while` — use tail recursion (constant stack) or `for`/Enum (GEP-0019/0020)."), (re.compile("\\belif\\b"), "`elif` is spelled as another `cond do` branch, or a `case` clause."), (re.compile("^\\s*def .*\\):\\s*$"), "Python `def ...():` — a Gandora head has no colon: `def f(x) do ... end` or `def f(x), do: expr`."), (re.compile("\\blambda\\b"), "`lambda` is spelled `fn x -> ... end`, called with `f.(x)`."), (re.compile("\\bNone\\b"), "`None` is spelled `nil`."), (re.compile("\\bTrue\\b|\\bFalse\\b"), "Booleans are lowercase: `true` / `false` (only `false` and `nil` are falsy)."), (re.compile("^\\s*import\\s+[a-z]"), "Python `import x` is `pyimport x` at module top, or an inline `$x` reference (GEP-0003)."), (re.compile("^\\s*from\\s+\\w+\\s+import\\b"), "`from x import y` has no direct spelling — `pyimport x` then `x.y`, or `$x.y` inline."), (re.compile("\\bprint\\("), "Prefer `IO.puts(...)` (falls through to Python print, but IO.puts is the idiom)."), (re.compile("\\bself\\."), "There is no `self` — Gandora functions are module functions over plain data."), (re.compile("&&|\\|\\|(?!>)"), "Boolean operators are the words `and` / `or` / `not`."), (re.compile("\\+=|-=|\\*="), "No augmented assignment — rebind: `x = x + 1`."), (re.compile("\\bf\""), "f-strings are plain strings — interpolation is built in: \"total " + ("#" + "{n}\".")), (re.compile("\\bswitch\\b"), "`switch` is `case ... do pattern -> ... end`."), (re.compile("\\bnil\\s*==|==\\s*nil\\b"), "Prefer `is_nil(x)` over comparing with nil."), (re.compile("\\$\""), "Quoted module refs were retired: write $(a.b) (GEP-0003-R010).")]

keywords = ["defmodule", "def", "defp", "defmacro", "defstruct", "case", "cond", "with", "for", "fn", "try", "rescue", "after", "else", "end", "do", "quote", "unquote", "pyimport", "import", "require", "alias", "use", "recur", "when", "true", "false", "nil", "and", "or", "not", "raise", "if", "unless"]


def help() -> str:
    """The skill guide printed by `gan lsc try` / `try --help`."""
    return "gan try - the Gandora sandbox (GEP-0023)\n\nValidate generated code BEFORE it touches a project. One JSON\nverdict: compile + lints + suggestions + sandboxed execution.\n\nUSAGE\n  echo 'code' | gan try -                   # stdin (from the project root)\n  gan try snippet.gan                       # file\n  gan try - --no-run < f.gan                # verdict without executing\n  (gan try == gan lsc try; --root <dir> sets the project)\n\nInput is a full module (defmodule ... end) or bare statements.\nExit code: 0 when ok, 1 otherwise - safe to chain in scripts.\n\nOUTPUT (one JSON object)\n  ok           true = compiled clean, no lints, ran without error\n  stage        compile | lint | run | ok - where it stopped\n  diagnostics  compiler errors + GEP-0022 lints, with line/col\n  suggestions  see KINDS below\n  python       the generated Python (inspect what your code becomes)\n  stdout       captured program output (10s timeout, temp dir)\n  value        repr of the last expression (bare-statement mode)\n\nSUGGESTION KINDS\n  did_you_mean  fuzzy match against REAL candidates:\n                Enum.mpa -> Enum.map (actual module symbols),\n                valeu -> value (your own identifiers),\n                defmodul -> defmodule (keywords)\n  migration     a cross-language habit with the Gandora spelling:\n                return / while / lambda / None / import / self. /\n                && / += / f\"...\" / Python def-colon / switch\n  practice      the documented standards (docs/syntax.md):\n                annotation coverage (@doc/@spec/@moduledoc/@example),\n                abstract containers in @spec parameters,\n                map+filter chains -> for, fn x -> f(x) end -> &f/1,\n                count == 0 -> Enum.empty?, bare rescue,\n                repeated $mod -> pyimport\n\nTHE LOOP (for agents)\n  generate -> try -> apply every suggestion -> try again -> only\n  then write into the project. Then: gan lsc check, gan test,\n  gan fmt src. A clean module returns \"suggestions\": [] - treat\n  anything else as work.\n\nStrings, docs, and comments never trigger suggestions - only code.\nNothing is written outside a temp directory.\n"


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
            return {"ok": False, "stage": "compile", "diagnostics": [{"severity": "error", "line": line, "col": gandora_std.enum.at(args, 3), "message": msg}], "suggestions": _dedupe(suggestions + _error_suggestions(source, msg, line)), "python": None, "stdout": None, "value": None}
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
            all_sugg = _dedupe(suggestions + (member_sugg + lint_sugg))
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


def _dedupe(suggestions: list[dict]) -> list[dict]:
    def _gan_fn0(*_gan_args):
        match _gan_args:
            case (s, (acc, seen) as _gan_t9,) if isinstance(_gan_t9, tuple):
                msg = gandora_std.map.get(s, "message")
                if _gan_truthy(gandora_std.enum.member_p(seen, msg)):
                    return (acc, seen)
                else:
                    return (acc + [s], seen + [msg])
        raise GanMatchError("no clause of _gan_fn0/2 matched " + repr(_gan_args))
    _gan_val10 = gandora_std.enum.reduce(suggestions, ([], []), _gan_fn0)
    match _gan_val10:
        case (out, _) as _gan_t11 if isinstance(_gan_t11, tuple):
            pass
        case _:
            raise GanMatchError("no match of right-hand side value: " + repr(_gan_val10))
    return out


def _run_diag(err):
    if (err is None):
        return []
    else:
        return [{"severity": "error", "line": 0, "col": 0, "message": err}]


def _mask_literals(source: str) -> str:
    spaces = lambda text: re.sub("\\S", " ", text)
    masked = re.sub("\"\"\"([\\s\\S]*?)\"\"\"", lambda m, *, spaces=spaces: "\"\"\"" + (spaces(m.group(1)) + "\"\"\""), source)
    masked = re.sub("(~[a-zA-Z]+\\()([^)]*)\\)", lambda m, *, spaces=spaces: m.group(1) + (spaces(m.group(2)) + ")"), masked)
    masked = re.sub("(~[a-zA-Z]+/)([^/\\n]*)/", lambda m, *, spaces=spaces: m.group(1) + (spaces(m.group(2)) + "/"), masked)
    masked = re.sub("\"((?:[^\"\\\\\\n]|\\\\.)*)\"", lambda m, *, spaces=spaces: "\"" + (spaces(m.group(1)) + "\""), masked)
    return re.sub("#(?!\\{)([^\\n]*)", lambda m, *, spaces=spaces: "#" + spaces(m.group(1)), masked)


def _common_mistakes(source):
    lines = _mask_literals(source).split("\n")
    def _gan_fn1(*_gan_args, lines=lines):
        match _gan_args:
            case ((pattern, advice) as _gan_t12,) if isinstance(_gan_t12, tuple):
                hit = gandora_std.enum.find_index(lines, lambda l, *, pattern=pattern: not ((pattern.search(l) is None)))
                if (hit is None):
                    return []
                else:
                    return [{"kind": "migration", "line": hit + 1, "message": advice}]
        raise GanMatchError("no clause of _gan_fn1/1 matched " + repr(_gan_args))
    return gandora_std.enum.flat_map(mistakes, _gan_fn1)


def _practice_hints(raw):
    source = _mask_literals(raw)
    in_module = gandora_std.string.match_p(source, re.compile("defmodule"))
    return _coverage_hints(raw, source, in_module) + (_spec_container_hints(source) + (_idiom_hints(source) + _pyimport_hints(source)))


def _coverage_hints(raw, source, in_module):
    if not (_gan_truthy(in_module)):
        return []
    else:
        heads = gandora_std.enum.uniq(re.compile("(?m)^\\s*def ([a-z_][A-Za-z0-9_]*[?!]?)").findall(source))
        specs = re.compile("(?m)^\\s*@spec ([a-z_][A-Za-z0-9_]*[?!]?)").findall(source)
        docs = _count_attr_blocks(source, "@doc")
        no_spec = gandora_std.enum.filter(heads, lambda h, *, specs=specs: not (_gan_truthy(gandora_std.enum.member_p(specs, h))))
        missing = []
        if _gan_truthy(gandora_std.enum.empty_p(no_spec)):
            _gan_tmp13 = missing
        else:
            _gan_fstr14 = gandora_std.enum.join(no_spec, ", ")
            _gan_tmp13 = missing + [f"@spec on: {_gan_fstr14}"]
        missing = _gan_tmp13
        if docs >= gandora_std.enum.count(heads):
            _gan_tmp15 = missing
        else:
            _gan_tmp15 = missing + [f"@doc on {gandora_std.enum.count(heads) - docs} public def(s)"]
        missing = _gan_tmp15
        if _gan_truthy(gandora_std.string.contains_p(source, "@moduledoc")):
            _gan_tmp16 = missing
        else:
            _gan_tmp16 = missing + ["@moduledoc"]
        missing = _gan_tmp16
        if (gandora_std.enum.count(heads) > 0) and not (_gan_truthy(gandora_std.string.contains_p(raw, "@example"))):
            _gan_tmp17 = [{"kind": "practice", "line": 0, "message": "No @example doctests — add one right above a def, e.g.:\n@example \"\"\"\n    gan> double(21)\n    42\n\"\"\"\n(`gan test` runs them; expected output is the Python repr) (GEP-0007)."}]
        else:
            _gan_tmp17 = []
        example_hint = _gan_tmp17
        if _gan_truthy(gandora_std.enum.empty_p(missing)):
            _gan_tmp18 = []
        else:
            _gan_fstr19 = gandora_std.enum.join(missing, "; ")
            _gan_tmp18 = [{"kind": "practice", "line": 0, "message": f"Annotation coverage: missing {_gan_fstr19} — e.g. @doc \"What it does.\" then @spec name(integer()) :: integer() above the def; a printing entry is @spec main() :: nil; unsure of a type? read the cheat sheet: gan lsc doc spec (docs/syntax.md)."}]
        head_hint = _gan_tmp18
        return head_hint + example_hint


def _count_attr_blocks(source, attr):
    return gandora_std.enum.count(re.compile(f"(?m)^\\s*{attr} ").findall(source))


def _spec_container_hints(source):
    def _gan_fn2(*_gan_args):
        match _gan_args:
            case ((name, _) as _gan_t20,) if isinstance(_gan_t20, tuple):
                return name
        raise GanMatchError("no clause of _gan_fn2/1 matched " + repr(_gan_args))
    offenders = gandora_std.enum.uniq(gandora_std.enum.map(re.compile("(?m)^\\s*@spec\\s+([a-z_][A-Za-z0-9_]*[?!]?)\\(([^)]*(?:list|map)\\()").findall(source), _gan_fn2))
    if _gan_truthy(gandora_std.enum.empty_p(offenders)):
        return []
    else:
        _gan_fstr21 = gandora_std.enum.join(offenders, ", ")
        return [{"kind": "practice", "line": 0, "message": f"Concrete `list()`/`map()` in parameter position of @spec {_gan_fstr21} — accept `sequence(t)`/`iterable(t)`/`mapping(k, v)`, return concrete (\"abstract in, concrete out\", docs/syntax.md)."}]


def _idiom_hints(source):
    checks = [(re.compile("fn (\\w+) -> ([a-z_][A-Za-z0-9_.]*[?!]?)\\(\\1\\) end"), "`fn x -> f(x) end` wraps a single call — the capture `&f/1` says the same thing."), (re.compile("Enum\\.count\\([^)]*\\)\\s*==\\s*0|length\\([^)]*\\)\\s*==\\s*0"), "`... == 0` on a count — `Enum.empty?(xs)` reads better."), (re.compile("\\|>\\s*Enum\\.map\\([\\s\\S]{0,120}?\\|>\\s*Enum\\.filter\\(|\\|>\\s*Enum\\.filter\\([\\s\\S]{0,120}?\\|>\\s*Enum\\.map\\("), "A map+filter pipeline — a single `for x <- xs, cond, do: expr` comprehension often reads better and compiles to one pass (GEP-0020)."), (re.compile(", do: true, else: false"), "`if cond, do: true, else: false` is just the condition (wrap with a boolean-shaped guard if needed).")]
    def _gan_fn3(*_gan_args, source=source):
        match _gan_args:
            case ((pattern, advice) as _gan_t22,) if isinstance(_gan_t22, tuple):
                if (pattern.search(source) is None):
                    return []
                else:
                    return [{"kind": "practice", "line": 0, "message": advice}]
        raise GanMatchError("no clause of _gan_fn3/1 matched " + repr(_gan_args))
    hits = gandora_std.enum.flat_map(checks, _gan_fn3)
    rescue_only_bare = _gan_and(gandora_std.string.match_p(source, re.compile("rescue\\s*\\n\\s*[a-z_][A-Za-z0-9_]* ->")), lambda: not (_gan_truthy(gandora_std.string.match_p(source, re.compile("rescue[\\s\\S]{0,200}? in ")))))
    if _gan_truthy(rescue_only_bare):
        _gan_tmp23 = [{"kind": "practice", "line": 0, "message": "A bare `rescue e ->` catches every Exception — rescue specific types first: `e in $builtins.ValueError -> ...` (GEP-0014)."}]
    else:
        _gan_tmp23 = []
    bare_hint = _gan_tmp23
    return hits + bare_hint


def _pyimport_hints(source):
    refs = re.compile("\\$([a-z_][a-z0-9_]*)").findall(source)
    counts = gandora_std.enum.reduce(refs, {}, lambda m, acc: gandora_std.map.put(acc, m, gandora_std.map.get(acc, m, 0) + 1))
    def _gan_fn4(*_gan_args):
        match _gan_args:
            case ((_, n) as _gan_t24,) if isinstance(_gan_t24, tuple):
                return n >= 3
        raise GanMatchError("no clause of _gan_fn4/1 matched " + repr(_gan_args))
    def _gan_fn5(*_gan_args):
        match _gan_args:
            case ((m, n) as _gan_t25,) if isinstance(_gan_t25, tuple):
                return {"kind": "practice", "line": 0, "message": f"`${m}` appears {n} times — declare `pyimport {m}` once and use the bare name (GEP-0003 rev 6)."}
        raise GanMatchError("no clause of _gan_fn5/1 matched " + repr(_gan_args))
    return gandora_std.enum.map(gandora_std.enum.filter(counts.items(), _gan_fn4), _gan_fn5)


def _error_suggestions(source, msg, line):
    from_msg = re.compile("'([A-Za-z_][A-Za-z0-9_.!?]*)'").findall(str(msg))
    if (line is None) or (line == 0):
        _gan_tmp26 = re.compile("[A-Za-z_][A-Za-z0-9_]*").findall(source)
    else:
        l = gandora_std.enum.at(source.split("\n"), line - 1)
        if (l is None):
            _gan_tmp26 = []
        else:
            _gan_tmp26 = re.compile("[A-Za-z_][A-Za-z0-9_]*").findall(l)
    at_line = _gan_tmp26
    def _gan_fn6(word, *, line=line):
        near = difflib.get_close_matches(word, keywords, n=1, cutoff=0.8)
        if _gan_truthy(_gan_or(gandora_std.enum.empty_p(near), lambda: gandora_std.enum.member_p(keywords, word))):
            return []
        else:
            return [{"kind": "did_you_mean", "line": line, "message": f"`{word}` — did you mean `{gandora_std.enum.at(near, 0)}`?"}]
    return gandora_std.enum.flat_map(gandora_std.enum.reject(gandora_std.enum.uniq(from_msg + at_line), lambda word, *, source=source: gandora_std.string.contains_p(source, "@" + word)), _gan_fn6)


def _member_suggestions(raw, root):
    source = _mask_literals(raw)
    calls = re.compile("\\b([A-Z][A-Za-z0-9_.]*)\\.([a-z_][A-Za-z0-9_]*[?!]?)\\(").findall(source)
    def _gan_fn7(*_gan_args, root=root):
        match _gan_args:
            case ((mod, fun) as _gan_t27,) if isinstance(_gan_t27, tuple):
                names = _module_functions(mod, root)
                if _gan_truthy(gandora_std.enum.empty_p(names)):
                    return []
                elif _gan_truthy(gandora_std.enum.member_p(names, fun)):
                    return []
                else:
                    near = difflib.get_close_matches(fun, names, n=3, cutoff=0.6)
                    if _gan_truthy(gandora_std.enum.empty_p(near)):
                        _gan_tmp28 = f"no close match; see `gan lsc symbols {mod}`"
                    else:
                        _gan_fstr29 = gandora_std.enum.join(gandora_std.enum.map(near, lambda n, *, mod=mod: f"`{mod}.{n}`"), " / ")
                        _gan_tmp28 = f"did you mean {_gan_fstr29}?"
                    hint = _gan_tmp28
                    return [{"kind": "did_you_mean", "line": 0, "message": f"`{mod}.{fun}` is not a function of {mod} — {hint}"}]
        raise GanMatchError("no clause of _gan_fn7/1 matched " + repr(_gan_args))
    return gandora_std.enum.flat_map(gandora_std.enum.uniq(calls), _gan_fn7)


def _module_functions(mod, root):
    try:
        _gan_tmp30 = core.symbols(mod, root)
    except Exception as _e:
        _gan_tmp30 = []
    syms = _gan_tmp30
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
            _gan_fstr31 = gandora_std.enum.join(gandora_std.enum.map(near, lambda n: f"`{n}`"), " / ")
            return [{"kind": "did_you_mean", "line": gandora_std.map.get(d, "line"), "message": f"`{word}` — did you mean {_gan_fstr31}?"}]


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
        _gan_tmp32 = python + "\n\nmain()\n"
    elif _gan_truthy(module_mode):
        _gan_tmp32 = python
    else:
        _gan_tmp32 = python + "\ntry:\n    print(\"__gan_value__\", repr(_))\nexcept NameError:\n    pass\n"
    code = _gan_tmp32
    pathlib.Path(path).write_text(code)
    try:
        _gan_tmp33 = ("ok", subprocess.run([_sandbox_python(root), path], capture_output=True, text=True, timeout=10, cwd=dir))
    except subprocess.TimeoutExpired as _e:
        _gan_tmp33 = "timeout"
    result = _gan_tmp33
    _gan_case34 = result
    match _gan_case34:
        case "timeout":
            return (None, None, "execution timed out after 10s")
        case ("ok", r) as _gan_t35 if isinstance(_gan_t35, tuple):
            _gan_val36 = _split_value(r.stdout)
            match _gan_val36:
                case (out, value) as _gan_t37 if isinstance(_gan_t37, tuple):
                    pass
                case _:
                    raise GanMatchError("no match of right-hand side value: " + repr(_gan_val36))
            if r.returncode == 0:
                return (out, value, None)
            else:
                return (out, value, _last_error_line(r.stderr))
        case _:
            raise GanMatchError("no case clause matched: " + repr(_gan_case34))


def _split_value(stdout):
    lines = stdout.split("\n")
    marked = gandora_std.enum.find(lines, lambda l: gandora_std.string.starts_with_p(l, "__gan_value__ "))
    plain = gandora_std.enum.join(gandora_std.enum.reject(lines, lambda l: gandora_std.string.starts_with_p(l, "__gan_value__ ")), "\n")
    if (marked is None):
        _gan_tmp38 = None
    else:
        _gan_tmp38 = gandora_std.string.replace(marked, "__gan_value__ ", "")
    value = _gan_tmp38
    if value == "None":
        _gan_tmp39 = None
    else:
        _gan_tmp39 = value
    value = _gan_tmp39
    return (plain, value)


def _last_error_line(stderr):
    return gandora_std.enum.at(gandora_std.enum.reject(stderr.strip().split("\n"), lambda l: l.strip() == ""), -1)
