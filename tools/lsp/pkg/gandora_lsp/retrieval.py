"""The three retrieval verbs (GEP-0031): `find` matches file names,
`grep` matches contents (regex, `rg`-accelerated with an
identical-shape fallback; or BM25-ranked for prose queries), and
`read` fetches one precise piece — a line range, a whole module, or
one definition's block located through the compiler's own line data.
"""

import builtins
import collections.abc
import fnmatch
import gandora_core as core
import math
import re
import gandora_lsp.atlas
import gandora_std.enum
import gandora_std.file
import gandora_std.map
import gandora_std.path
import gandora_std.string
import gandora_std.system
from gandora_tool.safe import *


def _gan_truthy(value):
    return value is not None and value is not False

def _gan_or(value, then):
    return value if _gan_truthy(value) else then()

def _gan_div(a, b):
    q = a // b
    if q < 0 and q * b != a:
        q += 1
    return q


def _gan_rem(a, b):
    return a - _gan_div(a, b) * b

class GanMatchError(Exception):
    pass

max_matches = 200

ranked_files = 10


def find(root: str, pattern: str, deps: bool) -> dict:
    """Corpus files whose *name* matches `pattern` — a glob when it carries
`*`/`?`, a substring otherwise. Sorted paths (GEP-0031-R004).

## Parameters

  - root: The project root.
  - pattern: The name pattern.
  - deps: Whether dependency sources join the search.
"""
    _gan_tmp0 = [gandora_lsp.atlas.relative(path, root) for path in gandora_lsp.atlas.corpus(root, deps) if _gan_truthy(_name_match_p(gandora_std.path.basename(path), pattern))]
    hits = _gan_tmp0
    return {"files": hits, "count": gandora_std.enum.count(hits)}


def _name_match_p(name, pattern):
    if _gan_truthy(_gan_or(gandora_std.string.contains_p(pattern, "*"), lambda: gandora_std.string.contains_p(pattern, "?"))):
        return fnmatch.fnmatch(name, pattern)
    else:
        return gandora_std.string.contains_p(gandora_std.string.downcase(name), gandora_std.string.downcase(pattern))


def grep(root: str, pattern: str, deps: bool) -> dict:
    """Content matches for a regular expression over the corpus:
`{path, line, text}` in path order, `rg` when present and a pure
scan otherwise — same shape either way, capped and saying so
(GEP-0031-R005).

## Parameters

  - root: The project root.
  - pattern: The regular expression.
  - deps: Whether dependency sources join the search.
"""
    files = gandora_lsp.atlas.corpus(root, deps)
    rg = gandora_std.system.find_executable("rg")
    if (rg is None):
        _gan_tmp1 = _scan(files, pattern)
    else:
        _gan_tmp1 = _rg_scan(rg, files, pattern)
    matches = _gan_tmp1
    kept = gandora_std.enum.take(matches, max_matches)
    def _gan_fn0(*_gan_args, root=root):
        match _gan_args:
            case ((path, line, text) as _gan_t2,) if isinstance(_gan_t2, tuple):
                return {"path": gandora_lsp.atlas.relative(path, root), "line": line, "text": text}
        raise GanMatchError("no clause of _gan_fn0/1 matched " + repr(_gan_args))
    return {"matches": gandora_std.enum.map(kept, _gan_fn0), "count": gandora_std.enum.count(kept), "truncated": gandora_std.enum.count(matches) > max_matches}


def _rg_scan(rg, files, pattern):
    _gan_val3 = gandora_std.system.cmd(rg, ["-n", "--no-heading", "-e", pattern, "--"] + files)
    match _gan_val3:
        case (out, _status) as _gan_t4 if isinstance(_gan_t4, tuple):
            pass
        case _:
            raise GanMatchError("no match of right-hand side value: " + repr(_gan_val3))
    lines = gandora_std.string.split_on(gandora_std.string.trim(out), "\n")
    return [_rg_hit(l) for l in lines if l != ""]


def _rg_hit(l):
    _gan_val5 = gandora_std.string.split_on(l, ":")
    match _gan_val5:
        case [path, line, *rest] as _gan_l6 if isinstance(_gan_l6, list):
            pass
        case _:
            raise GanMatchError("no match of right-hand side value: " + repr(_gan_val5))
    return (path, builtins.int(line), gandora_std.enum.join(rest, ":"))


def _scan(files, pattern):
    compiled = re.compile(pattern)
    def _gan_fn1(path, *, compiled=compiled):
        try:
            _gan_tmp7 = gandora_std.file.read_bang(path)
        except Exception as _e__gan1:
            _gan_tmp7 = ""
        lines = gandora_std.string.split_on(_gan_tmp7, "\n")
        return [(path, i + 1, l) for _gan_for8 in gandora_std.enum.with_index(lines) if isinstance(_gan_for8, tuple) and len(_gan_for8) == 2 for (l, i,) in [(_gan_for8[0], _gan_for8[1],)] if not ((compiled.search(l) is None))]
    return gandora_std.enum.flat_map(files, _gan_fn1)


def ranked(root: str, query: str, deps: bool) -> dict:
    """The BM25-ranked answer for a prose query (k1 = 1.2, b = 0.75, one
file per document): the top files, each with its best matching lines
and its score — deterministic, ties broken by path (GEP-0031-R005).

## Parameters

  - root: The project root.
  - query: The words to rank by.
  - deps: Whether dependency sources join the search.
"""
    terms = tokens(query)
    files = gandora_lsp.atlas.corpus(root, deps)
    def _gan_fn2(path):
        try:
            _gan_tmp10 = gandora_std.file.read_bang(path)
        except Exception as _e__gan2:
            _gan_tmp10 = ""
        return (path, _gan_tmp10)
    texts = gandora_std.enum.map(files, _gan_fn2)
    def _gan_fn3(*_gan_args):
        match _gan_args:
            case ((path, text) as _gan_t11,) if isinstance(_gan_t11, tuple):
                return (path, text, gandora_std.enum.frequencies(tokens(text)))
        raise GanMatchError("no clause of _gan_fn3/1 matched " + repr(_gan_args))
    freqs = gandora_std.enum.map(texts, _gan_fn3)
    n = gandora_std.enum.count(freqs)
    def _gan_fn4(*_gan_args):
        match _gan_args:
            case ((_p, _t, f) as _gan_t12,) if isinstance(_gan_t12, tuple):
                return _doc_len(f)
        raise GanMatchError("no clause of _gan_fn4/1 matched " + repr(_gan_args))
    avg = _average(gandora_std.enum.map(freqs, _gan_fn4))
    idfs = gandora_std.enum.map(terms, lambda t, *, freqs=freqs, n=n: (t, _idf(t, freqs, n)))
    def _gan_fn5(*_gan_args, avg=avg, idfs=idfs):
        match _gan_args:
            case ((path, text, f) as _gan_t13,) if isinstance(_gan_t13, tuple):
                return (_score(f, idfs, avg), path, text)
        raise GanMatchError("no clause of _gan_fn5/1 matched " + repr(_gan_args))
    def _gan_fn6(*_gan_args):
        match _gan_args:
            case ((s, _p, _t) as _gan_t14,) if isinstance(_gan_t14, tuple):
                return s > 0.0
        raise GanMatchError("no clause of _gan_fn6/1 matched " + repr(_gan_args))
    def _gan_fn7(*_gan_args):
        match _gan_args:
            case ((s, path, _t) as _gan_t15,) if isinstance(_gan_t15, tuple):
                return (-(s), path)
        raise GanMatchError("no clause of _gan_fn7/1 matched " + repr(_gan_args))
    scored = gandora_std.enum.take(gandora_std.enum.sort_by(gandora_std.enum.filter(gandora_std.enum.map(freqs, _gan_fn5), _gan_fn6), _gan_fn7), ranked_files)
    def _gan_fn8(*_gan_args, root=root, terms=terms):
        match _gan_args:
            case ((s, path, text) as _gan_t16,) if isinstance(_gan_t16, tuple):
                return {"path": gandora_lsp.atlas.relative(path, root), "score": builtins.round(s, 2), "lines": _best_lines(text, terms)}
        raise GanMatchError("no clause of _gan_fn8/1 matched " + repr(_gan_args))
    return {"files": gandora_std.enum.map(scored, _gan_fn8), "count": gandora_std.enum.count(scored), "truncated": (gandora_std.enum.count(freqs) > ranked_files) and (gandora_std.enum.count(scored) == ranked_files)}


def tokens(text: str) -> list[str]:
    """The lowercased `[a-z0-9_]+` tokens of a text (GEP-0031-R005).

## Parameters

  - text: Any text.

    >>> tokens("File.read! reads UTF-8")
    ['file', 'read', 'reads', 'utf', '8']
"""
    return re.compile("[a-z0-9_]+").findall(gandora_std.string.downcase(text))


def _doc_len(freq):
    return gandora_std.enum.sum(gandora_std.map.values(freq))


def _average(xs):
    if _gan_truthy(gandora_std.enum.empty_p(xs)):
        return 1.0
    else:
        return gandora_std.enum.sum(xs) / gandora_std.enum.count(xs)


def _idf(term, freqs, n):
    def _gan_fn9(*_gan_args, term=term):
        match _gan_args:
            case ((_p, _t, f) as _gan_t17,) if isinstance(_gan_t17, tuple):
                return gandora_std.map.has_key_p(f, term)
        raise GanMatchError("no clause of _gan_fn9/1 matched " + repr(_gan_args))
    df = gandora_std.enum.count(gandora_std.enum.filter(freqs, _gan_fn9))
    return math.log((((n - df) + 0.5) / (df + 0.5)) + 1.0)


def _score(freq, idfs, avg):
    len = _doc_len(freq)
    def _gan_fn10(*_gan_args, avg=avg, freq=freq, len=len):
        match _gan_args:
            case ((term, w) as _gan_t18,) if isinstance(_gan_t18, tuple):
                tf = gandora_std.map.get(freq, term, 0)
                return ((w * tf) * 2.2) / (tf + (1.2 * (0.25 + ((0.75 * len) / avg))))
        raise GanMatchError("no clause of _gan_fn10/1 matched " + repr(_gan_args))
    return gandora_std.enum.sum(gandora_std.enum.map(idfs, _gan_fn10))


def _best_lines(text, terms):
    def _gan_fn11(*_gan_args, terms=terms):
        match _gan_args:
            case ((l, i) as _gan_t19,) if isinstance(_gan_t19, tuple):
                return (_hits_on(l, terms), i + 1, l)
        raise GanMatchError("no clause of _gan_fn11/1 matched " + repr(_gan_args))
    def _gan_fn12(*_gan_args):
        match _gan_args:
            case ((h, _i, _l) as _gan_t20,) if isinstance(_gan_t20, tuple):
                return h > 0
        raise GanMatchError("no clause of _gan_fn12/1 matched " + repr(_gan_args))
    def _gan_fn13(*_gan_args):
        match _gan_args:
            case ((h, i, _l) as _gan_t21,) if isinstance(_gan_t21, tuple):
                return (-(h), i)
        raise GanMatchError("no clause of _gan_fn13/1 matched " + repr(_gan_args))
    def _gan_fn14(*_gan_args):
        match _gan_args:
            case ((_h, i, l) as _gan_t22,) if isinstance(_gan_t22, tuple):
                return {"line": i, "text": gandora_std.string.trim(l)}
        raise GanMatchError("no clause of _gan_fn14/1 matched " + repr(_gan_args))
    return gandora_std.enum.map(gandora_std.enum.take(gandora_std.enum.sort_by(gandora_std.enum.filter(gandora_std.enum.map(gandora_std.enum.with_index(gandora_std.string.split_on(text, "\n")), _gan_fn11), _gan_fn12), _gan_fn13), 3), _gan_fn14)


def _hits_on(line, terms):
    low = tokens(line)
    return gandora_std.enum.count(gandora_std.enum.filter(terms, lambda t, *, low=low: gandora_std.enum.member_p(low, t)))


def read(root: str, target: str, from__kw: int, to: int) -> dict:
    """One precise read (GEP-0031-R006): `{path, from, to, text}` for a
path with a 1-based inclusive line range (0, 0 = the whole file), a
module name (its whole source, project or installed), or `Mod.fun`
(the definition's block, annotations included, bounded by the next
symbol). An unresolvable target answers `{"error": why}`.

## Parameters

  - root: The project root.
  - target: A corpus path, `Mod`, or `Mod.fun`.
  - from: First line, 1-based; 0 for a named or whole-file read.
  - to: Last line, inclusive; 0 for a named or whole-file read.
"""
    if _gan_truthy(gandora_std.string.match_p(target, re.compile("^([A-Z][A-Za-z0-9_]*\\.)+[a-z_][A-Za-z0-9_]*[?!]?$"))):
        return _read_block(root, target)
    elif _gan_truthy(gandora_std.string.match_p(target, re.compile("^[A-Z][A-Za-z0-9_]*(\\.[A-Z][A-Za-z0-9_]*)*$"))):
        return _read_module(root, target)
    else:
        return _read_range(root, target, from__kw, to)


def _read_range(root, path, from__kw, to):
    if _gan_truthy(gandora_std.path.absolute_p(path)):
        _gan_tmp23 = path
    else:
        _gan_tmp23 = gandora_std.path.join(root, path)
    abs = _gan_tmp23
    _gan_case24 = gandora_std.file.read(abs)
    match _gan_case24:
        case ("error", why) as _gan_t25 if isinstance(_gan_t25, tuple):
            return {"error": why}
        case ("ok", text) as _gan_t26 if isinstance(_gan_t26, tuple):
            lines = gandora_std.string.split_on(text, "\n")
            total = gandora_std.enum.count(lines)
            if from__kw < 1:
                _gan_tmp27 = 1
            else:
                _gan_tmp27 = from__kw
            first = _gan_tmp27
            if (to < 1) or (to > total):
                _gan_tmp28 = total
            else:
                _gan_tmp28 = to
            last = _gan_tmp28
            slice = gandora_std.enum.take(gandora_std.enum.drop(lines, first - 1), (last - first) + 1)
            return {"path": gandora_lsp.atlas.relative(abs, root), "from": first, "to": last, "text": gandora_std.enum.join(slice, "\n")}
        case _:
            raise GanMatchError("no case clause matched: " + repr(_gan_case24))


def _read_module(root, mod):
    try:
        _gan_tmp29 = core.definition(mod, root)
    except Exception as _e__gan3:
        _gan_tmp29 = None
    hit = _gan_tmp29
    if (hit is None):
        return {"error": f"module {mod} does not resolve here"}
    else:
        return _read_range(root, str(gandora_std.map.get(hit, "path")), 0, 0)


def _read_block(root, target):
    try:
        _gan_tmp30 = core.definition(target, root)
    except Exception as _e__gan4:
        _gan_tmp30 = None
    hit = _gan_tmp30
    if (hit is None):
        return {"error": f"{target} does not resolve here"}
    else:
        path = str(gandora_std.map.get(hit, "path"))
        parts = gandora_std.string.split_on(target, ".")
        mod = gandora_std.enum.join(gandora_std.enum.take(parts, gandora_std.enum.count(parts) - 1), ".")
        name = gandora_std.enum.at(parts, -1)
        try:
            _gan_tmp31 = core.symbols(mod, root)
        except Exception as _e__gan5:
            _gan_tmp31 = []
        sym_lines = gandora_std.enum.sort(gandora_std.enum.map(_gan_tmp31, lambda s: gandora_std.map.get(s, "line", 0)))
        try:
            _gan_tmp32 = gandora_std.file.read_bang(path)
        except Exception as _e__gan6:
            _gan_tmp32 = ""
        lines = gandora_std.string.split_on(_gan_tmp32, "\n")
        total = gandora_std.enum.count(lines)
        def_line = gandora_std.map.get(hit, "line", 1)
        prev = gandora_std.enum.at(gandora_std.enum.filter(sym_lines, lambda l, *, def_line=def_line: l < def_line), -1)
        next = gandora_std.enum.at(gandora_std.enum.filter(sym_lines, lambda l, *, def_line=def_line: l > def_line), 0)
        if (prev is None):
            _gan_tmp33 = 0
        else:
            _gan_tmp33 = prev
        floor = _gan_tmp33
        if (next is None):
            _gan_tmp34 = total - 1
        else:
            _gan_tmp34 = block_start(lines, next, def_line) - 1
        stop = _gan_tmp34
        start = block_start(lines, def_line, floor)
        return gandora_std.map.put(gandora_std.map.put(_read_range(root, path, start, _trim_blank(lines, stop)), "target", target), "name", name)


def block_start(lines: collections.abc.Sequence[str], def_line: int, floor: int) -> int:
    """Where a definition's block starts: the first line of the annotation
run (`@...`/comments, heredoc bodies included) that ends directly
above `def_line`, never reaching back past `floor` — the previous
definition's own line.

## Parameters

  - lines: The file's lines.
  - def_line: The 1-based line of the definition.
  - floor: The previous definition's line; 0 at the top.

    >>> block_start(["defmodule M do", "  @doc \\"d\\"", "  def f(), do: 1", "end"], 3, 0)
    2
"""
    return _walk_annotations(lines, floor + 1, def_line, None, False)


def _walk_annotations(lines, i, def_line, start, open):
    while True:
        if i >= def_line:
            if (start is None):
                return def_line
            else:
                return start
        else:
            line = gandora_std.enum.at(lines, i - 1)
            fence_count = gandora_std.enum.count(gandora_std.string.split_on(line, "\"\"\""))
            fences = _gan_rem(fence_count - 1, 2) == 1
            if _gan_truthy(open):
                lines, i, def_line, start, open = lines, i + 1, def_line, start, not (_gan_truthy(fences))
                continue
            elif _gan_truthy(gandora_std.string.match_p(line, re.compile("^\\s*(@|#)"))):
                if (start is None):
                    _gan_tmp35 = i
                else:
                    _gan_tmp35 = start
                lines, i, def_line, start, open = lines, i + 1, def_line, _gan_tmp35, fences
                continue
            else:
                lines, i, def_line, start, open = lines, i + 1, def_line, None, False
                continue


def _trim_blank(lines, stop):
    while True:
        line = gandora_std.enum.at(lines, stop - 1)
        if ((stop > 1) and not ((line is None))) and (gandora_std.string.trim(line) == ""):
            lines, stop = lines, stop - 1
            continue
        else:
            return stop
