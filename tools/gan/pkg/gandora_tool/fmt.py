"""gan fmt (GEP-0016): conservative source formatting over the
gandora_core token stream. Rewrites indentation and horizontal
whitespace, and reflows multi-line map literals to one pair per
line (R011) — never joins lines — and refuses to write anything
whose comments or parsed terms differ from the original (R006).
"""

import builtins
import collections.abc
import difflib
import gandora_core as core
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

hang_enders = ["=", "<-", "\\\\", "++", "<>", "and", "or", "when", "in"]

hang_starters = ["++", "<>", "and", "or", "when", "in", "==", "!=", "<=", ">=", "<", ">"]

space_around = ["=", "|>", "->", "<-", "=>", "\\\\"]


def run(args: collections.abc.Sequence[str]) -> None:
    """The `gan fmt` entry: formats the given paths, or stdin with `-`.

## Parameters

  - args: CLI arguments after `fmt`: paths, `-` for stdin, --check, --diff.
"""
    check = gandora_std.enum.member_p(args, "--check")
    diff = gandora_std.enum.member_p(args, "--diff")
    paths = gandora_std.enum.reject(args, lambda a: gandora_std.string.starts_with_p(a, "--"))
    if paths == ["-"]:
        return _run_stdin()
    else:
        files = _collect_files(paths)
        if _gan_truthy(gandora_std.enum.empty_p(files)):
            print("gan fmt: no .gan files found")
            gandora_std.system.halt(1)
        results = gandora_std.enum.map(files, lambda f, *, check=check, diff=diff: _format_file(f, _gan_or(check, lambda: diff)))
        def _gan_fn0(*_gan_args):
            match _gan_args:
                case ((_, status) as _gan_t0,) if isinstance(_gan_t0, tuple):
                    return status == "changed"
            raise GanMatchError("no clause of _gan_fn0/1 matched " + repr(_gan_args))
        changed = gandora_std.enum.filter(results, _gan_fn0)
        def _gan_fn1(*_gan_args):
            match _gan_args:
                case ((_, status) as _gan_t1,) if isinstance(_gan_t1, tuple):
                    return status == "error"
            raise GanMatchError("no clause of _gan_fn1/1 matched " + repr(_gan_args))
        errors = gandora_std.enum.filter(results, _gan_fn1)
        if not (_gan_truthy(gandora_std.enum.empty_p(errors))):
            return gandora_std.system.halt(2)
        elif _gan_truthy(diff):
            def _gan_fn2(*_gan_args):
                match _gan_args:
                    case ((f, _) as _gan_t2,) if isinstance(_gan_t2, tuple):
                        return _print_diff(f)
                raise GanMatchError("no clause of _gan_fn2/1 matched " + repr(_gan_args))
            gandora_std.enum.each(changed, _gan_fn2)
            if not (_gan_truthy(gandora_std.enum.empty_p(changed))):
                return gandora_std.system.halt(1)
            else:
                return None
        elif _gan_truthy(_gan_and(check, lambda: not (_gan_truthy(gandora_std.enum.empty_p(changed))))):
            def _gan_fn3(*_gan_args):
                match _gan_args:
                    case ((f, _) as _gan_t3,) if isinstance(_gan_t3, tuple):
                        return print(f"would reformat {f}")
                raise GanMatchError("no clause of _gan_fn3/1 matched " + repr(_gan_args))
            gandora_std.enum.each(changed, _gan_fn3)
            return gandora_std.system.halt(1)
        else:
            return print(f"{gandora_std.enum.count(changed)} file(s) reformatted, {gandora_std.enum.count(files)} checked")


def _run_stdin():
    text = sys.stdin.read()
    _gan_case4 = format_text(text)
    match _gan_case4:
        case ("ok", new) as _gan_t5 if isinstance(_gan_t5, tuple):
            if (new != text) and (verify(text, new) != "ok"):
                sys.stderr.write("gan fmt: verification failed; input left as-is\n")
                sys.stdout.write(text)
                return gandora_std.system.halt(2)
            else:
                sys.stdout.write(new)
                return None
        case _:
            sys.stderr.write("gan fmt: cannot parse input\n")
            sys.stdout.write(text)
            return gandora_std.system.halt(2)


def _print_diff(file):
    text = gandora_std.file.read_bang(file)
    _gan_case6 = format_text(text)
    match _gan_case6:
        case ("ok", new) as _gan_t7 if isinstance(_gan_t7, tuple):
            lines = difflib.unified_diff(text.splitlines(keepends=True), new.splitlines(keepends=True), fromfile=file, tofile=file + " (formatted)")
            return gandora_std.enum.each(builtins.list(lines), lambda l: sys.stdout.write(l))
        case _:
            return None


def _collect_files(paths):
    if _gan_truthy(gandora_std.enum.empty_p(paths)):
        _gan_tmp8 = ["src"]
    else:
        _gan_tmp8 = paths
    roots = _gan_tmp8
    def _gan_fn4(p):
        if _gan_truthy(gandora_std.file.dir_p(p)):
            return gandora_std.path.wildcard(gandora_std.path.join(gandora_std.path.join(p, "**"), "*.gan"))
        elif _gan_truthy(gandora_std.string.ends_with_p(p, ".gan")):
            return [p]
        else:
            return []
    return gandora_std.enum.flat_map(roots, _gan_fn4)


def _format_file(file, check):
    text = gandora_std.file.read_bang(file)
    _gan_case10 = format_text(text)
    match _gan_case10:
        case ("ok", new) as _gan_t11 if isinstance(_gan_t11, tuple):
            if new == text:
                return (file, "unchanged")
            elif _gan_truthy(check):
                return (file, "changed")
            else:
                _gan_case12 = verify(text, new)
                match _gan_case12:
                    case "ok":
                        gandora_std.file.write_bang(file, new)
                        print(f"reformatted {file}")
                        return (file, "changed")
                    case ("error", why) as _gan_t13 if isinstance(_gan_t13, tuple):
                        print(f"gan fmt: internal error on {file}: {why} (GEP-0016-R006); file left unchanged")
                        return (file, "error")
                    case _:
                        raise GanMatchError("no case clause matched: " + repr(_gan_case12))
        case ("skip", why) as _gan_t14 if isinstance(_gan_t14, tuple):
            print(f"gan fmt: skipped {file}: {why}")
            return (file, "unchanged")
        case _:
            raise GanMatchError("no case clause matched: " + repr(_gan_case10))


def verify(old_text: str, new_text: str) -> str | tuple[str, str]:
    """Confirms a rewrite preserves comments and parsed terms (R006).

## Parameters

  - old_text: The original source.
  - new_text: The candidate rewrite that must preserve comments and parse.

    >>> verify("x = 1", "x = 1")
    'ok'
    >>> verify("x = 1", "x = 2")
    ('error', 'parsed terms changed')
"""
    if _comments_of(old_text) != _comments_of(new_text):
        return ("error", "comment sequence changed")
    elif _strip_meta(core.parse(old_text)) != _strip_meta(core.parse(new_text)):
        return ("error", "parsed terms changed")
    else:
        return "ok"


def _comments_of(text):
    return [gandora_std.map.get(t, "value").rstrip() for t in core.tokens(text) if gandora_std.map.get(t, "kind") == "comment"]


def _strip_meta(t):
    if _gan_truthy(_gan_and(builtins.isinstance(t, builtins.tuple), lambda: builtins.len(t) == 3)):
        return (_strip_meta(t[0]), _strip_meta(t[2]))
    elif _gan_truthy(builtins.isinstance(t, builtins.tuple)):
        return builtins.tuple(gandora_std.enum.map(builtins.list(t), _strip_meta))
    elif _gan_truthy(builtins.isinstance(t, builtins.list)):
        return gandora_std.enum.map(t, _strip_meta)
    else:
        return t


def format_text(text: str) -> tuple[str, str]:
    """Formats one source text: `{:ok, new}` or `{:skip, why}`.

## Parameters

  - text: The source to normalize.
"""
    try:
        core.parse(text)
        _gan_tmp15 = "ok"
    except core.CompileError as _e:
        _gan_tmp15 = "parse_error"
    parsed = _gan_tmp15
    if parsed == "parse_error":
        return ("skip", "does not parse")
    else:
        return ("ok", _reflow(_explode_maps(text)))


def _explode_maps(text):
    toks = gandora_std.enum.reject(core.tokens(text), lambda t: gandora_std.enum.member_p(["eof", "newline"], gandora_std.map.get(t, "kind")))
    points = gandora_std.enum.flat_map(_map_spans(toks), lambda span, *, toks=toks: _split_points(toks, span))
    if _gan_truthy(gandora_std.enum.empty_p(points)):
        return text
    else:
        return _apply_splits(text, gandora_std.enum.uniq(points))


def _map_spans(toks):
    def _gan_fn5(*_gan_args):
        match _gan_args:
            case ((t, i) as _gan_t16, (stack, spans) as _gan_t17,) if isinstance(_gan_t16, tuple) and isinstance(_gan_t17, tuple):
                if _gan_truthy(_opener_tok_p(t)):
                    return ([(t, i)] + stack, spans)
                elif _gan_truthy(_gan_and(_closer_p(t), lambda: not (_gan_truthy(gandora_std.enum.empty_p(stack))))):
                    _gan_val18 = gandora_std.enum.at(stack, 0)
                    match _gan_val18:
                        case (opened, j) as _gan_t19 if isinstance(_gan_t19, tuple):
                            pass
                        case _:
                            raise GanMatchError("no match of right-hand side value: " + repr(_gan_val18))
                    if _gan_truthy(_tok_is(opened, "op", "%{")):
                        _gan_tmp20 = [(j, i)] + spans
                    else:
                        _gan_tmp20 = spans
                    return (gandora_std.enum.drop(stack, 1), _gan_tmp20)
                else:
                    return (stack, spans)
        raise GanMatchError("no clause of _gan_fn5/2 matched " + repr(_gan_args))
    walk = gandora_std.enum.reduce(gandora_std.enum.with_index(toks), ([], []), _gan_fn5)
    _gan_val21 = walk
    match _gan_val21:
        case (_stack, spans) as _gan_t22 if isinstance(_gan_t22, tuple):
            pass
        case _:
            raise GanMatchError("no match of right-hand side value: " + repr(_gan_val21))
    def _gan_fn6(*_gan_args, toks=toks):
        match _gan_args:
            case ((i, j) as _gan_t23,) if isinstance(_gan_t23, tuple):
                return _reflowable_p(toks, i, j)
        raise GanMatchError("no clause of _gan_fn6/1 matched " + repr(_gan_args))
    return gandora_std.enum.filter(spans, _gan_fn6)


def _reflowable_p(toks, i, j):
    open = gandora_std.enum.at(toks, i)
    close = gandora_std.enum.at(toks, j)
    inside = gandora_std.enum.slice(toks, i + 1, (j - i) - 1)
    return _gan_and(gandora_std.map.get(close, "line") > gandora_std.map.get(open, "line"), lambda: gandora_std.enum.empty_p(gandora_std.enum.filter(inside, lambda t: _gan_or(gandora_std.map.get(t, "kind") == "comment", lambda: _multiline_p(t)))))


def _split_points(*_gan_args):
    match _gan_args:
        case (toks, (i, j) as _gan_t24,) if isinstance(_gan_t24, tuple):
            open = gandora_std.enum.at(toks, i)
            close = gandora_std.enum.at(toks, j)
            first = gandora_std.enum.at(toks, i + 1)
            if (gandora_std.map.get(first, "line") == gandora_std.map.get(open, "line")) and ((i + 1) < j):
                _gan_tmp25 = [_point(first)]
            else:
                _gan_tmp25 = []
            head = _gan_tmp25
            if gandora_std.map.get(gandora_std.enum.at(toks, j - 1), "line") == gandora_std.map.get(close, "line"):
                _gan_tmp26 = [_point(close)]
            else:
                _gan_tmp26 = []
            tail = _gan_tmp26
            return head + (_comma_points(toks, i, j) + tail)
    raise GanMatchError("no clause of split_points/2 matched " + repr(_gan_args))


def _comma_points(toks, i, j):
    def _gan_fn7(*_gan_args, j=j, toks=toks):
        match _gan_args:
            case (t, (depth, k, pts) as _gan_t27,) if isinstance(_gan_t27, tuple):
                if _gan_truthy(_opener_tok_p(t)):
                    return (depth + 1, k + 1, pts)
                elif _gan_truthy(_closer_p(t)):
                    return (depth - 1, k + 1, pts)
                elif _gan_truthy(_gan_and(_gan_and(_gan_and(_tok_is(t, "op", ","), lambda: depth == 1), lambda: (k + 1) < j), lambda: gandora_std.map.get(gandora_std.enum.at(toks, k + 1), "line") == gandora_std.map.get(t, "line"))):
                    return (depth, k + 1, [_point(gandora_std.enum.at(toks, k + 1))] + pts)
                else:
                    return (depth, k + 1, pts)
        raise GanMatchError("no clause of _gan_fn7/2 matched " + repr(_gan_args))
    walk = gandora_std.enum.reduce(gandora_std.enum.slice(toks, i + 1, (j - i) - 1), (1, i + 1, []), _gan_fn7)
    return walk[2]


def _point(t):
    return (gandora_std.map.get(t, "line"), gandora_std.map.get(t, "col"))


def _apply_splits(text, points):
    lines = text.split("\n")
    def _gan_fn8(*_gan_args):
        match _gan_args:
            case ((l, c) as _gan_t28, acc,) if isinstance(_gan_t28, tuple):
                return gandora_std.map.put(acc, l, gandora_std.enum.sort(gandora_std.map.get(acc, l, []) + [c]))
        raise GanMatchError("no clause of _gan_fn8/2 matched " + repr(_gan_args))
    by_line = gandora_std.enum.reduce(points, {}, _gan_fn8)
    def _gan_fn9(*_gan_args, by_line=by_line):
        match _gan_args:
            case ((line, idx) as _gan_t29,) if isinstance(_gan_t29, tuple):
                return _split_line(line, gandora_std.enum.reverse(gandora_std.map.get(by_line, idx + 1, [])))
        raise GanMatchError("no clause of _gan_fn9/1 matched " + repr(_gan_args))
    pieces = gandora_std.enum.map(gandora_std.enum.with_index(lines), _gan_fn9)
    return gandora_std.enum.join(pieces, "\n")


def _split_line(line, cols):
    return gandora_std.enum.reduce(cols, line, lambda c, acc: gandora_std.string.slice(acc, 0, c - 1).rstrip() + ("\n" + gandora_std.string.slice(acc, c - 1, builtins.len(acc))))


def _reflow(text):
    toks = gandora_std.enum.reject(core.tokens(text), lambda t: (gandora_std.map.get(t, "kind") == "eof") or (gandora_std.map.get(t, "kind") == "newline"))
    lines = text.split("\n")
    interior = _interior_lines(toks)
    buckets = _bucket_by_line(toks)
    n = gandora_std.enum.count(lines)
    out = _reflow_walk((1, [], "blank_start", None, False, {}, []), lines, interior, buckets, n)
    return gandora_std.enum.join(_trim_trailing_blanks(out), "\n") + "\n"


def _reflow_walk(*_gan_args):
    while True:
        match _gan_args:
            case ((i, stack, prev_kind, prev_last, prev_hang, deltas, acc) as _gan_t30, lines, interior, buckets, n,) if isinstance(_gan_t30, tuple):
                if i > n:
                    return gandora_std.enum.reverse(acc)
                else:
                    line = gandora_std.enum.at(lines, i - 1)
                    lt = gandora_std.map.get(buckets, i, [])
                    if _gan_truthy(gandora_std.map.has_key_p(interior, i)):
                        d = gandora_std.map.get(deltas, gandora_std.map.get(interior, i), 0)
                        if _gan_truthy(gandora_std.enum.empty_p(lt)):
                            _gan_tmp31 = stack
                        else:
                            _gan_tmp31 = _advance(lt, stack, gandora_std.enum.count(stack))
                        stack2 = _gan_tmp31
                        if _gan_truthy(gandora_std.enum.empty_p(lt)):
                            _gan_tmp32 = prev_last
                        else:
                            _gan_tmp32 = _last_significant(lt)
                        last2 = _gan_tmp32
                        _gan_args = ((i + 1, stack2, "code", last2, prev_hang, deltas, [_shift(line, d)] + acc), lines, interior, buckets, n)
                        continue
                    elif _gan_truthy(gandora_std.enum.empty_p(lt)):
                        if (prev_kind == "blank") or (prev_kind == "blank_start"):
                            _gan_args = ((i + 1, stack, prev_kind, prev_last, prev_hang, deltas, acc), lines, interior, buckets, n)
                            continue
                        else:
                            _gan_args = ((i + 1, stack, "blank", prev_last, prev_hang, deltas, [""] + acc), lines, interior, buckets, n)
                            continue
                    else:
                        _gan_val33 = _line_level(lt, stack, prev_last, prev_hang)
                        match _gan_val33:
                            case (level, stack2, hung) as _gan_t34 if isinstance(_gan_t34, tuple):
                                pass
                            case _:
                                raise GanMatchError("no match of right-hand side value: " + repr(_gan_val33))
                        indent = gandora_std.string.duplicate("  ", level)
                        multi = gandora_std.enum.find(lt, _multiline_p)
                        if (multi is None):
                            _gan_tmp35 = indent + _render_tokens(_capture_parens(lt), lines)
                        else:
                            _gan_tmp35 = indent + line.strip()
                        rendered = _gan_tmp35
                        if (multi is None):
                            _gan_tmp36 = deltas
                        else:
                            old_lead = builtins.len(line) - builtins.len(line.lstrip())
                            _gan_tmp36 = gandora_std.map.put(deltas, i, builtins.len(indent) - old_lead)
                        deltas2 = _gan_tmp36
                        stack3 = _advance(lt, stack2, level)
                        _gan_args = ((i + 1, stack3, "code", _last_significant(lt), hung, deltas2, [rendered] + acc), lines, interior, buckets, n)
                        continue
        raise GanMatchError("no clause of reflow_walk/5 matched " + repr(_gan_args))


def _trim_trailing_blanks(ls):
    return gandora_std.enum.reverse(gandora_std.enum.drop_while(gandora_std.enum.reverse(ls), lambda l: l == ""))


def _shift(line, d):
    s = line.lstrip()
    if s == "":
        return ""
    else:
        lead = builtins.len(line) - builtins.len(s)
        return gandora_std.string.duplicate(" ", gandora_std.enum.max([lead + d, 0])) + s.rstrip()


def _multiline_p(t):
    return ((gandora_std.map.get(t, "kind") == "str") or (gandora_std.map.get(t, "kind") == "sigil")) and (gandora_std.map.get(t, "end_line") > gandora_std.map.get(t, "line"))


def _interior_lines(toks):
    def _gan_fn10(t, acc):
        if _gan_truthy(_multiline_p(t)):
            interior = builtins.range(gandora_std.map.get(t, "line") + 1, gandora_std.map.get(t, "end_line") + 1)
            return gandora_std.enum.reduce(interior, acc, lambda l, a, *, t=t: gandora_std.map.put(a, l, gandora_std.map.get(t, "line")))
        else:
            return acc
    return gandora_std.enum.reduce(toks, {}, _gan_fn10)


def _bucket_by_line(toks):
    def _gan_fn11(t, acc):
        l = gandora_std.map.get(t, "line")
        return gandora_std.map.put(acc, l, gandora_std.map.get(acc, l, []) + [t])
    return gandora_std.enum.reduce(toks, {}, _gan_fn11)


def _last_significant(lt):
    sig = gandora_std.enum.reject(lt, lambda t: gandora_std.map.get(t, "kind") == "comment")
    return gandora_std.enum.at(sig, -1)


def _tok_is(t, kind, value):
    return (gandora_std.map.get(t, "kind") == kind) and (gandora_std.map.get(t, "value") == value)


def _closer_p(t):
    return _gan_or((gandora_std.map.get(t, "kind") == "kw") and (gandora_std.map.get(t, "value") == "end"), lambda: _gan_and(gandora_std.map.get(t, "kind") == "op", lambda: gandora_std.enum.member_p([")", "]", "}"], gandora_std.map.get(t, "value"))))


def _mid_p(t):
    return _gan_and(gandora_std.map.get(t, "kind") == "kw", lambda: gandora_std.enum.member_p(["else", "rescue", "after"], gandora_std.map.get(t, "value")))


def _opener_tok_p(t):
    return _gan_or(_gan_and(gandora_std.map.get(t, "kind") == "op", lambda: gandora_std.enum.member_p(["(", "[", "{", "%{"], gandora_std.map.get(t, "value"))), lambda: _gan_and(gandora_std.map.get(t, "kind") == "kw", lambda: gandora_std.enum.member_p(["do", "fn"], gandora_std.map.get(t, "value"))))


def _clause_head_p(lt):
    def _gan_fn12(*_gan_args):
        match _gan_args:
            case (t, (depth, found) as _gan_t38,) if isinstance(_gan_t38, tuple):
                if _gan_truthy(_opener_tok_p(t)):
                    return (depth + 1, found)
                elif _gan_truthy(_closer_p(t)):
                    return (gandora_std.enum.max([depth - 1, 0]), found)
                elif _gan_truthy(_gan_and(_tok_is(t, "op", "->"), lambda: depth == 0)):
                    return (depth, True)
                else:
                    return (depth, found)
        raise GanMatchError("no clause of _gan_fn12/2 matched " + repr(_gan_args))
    r = gandora_std.enum.reduce(lt, (0, False), _gan_fn12)
    return r[1]


def _ends_with_arrow_p(lt):
    last = _last_significant(lt)
    return _gan_and(not ((last is None)), lambda: _tok_is(last, "op", "->"))


def _hang_p(first, prev_last, prev_hang, in_paren):
    starts = _gan_and(gandora_std.enum.member_p(["op", "kw"], gandora_std.map.get(first, "kind")), lambda: gandora_std.enum.member_p(hang_starters, gandora_std.map.get(first, "value")))
    ends = _gan_and(_gan_and(not ((prev_last is None)), lambda: gandora_std.enum.member_p(["op", "kw"], gandora_std.map.get(prev_last, "kind"))), lambda: gandora_std.enum.member_p(hang_enders, gandora_std.map.get(prev_last, "value")))
    comma = _gan_and(not (_gan_truthy(in_paren)) and not ((prev_last is None)), lambda: _tok_is(prev_last, "op", ","))
    continues = _gan_and(_tok_is(first, "op", "|>"), lambda: prev_hang)
    return _gan_or(_gan_or(_gan_or(starts, lambda: ends), lambda: comma), lambda: continues)


def _line_level(lt, stack, prev_last, prev_hang):
    first = gandora_std.enum.at(lt, 0)
    popped = _pop_clauses(stack, first, lt)
    top = gandora_std.enum.at(popped, 0)
    if _gan_truthy(gandora_std.enum.empty_p(popped)):
        _gan_tmp39 = 0
    elif _gan_truthy(_closer_p(first)):
        _gan_tmp39 = gandora_std.map.get(top, "open")
    elif _gan_truthy(_mid_p(first)):
        _gan_tmp39 = gandora_std.map.get(top, "open")
    else:
        _gan_tmp39 = gandora_std.map.get(top, "body")
    level = _gan_tmp39
    top_paren = not (_gan_truthy(gandora_std.enum.empty_p(popped))) and (gandora_std.map.get(gandora_std.enum.at(popped, 0), "kind") == "paren")
    hung = _gan_and(not (_gan_truthy(_closer_p(first))) and not (_gan_truthy(_mid_p(first))), lambda: _hang_p(first, prev_last, prev_hang, top_paren))
    if _gan_truthy(hung):
        _gan_tmp40 = level + 1
    else:
        _gan_tmp40 = level
    level = _gan_tmp40
    return (gandora_std.enum.max([level, 0]), popped, hung)


def _pop_clauses(s, first, lt):
    while True:
        if _gan_truthy(gandora_std.enum.empty_p(s)):
            return s
        else:
            top = gandora_std.enum.at(s, 0)
            if _gan_truthy(_gan_and(gandora_std.map.get(top, "kind") == "clause", lambda: _gan_or(_gan_or(_closer_p(first), lambda: _mid_p(first)), lambda: _clause_head_p(lt)))):
                s, first, lt = gandora_std.enum.drop(s, 1), first, lt
                continue
            else:
                return s


def _advance(lt, stack, level):
    def _gan_fn13(t, s, *, level=level):
        if _gan_truthy(_opener_tok_p(t)):
            return [{"kind": _struct_kind(t), "body": level + 1, "open": level}] + s
        elif _gan_truthy(_closer_p(t)):
            return _pop_to_opener(s)
        else:
            return s
    stack2 = gandora_std.enum.reduce(lt, stack, _gan_fn13)
    if _gan_truthy(_ends_with_arrow_p(lt)):
        return [{"kind": "clause", "body": level + 1, "open": level}] + stack2
    else:
        return stack2


def _struct_kind(t):
    if gandora_std.map.get(t, "kind") == "kw":
        return "block"
    else:
        return "paren"


def _pop_to_opener(s):
    while True:
        _gan_case42 = s
        match _gan_case42:
            case [] as _gan_l43 if isinstance(_gan_l43, list):
                return []
            case [{"kind": "clause"}, *rest] as _gan_l44 if isinstance(_gan_l44, list):
                s = rest
                continue
            case [_, *rest] as _gan_l45 if isinstance(_gan_l45, list):
                return rest
            case _:
                raise GanMatchError("no case clause matched: " + repr(_gan_case42))


def _render_tokens(lt, lines):
    return _render_walk((lt, "", None), lines)


def _render_walk(*_gan_args):
    while True:
        match _gan_args:
            case ((toks, out, prev) as _gan_t46, lines,) if isinstance(_gan_t46, tuple):
                _gan_case47 = toks
                match _gan_case47:
                    case [] as _gan_l48 if isinstance(_gan_l48, list):
                        return out
                    case [t, *rest] as _gan_l49 if isinstance(_gan_l49, list):
                        txt = _tok_text(t, lines)
                        if (prev is None):
                            _gan_tmp50 = txt
                        elif _gap(prev, t) == 0:
                            _gan_tmp50 = txt
                        else:
                            _gan_tmp50 = " " + txt
                        piece = _gan_tmp50
                        _gan_args = ((rest, out + piece, t), lines)
                        continue
                    case _:
                        raise GanMatchError("no case clause matched: " + repr(_gan_case47))
        raise GanMatchError("no clause of render_walk/2 matched " + repr(_gan_args))


def _space_around_p(t):
    return _gan_and(gandora_std.map.get(t, "kind") == "op", lambda: gandora_std.enum.member_p(space_around, gandora_std.map.get(t, "value")))


def _gap(prev, t):
    orig = gandora_std.map.get(t, "col") - gandora_std.map.get(prev, "end_col")
    if _gan_truthy(_tok_is(t, "op", ",")):
        return 0
    elif _gan_truthy(_gan_and(gandora_std.map.get(t, "kind") == "op", lambda: gandora_std.enum.member_p([")", "]", "}"], gandora_std.map.get(t, "value")))):
        return 0
    elif _gan_truthy(_tok_is(prev, "op", ",")):
        return 1
    elif gandora_std.map.get(t, "kind") == "comment":
        return 1
    elif gandora_std.map.get(prev, "kind") == "kwkey":
        return 1
    elif _gan_truthy(_gan_or(_space_around_p(prev), lambda: _space_around_p(t))):
        return 1
    elif _gan_truthy(_gan_and(gandora_std.map.get(t, "kind") == "kw", lambda: gandora_std.enum.member_p(["do", "when", "in", "and", "or"], gandora_std.map.get(t, "value")))):
        return 1
    elif _gan_truthy(_gan_and(gandora_std.map.get(prev, "kind") == "kw", lambda: gandora_std.enum.member_p(["do", "when", "in", "and", "or", "not", "fn", "end", "else", "rescue", "after"], gandora_std.map.get(prev, "value")))):
        return 1
    elif orig <= 0:
        return 0
    else:
        return 1


def _tok_text(t, lines):
    if gandora_std.map.get(t, "kind") == "comment":
        return gandora_std.map.get(t, "value").rstrip()
    elif _gan_truthy(gandora_std.map.get(t, "synthetic", False)):
        return gandora_std.map.get(t, "value")
    else:
        line = gandora_std.enum.at(lines, gandora_std.map.get(t, "line") - 1)
        return gandora_std.string.slice(line, gandora_std.map.get(t, "col") - 1, gandora_std.map.get(t, "end_col") - gandora_std.map.get(t, "col"))


def _capture_parens(lt):
    return _capture_walk((lt, []))


def _capture_walk(*_gan_args):
    while True:
        match _gan_args:
            case ((toks, out) as _gan_t51,) if isinstance(_gan_t51, tuple):
                _gan_case52 = toks
                match _gan_case52:
                    case [] as _gan_l53 if isinstance(_gan_l53, list):
                        return gandora_std.enum.reverse(out)
                    case [t, *rest] as _gan_l54 if isinstance(_gan_l54, list):
                        if _gan_truthy(_gan_and(_gan_and(_tok_is(t, "op", "&"), lambda: not (_gan_truthy(gandora_std.enum.empty_p(rest)))), lambda: gandora_std.map.get(gandora_std.enum.at(rest, 0), "kind") == "pyref")):
                            _gan_case55 = _split_capture(rest)
                            match _gan_case55:
                                case (body, tail) as _gan_t56 if isinstance(_gan_t56, tuple):
                                    open = _synth("(", t)
                                    close = _synth(")", gandora_std.enum.at(body, -1))
                                    _gan_args = ((tail, [close] + (gandora_std.enum.reverse(body) + ([open, t] + out))),)
                                    continue
                                case None:
                                    _gan_args = ((rest, [t] + out),)
                                    continue
                                case _:
                                    raise GanMatchError("no case clause matched: " + repr(_gan_case55))
                        else:
                            _gan_args = ((rest, [t] + out),)
                            continue
                    case _:
                        raise GanMatchError("no case clause matched: " + repr(_gan_case52))
        raise GanMatchError("no clause of capture_walk/1 matched " + repr(_gan_args))


def _split_capture(toks):
    return _split_walk((toks, [], "ref"))


def _split_walk(*_gan_args):
    while True:
        match _gan_args:
            case ((left, taken, state) as _gan_t57,) if isinstance(_gan_t57, tuple):
                _gan_case58 = (state, left)
                match _gan_case58:
                    case ("ref", [t, *rest] as _gan_l59) as _gan_t60 if isinstance(_gan_l59, list) and isinstance(_gan_t60, tuple):
                        if gandora_std.map.get(t, "kind") == "pyref":
                            _gan_args = ((rest, [t] + taken, "dot"),)
                            continue
                        else:
                            return None
                    case ("dot", [t, *rest] as _gan_l61) as _gan_t62 if isinstance(_gan_l61, list) and isinstance(_gan_t62, tuple):
                        if _gan_truthy(_tok_is(t, "op", ".")):
                            _gan_args = ((rest, [t] + taken, "name"),)
                            continue
                        elif _gan_truthy(_tok_is(t, "op", "/")):
                            _gan_args = ((rest, [t] + taken, "arity"),)
                            continue
                        else:
                            return None
                    case ("name", [t, *rest] as _gan_l63) as _gan_t64 if isinstance(_gan_l63, list) and isinstance(_gan_t64, tuple):
                        if (gandora_std.map.get(t, "kind") == "ident") or (gandora_std.map.get(t, "kind") == "upident"):
                            _gan_args = ((rest, [t] + taken, "dot"),)
                            continue
                        else:
                            return None
                    case ("arity", [t, *rest] as _gan_l65) as _gan_t66 if isinstance(_gan_l65, list) and isinstance(_gan_t66, tuple):
                        if gandora_std.map.get(t, "kind") == "int":
                            return (gandora_std.enum.reverse([t] + taken), rest)
                        else:
                            return None
                    case (_, [] as _gan_l67) as _gan_t68 if isinstance(_gan_l67, list) and isinstance(_gan_t68, tuple):
                        return None
                    case _:
                        raise GanMatchError("no case clause matched: " + repr(_gan_case58))
        raise GanMatchError("no clause of split_walk/1 matched " + repr(_gan_args))


def _synth(op, near):
    return {"kind": "op", "value": op, "synthetic": True, "line": gandora_std.map.get(near, "line"), "col": gandora_std.map.get(near, "end_col"), "end_line": gandora_std.map.get(near, "line"), "end_col": gandora_std.map.get(near, "end_col")}
