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
from gandora_tool.safe import *


def _gan_truthy(value):
    return value is not None and value is not False

def _gan_and(value, then):
    return then() if _gan_truthy(value) else value

def _gan_or(value, then):
    return value if _gan_truthy(value) else then()

class GanMatchError(Exception):
    pass

mistakes = [(re.compile("^\\s*return\\b"), "Gandora has no `return` — a function's value is its last expression."), (re.compile("^\\s*while\\b"), "There is no `while` — use tail recursion (constant stack) or `for`/Enum (GEP-0019/0020)."), (re.compile("\\belif\\b"), "`elif` is spelled as another `cond do` branch, or a `case` clause."), (re.compile("^\\s*def .*\\):\\s*$"), "Python `def ...():` — a Gandora head has no colon: `def f(x) do ... end` or `def f(x), do: expr`."), (re.compile("\\blambda\\b"), "`lambda` is spelled `fn x -> ... end`, called with `f.(x)`."), (re.compile("\\bNone\\b"), "`None` is spelled `nil`."), (re.compile("\\bTrue\\b|\\bFalse\\b"), "Booleans are lowercase: `true` / `false` (only `false` and `nil` are falsy)."), (re.compile("^\\s*import\\s+[a-z]"), "Python `import x` is `pyimport x` at module top, or an inline `$x` reference (GEP-0003)."), (re.compile("^\\s*from\\s+\\w+\\s+import\\b"), "`from x import y` has no direct spelling — `pyimport x` then `x.y`, or `$x.y` inline."), (re.compile("\\bprint\\("), "Prefer `IO.puts(...)` (falls through to Python print, but IO.puts is the idiom)."), (re.compile("\\bself\\."), "There is no `self` — Gandora functions are module functions over plain data."), (re.compile("&&|\\|\\|(?!>)"), "Boolean operators are the words `and` / `or` / `not`."), (re.compile("\\+=|-=|\\*="), "No augmented assignment — rebind: `x = x + 1`."), (re.compile("\\bf\""), "f-strings are plain strings — interpolation is built in: \"total " + ("#" + "{n}\".")), (re.compile("\\bswitch\\b"), "`switch` is `case ... do pattern -> ... end`."), (re.compile("\\bnil\\s*==|==\\s*nil\\b"), "Prefer `is_nil(x)` over comparing with nil."), (re.compile("\\$\""), "Quoted module refs were retired: write $(a.b) (GEP-0003-R010)."), (re.compile("\\bPython\\b"), "There is no `Python` object — interop is `$math.sqrt(x)` inline, or `pyimport math` at module top, then `math.sqrt(x)` (GEP-0003)."), (re.compile("\\|>\\s*then\\("), "Elixir's `then/2` does not exist — pipe into an anonymous fn `x |> (fn v -> f(v, 1) end).()`, or just bind a variable."), (re.compile("~python\\("), "`~python(...)` is plain TEXT tagged python (GEP-0009 rev 5) — executable Python is `$python(expr)`; if you truly meant a text template, ignore this."), (re.compile("~json[(\"]"), "`~json` is plain TEXT tagged json — data lives in Gandora maps: %{\"k\" => v} (atom keys: %{k: v}); parse runtime JSON with $json.loads(s)."), (re.compile("^\\s*\"[A-Za-z_]+\":\\s"), "That looks like a JSON object line — a Gandora map spells it %{\"key\" => value} (atom keys: %{key: value}); lists keep [ ].")]

elixir_reflexes = {"Integer": "no `Integer` module — `to_string/1`, `rem/2`, `div/2` are built-ins; parse with `$builtins.int(s)`", "Float": "no `Float` module — `$builtins.float(s)` parses, `$builtins.round(x, n)` rounds", "Kernel": "no `Kernel` module — its functions (to_string, inspect, rem, div, elem...) are bare built-ins", "Tuple": "no `Tuple` module — pattern matching and `elem/2` cover tuples", "Atom": "no `Atom` module — `to_string/1` renders an atom", "Range": "no `Range` module — `1..n` literals feed `for`/Enum directly", "Regex": "no `Regex` module — `~r/.../ ` sigils plus `$re` (Python re) methods", "Stream": "no `Stream` module — `for` comprehensions and Enum are eager; reach for `$itertools` when you need laziness", "Process": "no BEAM processes — this compiles to plain Python; use functions and data", "GenServer": "no BEAM — no GenServer; model state as data passed through functions", "Agent": "no BEAM — no Agent; carry state explicitly"}

keywords = ["defmodule", "def", "defp", "defmacro", "defstruct", "case", "cond", "with", "for", "fn", "try", "rescue", "after", "else", "end", "do", "quote", "unquote", "pyimport", "import", "require", "alias", "use", "recur", "when", "true", "false", "nil", "and", "or", "not", "raise", "if", "unless"]


def analyze(source: str, root: str) -> list[dict]:
    """Every suggestion for one source: cross-language migration hints,
practice gaps, and member/name did-you-means.

## Parameters

  - source: The Gandora source text.
  - root: Project root for symbol resolution.
"""
    return _dedupe(_common_mistakes(source) + (_reflex_hints(source, root) + (_exception_hints(source) + (_struct_update_hints(source) + (_practice_hints(source) + _member_suggestions(source, root))))))


def error_hints(source: str, msg: str, line: int) -> list[dict]:
    """Fuzzy candidates for a compile error at `line` of `source`.

## Parameters

  - source: The source that failed to compile.
  - msg: The compiler error message.
  - line: The error line (0 when unknown).
"""
    return _dedupe(_error_suggestions(source, msg, line))


def _hint(kind, line, message):
    return {"kind": kind, "line": line, "message": message}


def _mask_literals(source: str) -> str:
    spaces = lambda text: re.sub("\\S", " ", text)
    masked = re.sub("\"\"\"([\\s\\S]*?)\"\"\"", lambda m, *, spaces=spaces: "\"\"\"" + (spaces(m.group(1)) + "\"\"\""), source)
    masked = _mask_paren_sigils(masked)
    masked = re.sub("(~[a-zA-Z]+/)([^/\\n]*)/", lambda m, *, spaces=spaces: m.group(1) + (spaces(m.group(2)) + "/"), masked)
    masked = re.sub("\"((?:[^\"\\\\\\n]|\\\\.)*)\"", lambda m, *, spaces=spaces: "\"" + (spaces(m.group(1)) + "\""), masked)
    return re.sub("#(?!\\{)([^\\n]*)", lambda m, *, spaces=spaces: "#" + spaces(m.group(1)), masked)


def _mask_paren_sigils(source):
    starts = gandora_std.enum.map(builtins.list(re.finditer("(?:~[a-zA-Z]+|\\$python)\\(", source)), lambda m: ((m).end()))
    def _gan_fn0(start, acc):
        stop = _balance_from(acc, start, 1)
        return _cut(acc, 0, start) + (re.sub("\\S", " ", _cut(acc, start, stop)) + _cut(acc, stop, builtins.len(acc)))
    return gandora_std.enum.reduce(starts, source, _gan_fn0)


def _cut(s, a, b):
    return gandora_std.string.slice(s, a, b - a)


def _char_at(s, i):
    return gandora_std.string.at(s, i)


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
                    return [_hint("migration", hit + 1, advice)]
        raise GanMatchError("no clause of _gan_fn1/1 matched " + repr(_gan_args))
    return gandora_std.enum.flat_map(mistakes, _gan_fn1)


def _practice_hints(raw):
    source = _mask_literals(raw)
    in_module = gandora_std.string.match_p(source, re.compile("defmodule"))
    return _coverage_hints(raw, source, in_module) + (_spec_container_hints(source) + (_lone_typevar_hints(source) + (_doc_example_hints(raw) + (_idiom_hints(source) + _pyimport_hints(source)))))


def _coverage_hints(raw, source, in_module):
    test_module = _gan_or(gandora_std.string.match_p(source, re.compile("(?m)^\\s*use Test\\b")), lambda: gandora_std.string.match_p(source, re.compile("defmodule Test[A-Z]")))
    if _gan_truthy(_gan_or(not (_gan_truthy(in_module)), lambda: test_module)):
        return []
    else:
        heads = gandora_std.enum.filter(gandora_std.enum.uniq(re.compile("(?m)^\\s*(?:async )?def ([a-z_][A-Za-z0-9_]*[?!]?)").findall(source)), lambda h: h != "unquote")
        specs = re.compile("(?m)^\\s*@spec ([a-z_][A-Za-z0-9_]*[?!]?)").findall(source)
        docs = _count_attr_blocks(source, "@doc")
        no_spec = gandora_std.enum.filter(heads, lambda h, *, specs=specs: not (_gan_truthy(gandora_std.enum.member_p(specs, h))))
        _gan_fstr1 = gandora_std.enum.join(no_spec, ", ")
        gaps = [(not (_gan_truthy(gandora_std.enum.empty_p(no_spec))), f"@spec on: {_gan_fstr1}"), (docs < gandora_std.enum.count(heads), f"@doc on {gandora_std.enum.count(heads) - docs} public def(s)"), (not (_gan_truthy(gandora_std.string.contains_p(source, "@moduledoc"))), "@moduledoc")]
        _gan_tmp2 = [label for _gan_for3 in gaps if isinstance(_gan_for3, tuple) and len(_gan_for3) == 2 for (gap, label,) in [(_gan_for3[0], _gan_for3[1],)] if _gan_truthy(gap)]
        missing = _gan_tmp2
        script_only = heads == ["main"]
        if ((gandora_std.enum.count(heads) > 0) and not (_gan_truthy(script_only))) and not (_gan_truthy(gandora_std.string.contains_p(raw, "@example"))):
            subject = gandora_std.enum.at(gandora_std.enum.filter(heads, lambda h: h != "main") + heads, 0)
            _gan_tmp4 = [_hint("practice", _line_of(source, re.compile("(?m)^\\s*(?:async )?def " + re.escape(subject))), f"No @example doctests — add one right above `def {subject}`:\n@example \"\"\"\n    gan> {subject}(...)\n    expected_value\n\"\"\"\n(fill in real arguments; expected output is the Python repr — {{:ok, 21}} prints as ('ok', 21), \"hi\" as 'hi'; `gan test` runs it) (GEP-0007).")]
        else:
            _gan_tmp4 = []
        example_hint = _gan_tmp4
        if _gan_truthy(gandora_std.enum.empty_p(missing)):
            _gan_tmp5 = []
        else:
            if _gan_truthy(gandora_std.enum.empty_p(no_spec)):
                _gan_tmp6 = re.compile("(?m)^\\s*(?:async )?def ")
            else:
                _gan_tmp6 = re.compile("(?m)^\\s*(?:async )?def " + re.escape(gandora_std.enum.at(no_spec, 0)))
            first = _gan_tmp6
            _gan_fstr7 = gandora_std.enum.join(missing, "; ")
            _gan_tmp5 = [_hint("practice", _line_of(source, first), f"Annotation coverage: missing {_gan_fstr7} — e.g. @doc \"What it does.\" then @spec name(integer()) :: integer() above the def; a printing entry is @spec main() :: nil; unsure of a type? read the cheat sheet: gan lsc doc spec (docs/syntax.md).")]
        head_hint = _gan_tmp5
        return head_hint + example_hint


def _count_attr_blocks(source, attr):
    return gandora_std.enum.count(re.compile(f"(?m)^\\s*{attr} ").findall(source))


def _doc_example_hints(raw):
    bodies = re.compile("@(?:module)?doc(?:_trans[^\"]*)?\\s+\"{3}([\\s\\S]*?)\"{3}").findall(raw)
    smells = gandora_std.enum.filter(bodies, lambda b: _gan_or(_gan_or(gandora_std.string.match_p(b, re.compile("(?mi)^\\s*(##\\s*)?examples?:?\\s*$")), lambda: gandora_std.string.contains_p(b, "iex>")), lambda: gandora_std.string.contains_p(b, "gan>")))
    if _gan_truthy(gandora_std.enum.empty_p(smells)):
        return []
    else:
        return [_hint("practice", _line_of(raw, re.compile("@doc\\s+\"{3}")), "an example lives in @example, not prose inside @doc — move it to:\n@example \"\"\"\n    gan> call(...)\n    expected_repr\n\"\"\"\n(runnable by `gan test` on defs; displayed in docs on defmacro). Keep @doc to what/why prose (GEP-0007).")]


def _lone_typevar_hints(source):
    specs = re.compile("(?m)^\\s*@spec\\s+([a-z_][A-Za-z0-9_]*[?!]?)\\((.*?)::\\s*([a-z]{1,2})\\s*$").findall(source)
    def _gan_fn2(*_gan_args, source=source):
        match _gan_args:
            case ((name, params, ret) as _gan_t8,) if isinstance(_gan_t8, tuple):
                seen = re.compile(f"(?<![A-Za-z0-9_$.]){ret}(?![A-Za-z0-9_(])").findall(params)
                if _gan_truthy(gandora_std.enum.empty_p(seen)):
                    return [_hint("practice", _line_of(source, re.compile("(?m)^\\s*@spec\\s+" + re.escape(name))), f"the return of @spec {name} is the lone type variable `{ret}` — it constrains nothing; write term(), or use the variable in a parameter too (GEP-0027).")]
                else:
                    return []
        raise GanMatchError("no clause of _gan_fn2/1 matched " + repr(_gan_args))
    return gandora_std.enum.flat_map(specs, _gan_fn2)


def _spec_container_hints(source):
    def _gan_fn3(*_gan_args):
        match _gan_args:
            case ((name, _) as _gan_t9,) if isinstance(_gan_t9, tuple):
                return name
        raise GanMatchError("no clause of _gan_fn3/1 matched " + repr(_gan_args))
    offenders = gandora_std.enum.uniq(gandora_std.enum.map(re.compile("(?m)^\\s*@spec\\s+([a-z_][A-Za-z0-9_]*[?!]?)\\(([^)]*(?:list|map)\\()").findall(source), _gan_fn3))
    if _gan_truthy(gandora_std.enum.empty_p(offenders)):
        return []
    else:
        first = re.compile("(?m)^\\s*@spec\\s+" + re.escape(gandora_std.enum.at(offenders, 0)))
        _gan_fstr10 = gandora_std.enum.join(offenders, ", ")
        return [_hint("practice", _line_of(source, first), f"Concrete `list()`/`map()` in parameter position of @spec {_gan_fstr10} — accept `sequence(t)`/`iterable(t)`/`mapping(k, v)`, return concrete (\"abstract in, concrete out\", docs/syntax.md).")]


def _idiom_hints(source):
    checks = [(re.compile("fn (\\w+) -> ((?:[A-Z][A-Za-z0-9_.]*\\.)?[a-z_][A-Za-z0-9_]*[?!]?)\\(\\1\\) end"), "`fn x -> f(x) end` wraps a single call — the capture `&f/1` says the same thing."), (re.compile("Enum\\.count\\([^)]*\\)\\s*==\\s*0|length\\([^)]*\\)\\s*==\\s*0"), "`... == 0` on a count — `Enum.empty?(xs)` reads better."), (re.compile("\\|>\\s*Enum\\.filter\\((?:(?!Enum\\.)[\\s\\S]){0,120}?\\|>\\s*Enum\\.map\\("), "A filter-then-map pipeline fuses: `for x <- xs, cond, do: expr` is one pass and one line (GEP-0020). (map-then-filter has no `for` spelling — keep piping those.)"), (re.compile(", do: true, else: false"), "`if cond, do: true, else: false` is just the condition (wrap with a boolean-shaped guard if needed).")]
    def _gan_fn4(*_gan_args, source=source):
        match _gan_args:
            case ((pattern, advice) as _gan_t11,) if isinstance(_gan_t11, tuple):
                if (pattern.search(source) is None):
                    return []
                else:
                    return [_hint("practice", _line_of(source, pattern), advice)]
        raise GanMatchError("no clause of _gan_fn4/1 matched " + repr(_gan_args))
    hits = gandora_std.enum.flat_map(checks, _gan_fn4)
    bare_pat = re.compile("rescue\\s*\\n\\s*[a-z][A-Za-z0-9_]* ->")
    rescue_only_bare = _gan_and(gandora_std.string.match_p(source, bare_pat), lambda: not (_gan_truthy(gandora_std.string.match_p(source, re.compile("rescue[\\s\\S]{0,200}? in ")))))
    if _gan_truthy(rescue_only_bare):
        _gan_tmp12 = [_hint("practice", _line_of(source, bare_pat), "A bare `rescue e ->` catches every Exception — rescue specific types first: `e in $builtins.ValueError -> ...` (GEP-0014).")]
    else:
        _gan_tmp12 = []
    bare_hint = _gan_tmp12
    return hits + bare_hint


def _pyimport_hints(source):
    value_source = gandora_std.enum.join(gandora_std.enum.filter(source.split("\n"), lambda l: not (_gan_truthy(gandora_std.string.match_p(l, re.compile("^\\s*@spec "))))), "\n")
    refs = re.compile("\\$([a-z_][a-z0-9_]*)").findall(value_source)
    counts = gandora_std.enum.reduce(refs, {}, lambda m, acc: gandora_std.map.put(acc, m, gandora_std.map.get(acc, m, 0) + 1))
    return [_hint("practice", _line_of(source, re.compile("\\$" + m)), f"`${m}` appears {n} times — declare `pyimport {m}` once and use the bare name; a dotted chain imports as `pyimport {m}.sub, as: s` (GEP-0003 rev 6).") for _gan_for13 in gandora_std.map.to_list(counts) if isinstance(_gan_for13, tuple) and len(_gan_for13) == 2 for (m, n,) in [(_gan_for13[0], _gan_for13[1],)] if (n >= 3) and not (_gan_truthy(gandora_std.enum.member_p(["builtins", "python"], m)))]


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
    def _gan_fn5(word, *, line=line):
        near = difflib.get_close_matches(word, keywords, n=1, cutoff=0.8)
        if _gan_truthy(_gan_or(gandora_std.enum.empty_p(near), lambda: gandora_std.enum.member_p(keywords, word))):
            return []
        else:
            return [_hint("did_you_mean", line, f"`{word}` — did you mean `{gandora_std.enum.at(near, 0)}`?")]
    return gandora_std.enum.flat_map(gandora_std.enum.reject(gandora_std.enum.uniq(from_msg + at_line), lambda word, *, source=source: gandora_std.string.contains_p(source, "@" + word)), _gan_fn5)


def _reflex_hints(raw, root):
    source = _mask_literals(raw)
    return [_hint("migration", _line_of(source, re.compile("\\b" + (mod + "\\.[a-z_]"))), f"`{mod}.` — {advice} (GEP-0003).") for _gan_for15 in gandora_std.map.to_list(elixir_reflexes) if isinstance(_gan_for15, tuple) and len(_gan_for15) == 2 for (mod, advice,) in [(_gan_for15[0], _gan_for15[1],)] if not ((re.compile("\\b" + (mod + "\\.[a-z_]")).search(source) is None)) if _gan_truthy(gandora_std.enum.empty_p(_module_functions(mod, root)))]


def _struct_update_hints(raw):
    source = _mask_literals(raw)
    pat = re.compile("%\\{[a-z_][A-Za-z0-9_]*\\s*\\|")
    if _gan_truthy(_gan_and(gandora_std.string.match_p(source, re.compile("defstruct")), lambda: gandora_std.string.match_p(source, pat))):
        return [_hint("migration", _line_of(source, pat), "`%{x | field: v}` updates a plain map — a struct value updates as `%Mod{x | field: v}` (compiles to a dataclass replace) (GEP-0009).")]
    else:
        return []


def _exception_hints(raw):
    source = _mask_literals(raw)
    pat = re.compile("(?<![.\\w])([A-Z][A-Za-z0-9]*(?:Error|Exception))\\b")
    hits = gandora_std.enum.uniq(pat.findall(source))
    if _gan_truthy(gandora_std.enum.empty_p(hits)):
        return []
    else:
        return [_hint("migration", _line_of(source, pat), f"Python exceptions are spelled through their module: `$builtins.{gandora_std.enum.at(hits, 0)}` (rescue e in $builtins.{gandora_std.enum.at(hits, 0)} -> ...) (GEP-0014).")]


def _member_suggestions(raw, root):
    source = re.sub("(?m)^\\s*@(spec|type|opaque)[^\\n]*", "", _mask_literals(raw))
    calls = re.compile("\\b([A-Z][A-Za-z0-9_.]*)\\.([a-z_][A-Za-z0-9_]*[?!]?)\\(").findall(source)
    def _gan_fn6(*_gan_args, root=root, source=source):
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
                    guess = _gan_tmp17
                    return [_hint("did_you_mean", _line_of(source, re.compile(re.escape(f"{mod}.{fun}("))), f"`{mod}.{fun}` is not a function of {mod} — {guess}")]
        raise GanMatchError("no clause of _gan_fn6/1 matched " + repr(_gan_args))
    return gandora_std.enum.flat_map(gandora_std.enum.uniq(calls), _gan_fn6)


def _module_functions(mod, root):
    try:
        _gan_tmp19 = core.symbols(mod, root)
    except Exception as _e__gan1:
        _gan_tmp19 = []
    names = gandora_std.enum.map(_gan_tmp19, lambda s: gandora_std.map.get(s, "name"))
    if _gan_truthy(gandora_std.enum.empty_p(names)):
        return _std_functions(mod)
    else:
        return names


def _std_functions(mod):
    known = {"Enum": "enum", "Map": "map", "List": "list", "Keyword": "keyword", "String": "string", "Test": "test", "Task": "task", "File": "file", "Path": "path", "System": "system"}
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
        idents = gandora_std.enum.uniq(re.compile("(?<![A-Za-z0-9_])[a-z_][A-Za-z0-9_]*").findall(source))
        near = gandora_std.enum.filter(difflib.get_close_matches(word, idents, n=3, cutoff=0.7), lambda n, *, word=word: n != word)
        if _gan_truthy(gandora_std.enum.empty_p(near)):
            return []
        else:
            _gan_fstr20 = gandora_std.enum.join(gandora_std.enum.map(near, lambda n: f"`{n}`"), " / ")
            return [_hint("did_you_mean", gandora_std.map.get(d, "line"), f"`{word}` — did you mean {_gan_fstr20}?")]


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
    counts = gandora_std.enum.frequencies(gandora_std.enum.map(suggestions, lambda s: gandora_std.map.get(s, "message")))
    def _gan_fn7(s, *, counts=counts):
        others = gandora_std.map.get(counts, gandora_std.map.get(s, "message"), 1) - 1
        if others > 0:
            _gan_fstr21 = gandora_std.map.get(s, "message")
            return gandora_std.map.put(s, "message", f"{_gan_fstr21} (also in {others} other file(s))")
        else:
            return s
    return gandora_std.enum.map(_dedupe(suggestions), _gan_fn7)


def _dedupe(suggestions: collections.abc.Sequence[collections.abc.Mapping]) -> list[dict]:
    def _gan_fn8(*_gan_args):
        match _gan_args:
            case (s, (acc, seen) as _gan_t22,) if isinstance(_gan_t22, tuple):
                msg = gandora_std.map.get(s, "message")
                if _gan_truthy(gandora_std.enum.member_p(seen, msg)):
                    return (acc, seen)
                else:
                    return (acc + [s], seen + [msg])
        raise GanMatchError("no clause of _gan_fn8/2 matched " + repr(_gan_args))
    _gan_val23 = gandora_std.enum.reduce(suggestions, ([], []), _gan_fn8)
    match _gan_val23:
        case (out, _) as _gan_t24 if isinstance(_gan_t24, tuple):
            pass
        case _:
            raise GanMatchError("no match of right-hand side value: " + repr(_gan_val23))
    return out
