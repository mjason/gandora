"""The compiler's teaching voice (GEP-0025): practice, migration, and
did-you-mean suggestions over source text. `gan check` and
`gan lsc check` share this engine — core diagnostics say what is
broken, the Advisor says what to write instead.
"""

import builtins
import collections.abc
import difflib
import gandora_core as core
import importlib
import re
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

mistakes = [(re.compile("^\\s*return\\b"), "Gandora has no `return` — a function's value is its last expression."), (re.compile("^\\s*while\\b"), "There is no `while` — use tail recursion (constant stack) or `for`/Enum (GEP-0019/0020)."), (re.compile("\\belif\\b"), "`elif` is spelled as another `cond do` branch, or a `case` clause."), (re.compile("^\\s*def .*\\):\\s*$"), "Python `def ...():` — a Gandora head has no colon: `def f(x) do ... end` or `def f(x), do: expr`."), (re.compile("\\blambda\\b"), "`lambda` is spelled `fn x -> ... end`, called with `f.(x)`."), (re.compile("\\bNone\\b"), "`None` is spelled `nil`."), (re.compile("\\bTrue\\b|\\bFalse\\b"), "Booleans are lowercase: `true` / `false` (only `false` and `nil` are falsy)."), (re.compile("^\\s*import\\s+[a-z]"), "Python `import x` is `pyimport x` at module top, or an inline `$x` reference (GEP-0003)."), (re.compile("^\\s*from\\s+\\w+\\s+import\\b"), "`from x import y` has no direct spelling — `pyimport x` then `x.y`, or `$x.y` inline."), (re.compile("\\bprint\\("), "Prefer `IO.puts(...)` (falls through to Python print, but IO.puts is the idiom)."), (re.compile("\\bself\\."), "There is no `self` — Gandora functions are module functions over plain data."), (re.compile("&&|\\|\\|(?!>)"), "Boolean operators are the words `and` / `or` / `not`."), (re.compile("\\+=|-=|\\*="), "No augmented assignment — rebind: `x = x + 1`."), (re.compile("\\bf\""), "f-strings are plain strings — interpolation is built in: \"total " + ("#" + "{n}\".")), (re.compile("\\bswitch\\b"), "`switch` is `case ... do pattern -> ... end`."), (re.compile("\\bnil\\s*==|==\\s*nil\\b"), "Prefer `is_nil(x)` over comparing with nil."), (re.compile("\\$\""), "Quoted module refs were retired: write $(a.b) (GEP-0003-R010)."), (re.compile("\\bPython\\b"), "There is no `Python` object — interop is `$math.sqrt(x)` inline, or `pyimport math` at module top, then `math.sqrt(x)` (GEP-0003).")]

elixir_reflexes = {"Integer": "no `Integer` module — `to_string/1`, `rem/2`, `div/2` are built-ins; parse with `$builtins.int(s)`", "Float": "no `Float` module — `$builtins.float(s)` parses, `$builtins.round(x, n)` rounds", "Kernel": "no `Kernel` module — its functions (to_string, inspect, rem, div, elem...) are bare built-ins", "Tuple": "no `Tuple` module — pattern matching and `elem/2` cover tuples", "Atom": "no `Atom` module — `to_string/1` renders an atom", "Range": "no `Range` module — `1..n` literals feed `for`/Enum directly", "Regex": "no `Regex` module — `~r/.../ ` sigils plus `$re` (Python re) methods", "Stream": "no `Stream` module — `for` comprehensions and Enum are eager; reach for `$itertools` when you need laziness", "Process": "no BEAM processes — this compiles to plain Python; use functions and data", "GenServer": "no BEAM — no GenServer; model state as data passed through functions", "Agent": "no BEAM — no Agent; carry state explicitly", "Task": "no BEAM — no Task; call functions directly or use Python concurrency via $-interop", "File": "no `File` module — `$pathlib.Path(p).read_text()` and friends", "Path": "no `Path` module — `$os.path` or `$pathlib`", "System": "no `System` module — `$sys` / `$os` cover arguments and environment"}

keywords = ["defmodule", "def", "defp", "defmacro", "defstruct", "case", "cond", "with", "for", "fn", "try", "rescue", "after", "else", "end", "do", "quote", "unquote", "pyimport", "import", "require", "alias", "use", "recur", "when", "true", "false", "nil", "and", "or", "not", "raise", "if", "unless"]


def analyze(source: str, root: str) -> list[dict]:
    """Every suggestion for one source: cross-language migration hints,
practice gaps, and member/name did-you-means.

## Parameters

  - source: The Gandora source text.
  - root: Project root for symbol resolution.
"""
    return _dedupe(_common_mistakes(source) + (_reflex_hints(source) + (_exception_hints(source) + (_struct_update_hints(source) + (_practice_hints(source) + _member_suggestions(source, root))))))


def error_hints(source: str, msg: str, line: int) -> list[dict]:
    """Fuzzy candidates for a compile error at `line` of `source`.

## Parameters

  - source: The source that failed to compile.
  - msg: The compiler error message.
  - line: The error line (0 when unknown).
"""
    return _dedupe(_error_suggestions(source, msg, line))


def _mask_literals(source: str) -> str:
    spaces = lambda text: re.sub("\\S", " ", text)
    masked = re.sub("\"\"\"([\\s\\S]*?)\"\"\"", lambda m, *, spaces=spaces: "\"\"\"" + (spaces(m.group(1)) + "\"\"\""), source)
    masked = _mask_paren_sigils(masked)
    masked = re.sub("(~[a-zA-Z]+/)([^/\\n]*)/", lambda m, *, spaces=spaces: m.group(1) + (spaces(m.group(2)) + "/"), masked)
    masked = re.sub("\"((?:[^\"\\\\\\n]|\\\\.)*)\"", lambda m, *, spaces=spaces: "\"" + (spaces(m.group(1)) + "\""), masked)
    return re.sub("#(?!\\{)([^\\n]*)", lambda m, *, spaces=spaces: "#" + spaces(m.group(1)), masked)


def _mask_paren_sigils(source):
    starts = gandora_std.enum.map(builtins.list(re.finditer("~[a-zA-Z]+\\(", source)), lambda m: ((m).end()))
    def _gan_fn0(start, acc):
        stop = _balance_from(acc, start, 1)
        return _cut(acc, 0, start) + (re.sub("\\S", " ", _cut(acc, start, stop)) + _cut(acc, stop, builtins.len(acc)))
    return gandora_std.enum.reduce(starts, source, _gan_fn0)


def _cut(s, a, b):
    return ((s)[(a):(b)])


def _char_at(s, i):
    return ((s)[(i)])


def _balance_from(source, i, depth):
    while True:
        if (i >= builtins.len(source)) or (depth == 0):
            return i - 1
        elif _char_at(source, i) == "(":
            source, i, depth = source, i + 1, depth + 1
            continue
        elif _char_at(source, i) == ")":
            source, i, depth = source, i + 1, depth - 1
            continue
        else:
            source, i, depth = source, i + 1, depth
            continue


def _line_of(source, pattern):
    m = pattern.search(source)
    if (m is None):
        return 0
    else:
        return source.count("\n", 0, m.start()) + 1


def _common_mistakes(source):
    lines = _mask_literals(source).split("\n")
    def _gan_fn1(*_gan_args, lines=lines):
        match _gan_args:
            case ((pattern, advice) as _gan_t0,) if isinstance(_gan_t0, tuple):
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
    test_module = _gan_or(gandora_std.string.match_p(source, re.compile("(?m)^\\s*use Test\\b")), lambda: gandora_std.string.match_p(source, re.compile("defmodule Test[A-Z]")))
    if _gan_truthy(_gan_or(not (_gan_truthy(in_module)), lambda: test_module)):
        return []
    else:
        heads = gandora_std.enum.filter(gandora_std.enum.uniq(re.compile("(?m)^\\s*def ([a-z_][A-Za-z0-9_]*[?!]?)").findall(source)), lambda h: h != "unquote")
        specs = re.compile("(?m)^\\s*@spec ([a-z_][A-Za-z0-9_]*[?!]?)").findall(source)
        docs = _count_attr_blocks(source, "@doc")
        no_spec = gandora_std.enum.filter(heads, lambda h, *, specs=specs: not (_gan_truthy(gandora_std.enum.member_p(specs, h))))
        missing = []
        if _gan_truthy(gandora_std.enum.empty_p(no_spec)):
            _gan_tmp1 = missing
        else:
            _gan_fstr2 = gandora_std.enum.join(no_spec, ", ")
            _gan_tmp1 = missing + [f"@spec on: {_gan_fstr2}"]
        missing = _gan_tmp1
        if docs >= gandora_std.enum.count(heads):
            _gan_tmp3 = missing
        else:
            _gan_tmp3 = missing + [f"@doc on {gandora_std.enum.count(heads) - docs} public def(s)"]
        missing = _gan_tmp3
        if _gan_truthy(gandora_std.string.contains_p(source, "@moduledoc")):
            _gan_tmp4 = missing
        else:
            _gan_tmp4 = missing + ["@moduledoc"]
        missing = _gan_tmp4
        script_only = heads == ["main"]
        if (gandora_std.enum.count(heads) > 0) and not (_gan_truthy(script_only)) and not (_gan_truthy(gandora_std.string.contains_p(raw, "@example"))):
            _gan_tmp5 = [{"kind": "practice", "line": _line_of(source, re.compile("(?m)^\\s*def ")), "message": "No @example doctests — add one right above a def, e.g.:\n@example \"\"\"\n    gan> double(21)\n    42\n\"\"\"\n(`gan test` runs them; expected output is the Python repr) (GEP-0007)."}]
        else:
            _gan_tmp5 = []
        example_hint = _gan_tmp5
        if _gan_truthy(gandora_std.enum.empty_p(missing)):
            _gan_tmp6 = []
        else:
            if _gan_truthy(gandora_std.enum.empty_p(no_spec)):
                _gan_tmp7 = re.compile("(?m)^\\s*def ")
            else:
                _gan_tmp7 = re.compile("(?m)^\\s*def " + re.escape(gandora_std.enum.at(no_spec, 0)))
            first = _gan_tmp7
            _gan_fstr8 = gandora_std.enum.join(missing, "; ")
            _gan_tmp6 = [{"kind": "practice", "line": _line_of(source, first), "message": f"Annotation coverage: missing {_gan_fstr8} — e.g. @doc \"What it does.\" then @spec name(integer()) :: integer() above the def; a printing entry is @spec main() :: nil; unsure of a type? read the cheat sheet: gan lsc doc spec (docs/syntax.md)."}]
        head_hint = _gan_tmp6
        return head_hint + example_hint


def _count_attr_blocks(source, attr):
    return gandora_std.enum.count(re.compile(f"(?m)^\\s*{attr} ").findall(source))


def _spec_container_hints(source):
    def _gan_fn2(*_gan_args):
        match _gan_args:
            case ((name, _) as _gan_t9,) if isinstance(_gan_t9, tuple):
                return name
        raise GanMatchError("no clause of _gan_fn2/1 matched " + repr(_gan_args))
    offenders = gandora_std.enum.uniq(gandora_std.enum.map(re.compile("(?m)^\\s*@spec\\s+([a-z_][A-Za-z0-9_]*[?!]?)\\(([^)]*(?:list|map)\\()").findall(source), _gan_fn2))
    if _gan_truthy(gandora_std.enum.empty_p(offenders)):
        return []
    else:
        first = re.compile("(?m)^\\s*@spec\\s+" + re.escape(gandora_std.enum.at(offenders, 0)))
        _gan_fstr10 = gandora_std.enum.join(offenders, ", ")
        return [{"kind": "practice", "line": _line_of(source, first), "message": f"Concrete `list()`/`map()` in parameter position of @spec {_gan_fstr10} — accept `sequence(t)`/`iterable(t)`/`mapping(k, v)`, return concrete (\"abstract in, concrete out\", docs/syntax.md)."}]


def _idiom_hints(source):
    checks = [(re.compile("fn (\\w+) -> ((?:[A-Z][A-Za-z0-9_.]*\\.)?[a-z_][A-Za-z0-9_]*[?!]?)\\(\\1\\) end"), "`fn x -> f(x) end` wraps a single call — the capture `&f/1` says the same thing."), (re.compile("Enum\\.count\\([^)]*\\)\\s*==\\s*0|length\\([^)]*\\)\\s*==\\s*0"), "`... == 0` on a count — `Enum.empty?(xs)` reads better."), (re.compile("\\|>\\s*Enum\\.map\\([\\s\\S]{0,120}?\\|>\\s*Enum\\.filter\\(|\\|>\\s*Enum\\.filter\\([\\s\\S]{0,120}?\\|>\\s*Enum\\.map\\("), "A map+filter pipeline — a single `for x <- xs, cond, do: expr` comprehension often reads better and compiles to one pass (GEP-0020)."), (re.compile(", do: true, else: false"), "`if cond, do: true, else: false` is just the condition (wrap with a boolean-shaped guard if needed).")]
    def _gan_fn3(*_gan_args, source=source):
        match _gan_args:
            case ((pattern, advice) as _gan_t11,) if isinstance(_gan_t11, tuple):
                if (pattern.search(source) is None):
                    return []
                else:
                    return [{"kind": "practice", "line": _line_of(source, pattern), "message": advice}]
        raise GanMatchError("no clause of _gan_fn3/1 matched " + repr(_gan_args))
    hits = gandora_std.enum.flat_map(checks, _gan_fn3)
    bare_pat = re.compile("rescue\\s*\\n\\s*[a-z][A-Za-z0-9_]* ->")
    rescue_only_bare = _gan_and(gandora_std.string.match_p(source, bare_pat), lambda: not (_gan_truthy(gandora_std.string.match_p(source, re.compile("rescue[\\s\\S]{0,200}? in ")))))
    if _gan_truthy(rescue_only_bare):
        _gan_tmp12 = [{"kind": "practice", "line": _line_of(source, bare_pat), "message": "A bare `rescue e ->` catches every Exception — rescue specific types first: `e in $builtins.ValueError -> ...` (GEP-0014)."}]
    else:
        _gan_tmp12 = []
    bare_hint = _gan_tmp12
    return hits + bare_hint


def _pyimport_hints(source):
    value_source = gandora_std.enum.join(gandora_std.enum.filter(source.split("\n"), lambda l: not (_gan_truthy(gandora_std.string.match_p(l, re.compile("^\\s*@spec "))))), "\n")
    refs = re.compile("\\$([a-z_][a-z0-9_]*)").findall(value_source)
    counts = gandora_std.enum.reduce(refs, {}, lambda m, acc: gandora_std.map.put(acc, m, gandora_std.map.get(acc, m, 0) + 1))
    return [{"kind": "practice", "line": _line_of(source, re.compile("\\$" + m)), "message": f"`${m}` appears {n} times — declare `pyimport {m}` once and use the bare name; a dotted chain imports as `pyimport {m}.sub, as: s` (GEP-0003 rev 6)."} for _gan_for13 in counts.items() if isinstance(_gan_for13, tuple) and len(_gan_for13) == 2 for (m, n,) in [(_gan_for13[0], _gan_for13[1],)] if (n >= 3) and (m != "builtins")]


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
    def _gan_fn4(word, *, line=line):
        near = difflib.get_close_matches(word, keywords, n=1, cutoff=0.8)
        if _gan_truthy(_gan_or(gandora_std.enum.empty_p(near), lambda: gandora_std.enum.member_p(keywords, word))):
            return []
        else:
            return [{"kind": "did_you_mean", "line": line, "message": f"`{word}` — did you mean `{gandora_std.enum.at(near, 0)}`?"}]
    return gandora_std.enum.flat_map(gandora_std.enum.reject(gandora_std.enum.uniq(from_msg + at_line), lambda word, *, source=source: gandora_std.string.contains_p(source, "@" + word)), _gan_fn4)


def _reflex_hints(raw):
    source = _mask_literals(raw)
    return [{"kind": "migration", "line": _line_of(source, re.compile("\\b" + (mod + "\\.[a-z_]"))), "message": f"`{mod}.` — {advice} (GEP-0003)."} for _gan_for15 in elixir_reflexes.items() if isinstance(_gan_for15, tuple) and len(_gan_for15) == 2 for (mod, advice,) in [(_gan_for15[0], _gan_for15[1],)] if not ((re.compile("\\b" + (mod + "\\.[a-z_]")).search(source) is None))]


def _struct_update_hints(raw):
    source = _mask_literals(raw)
    pat = re.compile("%\\{[a-z_][A-Za-z0-9_]*\\s*\\|")
    if _gan_truthy(_gan_and(gandora_std.string.match_p(source, re.compile("defstruct")), lambda: gandora_std.string.match_p(source, pat))):
        return [{"kind": "migration", "line": _line_of(source, pat), "message": "`%{x | field: v}` updates a plain map — a struct value updates as `%Mod{x | field: v}` (compiles to a dataclass replace) (GEP-0009)."}]
    else:
        return []


def _exception_hints(raw):
    source = _mask_literals(raw)
    pat = re.compile("(?<![.\\w])([A-Z][A-Za-z0-9]*(?:Error|Exception))\\b")
    hits = gandora_std.enum.uniq(pat.findall(source))
    if _gan_truthy(gandora_std.enum.empty_p(hits)):
        return []
    else:
        return [{"kind": "migration", "line": _line_of(source, pat), "message": f"Python exceptions are spelled through their module: `$builtins.{gandora_std.enum.at(hits, 0)}` (rescue e in $builtins.{gandora_std.enum.at(hits, 0)} -> ...) (GEP-0014)."}]


def _member_suggestions(raw, root):
    source = re.sub("(?m)^\\s*@(spec|type|opaque)[^\\n]*", "", _mask_literals(raw))
    calls = re.compile("\\b([A-Z][A-Za-z0-9_.]*)\\.([a-z_][A-Za-z0-9_]*[?!]?)\\(").findall(source)
    def _gan_fn5(*_gan_args, root=root, source=source):
        match _gan_args:
            case ((mod, fun) as _gan_t16,) if isinstance(_gan_t16, tuple):
                names = _module_functions(mod, root)
                if fun == "t":
                    return []
                elif _gan_truthy(gandora_std.enum.empty_p(names)):
                    return []
                elif _gan_truthy(gandora_std.enum.member_p(names, fun)):
                    return []
                else:
                    near = difflib.get_close_matches(fun, names, n=3, cutoff=0.6)
                    if _gan_truthy(gandora_std.enum.empty_p(near)):
                        _gan_tmp17 = f"no close match; see `gan lsc symbols {mod}`"
                    else:
                        _gan_fstr18 = gandora_std.enum.join(gandora_std.enum.map(near, lambda n, *, mod=mod: f"`{mod}.{n}`"), " / ")
                        _gan_tmp17 = f"did you mean {_gan_fstr18}?"
                    hint = _gan_tmp17
                    return [{"kind": "did_you_mean", "line": _line_of(source, re.compile(re.escape(f"{mod}.{fun}("))), "message": f"`{mod}.{fun}` is not a function of {mod} — {hint}"}]
        raise GanMatchError("no clause of _gan_fn5/1 matched " + repr(_gan_args))
    return gandora_std.enum.flat_map(gandora_std.enum.uniq(calls), _gan_fn5)


def _module_functions(mod, root):
    try:
        _gan_tmp19 = core.symbols(mod, root)
    except Exception as _e:
        _gan_tmp19 = []
    syms = _gan_tmp19
    names = gandora_std.enum.map(syms, lambda s: gandora_std.map.get(s, "name"))
    if _gan_truthy(gandora_std.enum.empty_p(names)):
        return _std_functions(mod)
    else:
        return names


def _std_functions(mod):
    known = {"Enum": "enum", "Map": "map", "List": "list", "Keyword": "keyword", "String": "string", "Test": "test"}
    pymod = gandora_std.map.get(known, mod)
    if (pymod is None):
        return []
    else:
        m = importlib.import_module("gandora_std." + pymod)
        return [gandora_std.string.replace(gandora_std.string.replace(n, "_bang", "!"), "_p", "?") for n in builtins.dir(m) if not (_gan_truthy(gandora_std.string.starts_with_p(n, "_")))]


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
            _gan_fstr20 = gandora_std.enum.join(gandora_std.enum.map(near, lambda n: f"`{n}`"), " / ")
            return [{"kind": "did_you_mean", "line": gandora_std.map.get(d, "line"), "message": f"`{word}` — did you mean {_gan_fstr20}?"}]


def lint_hints(source: str, diags: collections.abc.Sequence[collections.abc.Mapping]) -> list[dict]:
    """Did-you-mean hints for undefined-variable lints.

## Parameters

  - source: The source the lints came from.
  - diags: The diagnostics list from core.check/diagnostics.
"""
    return _dedupe(gandora_std.enum.flat_map(diags, lambda d, *, source=source: _lint_suggestions(source, d)))


def consolidate(suggestions: collections.abc.Sequence[collections.abc.Mapping]) -> list[dict]:
    """Project-level view: identical messages from many files collapse to
the first occurrence, annotated with the spread — 16 modules missing
doctests is one line, not sixteen.

## Parameters

  - suggestions: Per-file suggestion maps (may carry "path").
"""
    def _gan_fn6(s, acc):
        msg = gandora_std.map.get(s, "message")
        return gandora_std.map.put(acc, msg, gandora_std.map.get(acc, msg, 0) + 1)
    counts = gandora_std.enum.reduce(suggestions, {}, _gan_fn6)
    def _gan_fn7(*_gan_args, counts=counts):
        match _gan_args:
            case (s, (acc, seen) as _gan_t21,) if isinstance(_gan_t21, tuple):
                msg = gandora_std.map.get(s, "message")
                if _gan_truthy(gandora_std.enum.member_p(seen, msg)):
                    return (acc, seen)
                elif gandora_std.map.get(counts, msg, 1) > 1:
                    n = gandora_std.map.get(counts, msg) - 1
                    return (acc + [gandora_std.map.put(s, "message", f"{msg} (also in {n} other file(s))")], seen + [msg])
                else:
                    return (acc + [s], seen + [msg])
        raise GanMatchError("no clause of _gan_fn7/2 matched " + repr(_gan_args))
    _gan_val22 = gandora_std.enum.reduce(suggestions, ([], []), _gan_fn7)
    match _gan_val22:
        case (out, _) as _gan_t23 if isinstance(_gan_t23, tuple):
            pass
        case _:
            raise GanMatchError("no match of right-hand side value: " + repr(_gan_val22))
    return out


def _dedupe(suggestions: collections.abc.Sequence[collections.abc.Mapping]) -> list[dict]:
    def _gan_fn8(*_gan_args):
        match _gan_args:
            case (s, (acc, seen) as _gan_t24,) if isinstance(_gan_t24, tuple):
                msg = gandora_std.map.get(s, "message")
                if _gan_truthy(gandora_std.enum.member_p(seen, msg)):
                    return (acc, seen)
                else:
                    return (acc + [s], seen + [msg])
        raise GanMatchError("no clause of _gan_fn8/2 matched " + repr(_gan_args))
    _gan_val25 = gandora_std.enum.reduce(suggestions, ([], []), _gan_fn8)
    match _gan_val25:
        case (out, _) as _gan_t26 if isinstance(_gan_t26, tuple):
            pass
        case _:
            raise GanMatchError("no match of right-hand side value: " + repr(_gan_val25))
    return out
