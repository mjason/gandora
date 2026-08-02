"""gan fmt (GEP-0016): conservative source formatting over the
gandora_core token stream. Rewrites indentation and horizontal
whitespace only — never joins or splits lines — and refuses to
write anything whose comments or parsed terms differ from the
original (R006).
"""

import builtins
import gandora_core as core
import pathlib
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

class GanMatchError(Exception):
    pass

hang_enders = ["=", "<-", "\\\\", "++", "<>", "and", "or", "when", "in"]

hang_starters = ["++", "<>", "and", "or", "when", "in", "==", "!=", "<=", ">=", "<", ">"]

space_around = ["=", "|>", "->", "<-", "=>", "\\\\"]


def run(args):
    check = gandora_std.enum.member_p(args, "--check")
    paths = gandora_std.enum.reject(args, lambda a: gandora_std.string.starts_with_p(a, "--"))
    files = _collect_files(paths)
    if _gan_truthy(gandora_std.enum.empty_p(files)):
        print("gan fmt: no .gan files found")
        sys.exit(1)
    results = gandora_std.enum.map(files, lambda f: _format_file(f, check))
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
        return sys.exit(2)
    elif _gan_truthy(_gan_and(check, lambda: not (_gan_truthy(gandora_std.enum.empty_p(changed))))):
        def _gan_fn2(*_gan_args):
            match _gan_args:
                case ((f, _) as _gan_t2,) if isinstance(_gan_t2, tuple):
                    return print(f"would reformat {f}")
            raise GanMatchError("no clause of _gan_fn2/1 matched " + repr(_gan_args))
        gandora_std.enum.each(changed, _gan_fn2)
        return sys.exit(1)
    else:
        return print(f"{gandora_std.enum.count(changed)} file(s) reformatted, {gandora_std.enum.count(files)} checked")


def _collect_files(paths):
    if _gan_truthy(gandora_std.enum.empty_p(paths)):
        _gan_tmp3 = ["src"]
    else:
        _gan_tmp3 = paths
    roots = _gan_tmp3
    def _gan_fn3(p):
        path = pathlib.Path(p)
        if _gan_truthy(path.is_dir()):
            return gandora_std.enum.sort(gandora_std.enum.map(builtins.list(path.rglob("*.gan")), lambda f: str(f)))
        elif _gan_truthy(gandora_std.string.ends_with_p(p, ".gan")):
            return [p]
        else:
            return []
    return gandora_std.enum.flat_map(roots, _gan_fn3)


def _format_file(file, check):
    text = pathlib.Path(file).read_text()
    _gan_case4 = format_text(text)
    match _gan_case4:
        case ("ok", new) as _gan_t5 if isinstance(_gan_t5, tuple):
            if new == text:
                return (file, "unchanged")
            elif _gan_truthy(check):
                return (file, "changed")
            else:
                _gan_case6 = verify(text, new)
                match _gan_case6:
                    case "ok":
                        pathlib.Path(file).write_text(new)
                        print(f"reformatted {file}")
                        return (file, "changed")
                    case ("error", why) as _gan_t7 if isinstance(_gan_t7, tuple):
                        print(f"gan fmt: internal error on {file}: {why} (GEP-0016-R006); file left unchanged")
                        return (file, "error")
                    case _:
                        raise GanMatchError("no case clause matched: " + repr(_gan_case6))
        case ("skip", why) as _gan_t8 if isinstance(_gan_t8, tuple):
            print(f"gan fmt: skipped {file}: {why}")
            return (file, "unchanged")
        case _:
            raise GanMatchError("no case clause matched: " + repr(_gan_case4))


def verify(old_text, new_text):
    if _comments_of(old_text) != _comments_of(new_text):
        return ("error", "comment sequence changed")
    elif _strip_meta(core.parse(old_text)) != _strip_meta(core.parse(new_text)):
        return ("error", "parsed terms changed")
    else:
        return "ok"


def _comments_of(text):
    return gandora_std.enum.map(gandora_std.enum.filter(core.tokens(text), lambda t: gandora_std.map.get(t, "kind") == "comment"), lambda t: gandora_std.map.get(t, "value").rstrip())


def _strip_meta(t):
    if _gan_truthy(_gan_and(builtins.isinstance(t, builtins.tuple), lambda: builtins.len(t) == 3)):
        return (_strip_meta(t[0]), _strip_meta(t[2]))
    elif _gan_truthy(builtins.isinstance(t, builtins.tuple)):
        return builtins.tuple(gandora_std.enum.map(builtins.list(t), lambda x: _strip_meta(x)))
    elif _gan_truthy(builtins.isinstance(t, builtins.list)):
        return gandora_std.enum.map(t, lambda x: _strip_meta(x))
    else:
        return t


def format_text(text):
    try:
        core.parse(text)
        _gan_tmp9 = "ok"
    except core.CompileError as _e:
        _gan_tmp9 = "parse_error"
    parsed = _gan_tmp9
    if parsed == "parse_error":
        return ("skip", "does not parse")
    else:
        return ("ok", _reflow(text))


def _reflow(text):
    toks = gandora_std.enum.reject(core.tokens(text), lambda t: (gandora_std.map.get(t, "kind") == "eof") or (gandora_std.map.get(t, "kind") == "newline"))
    lines = text.split("\n")
    interior = _interior_lines(toks)
    buckets = _bucket_by_line(toks)
    n = gandora_std.enum.count(lines)
    _gan_loop11 = (1, [], "blank_start", None, False, {}, [])
    _gan_res12 = None
    while True:
        match _gan_loop11:
            case (i, stack, prev_kind, prev_last, prev_hang, deltas, acc) as _gan_t13 if isinstance(_gan_t13, tuple):
                pass
            case _:
                raise GanMatchError("loop state did not match: " + repr(_gan_loop11))
        if i > n:
            _gan_res12 = gandora_std.enum.reverse(acc)
            break
        else:
            line = gandora_std.enum.at(lines, i - 1)
            lt = gandora_std.map.get(buckets, i, [])
            if _gan_truthy(gandora_std.map.has_key_p(interior, i)):
                d = gandora_std.map.get(deltas, gandora_std.map.get(interior, i), 0)
                _gan_loop11 = (i + 1, stack, "code", prev_last, prev_hang, deltas, [_shift(line, d)] + acc)
                continue
            elif _gan_truthy(gandora_std.enum.empty_p(lt)):
                if (prev_kind == "blank") or (prev_kind == "blank_start"):
                    _gan_loop11 = (i + 1, stack, prev_kind, prev_last, prev_hang, deltas, acc)
                    continue
                else:
                    _gan_loop11 = (i + 1, stack, "blank", prev_last, prev_hang, deltas, [""] + acc)
                    continue
            else:
                _gan_val14 = _line_level(lt, stack, prev_last, prev_hang)
                match _gan_val14:
                    case (level, stack2, hung) as _gan_t15 if isinstance(_gan_t15, tuple):
                        pass
                    case _:
                        raise GanMatchError("no match of right-hand side value: " + repr(_gan_val14))
                indent = gandora_std.string.duplicate("  ", level)
                multi = gandora_std.enum.find(lt, lambda t: _multiline_p(t))
                if _gan_truthy((multi is None)):
                    _gan_tmp16 = indent + _render_tokens(_capture_parens(lt), lines)
                else:
                    _gan_tmp16 = indent + line.strip()
                rendered = _gan_tmp16
                if _gan_truthy((multi is None)):
                    _gan_tmp17 = deltas
                else:
                    old_lead = builtins.len(line) - builtins.len(line.lstrip())
                    _gan_tmp17 = gandora_std.map.put(deltas, i, builtins.len(indent) - old_lead)
                deltas2 = _gan_tmp17
                stack3 = _advance(lt, stack2, level)
                _gan_loop11 = (i + 1, stack3, "code", _last_significant(lt), hung, deltas2, [rendered] + acc)
                continue
        break
    _gan_tmp10 = _gan_res12
    out = _gan_tmp10
    return gandora_std.enum.join(_trim_trailing_blanks(out), "\n") + "\n"


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
    return (gandora_std.map.get(t, "kind") == "str") or (gandora_std.map.get(t, "kind") == "sigil") and (gandora_std.map.get(t, "end_line") > gandora_std.map.get(t, "line"))


def _interior_lines(toks):
    def _gan_fn4(t, acc):
        if _gan_truthy(_multiline_p(t)):
            return gandora_std.enum.reduce(builtins.list(builtins.range(gandora_std.map.get(t, "line") + 1, gandora_std.map.get(t, "end_line") + 1)), acc, lambda l, a: gandora_std.map.put(a, l, gandora_std.map.get(t, "line")))
        else:
            return acc
    return gandora_std.enum.reduce(toks, {}, _gan_fn4)


def _bucket_by_line(toks):
    def _gan_fn5(t, acc):
        l = gandora_std.map.get(t, "line")
        return gandora_std.map.put(acc, l, gandora_std.map.get(acc, l, []) + [t])
    return gandora_std.enum.reduce(toks, {}, _gan_fn5)


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
    def _gan_fn6(*_gan_args):
        match _gan_args:
            case (t, (depth, found) as _gan_t19,) if isinstance(_gan_t19, tuple):
                if _gan_truthy(_opener_tok_p(t)):
                    return (depth + 1, found)
                elif _gan_truthy(_closer_p(t)):
                    return (gandora_std.enum.max([depth - 1, 0]), found)
                elif _gan_truthy(_gan_and(_tok_is(t, "op", "->"), lambda: depth == 0)):
                    return (depth, True)
                else:
                    return (depth, found)
        raise GanMatchError("no clause of _gan_fn6/2 matched " + repr(_gan_args))
    r = gandora_std.enum.reduce(lt, (0, False), _gan_fn6)
    return r[1]


def _ends_with_arrow_p(lt):
    last = _last_significant(lt)
    return _gan_and(not (_gan_truthy((last is None))), lambda: _tok_is(last, "op", "->"))


def _hang_p(first, prev_last, prev_hang, in_paren):
    starts = _gan_and(gandora_std.enum.member_p(["op", "kw"], gandora_std.map.get(first, "kind")), lambda: gandora_std.enum.member_p(hang_starters, gandora_std.map.get(first, "value")))
    ends = _gan_and(_gan_and(not (_gan_truthy((prev_last is None))), lambda: gandora_std.enum.member_p(["op", "kw"], gandora_std.map.get(prev_last, "kind"))), lambda: gandora_std.enum.member_p(hang_enders, gandora_std.map.get(prev_last, "value")))
    comma = _gan_and(not (_gan_truthy(in_paren)) and not (_gan_truthy((prev_last is None))), lambda: _tok_is(prev_last, "op", ","))
    continues = _gan_and(_tok_is(first, "op", "|>"), lambda: prev_hang)
    return _gan_or(_gan_or(_gan_or(starts, lambda: ends), lambda: comma), lambda: continues)


def _line_level(lt, stack, prev_last, prev_hang):
    first = gandora_std.enum.at(lt, 0)
    _gan_loop21 = stack
    _gan_res22 = None
    while True:
        s = _gan_loop21
        if _gan_truthy(gandora_std.enum.empty_p(s)):
            _gan_res22 = s
            break
        else:
            top = gandora_std.enum.at(s, 0)
            if _gan_truthy(_gan_and(gandora_std.map.get(top, "kind") == "clause", lambda: _gan_or(_gan_or(_closer_p(first), lambda: _mid_p(first)), lambda: _clause_head_p(lt)))):
                _gan_loop21 = gandora_std.enum.drop(s, 1)
                continue
            else:
                _gan_res22 = s
                break
        break
    _gan_tmp20 = _gan_res22
    popped = _gan_tmp20
    top = gandora_std.enum.at(popped, 0)
    if _gan_truthy(gandora_std.enum.empty_p(popped)):
        _gan_tmp23 = 0
    elif _gan_truthy(_closer_p(first)):
        _gan_tmp23 = gandora_std.map.get(top, "open")
    elif _gan_truthy(_mid_p(first)):
        _gan_tmp23 = gandora_std.map.get(top, "open")
    else:
        _gan_tmp23 = gandora_std.map.get(top, "body")
    level = _gan_tmp23
    top_paren = not (_gan_truthy(gandora_std.enum.empty_p(popped))) and (gandora_std.map.get(gandora_std.enum.at(popped, 0), "kind") == "paren")
    hung = _gan_and(not (_gan_truthy(_closer_p(first))) and not (_gan_truthy(_mid_p(first))), lambda: _hang_p(first, prev_last, prev_hang, top_paren))
    if _gan_truthy(hung):
        _gan_tmp24 = level + 1
    else:
        _gan_tmp24 = level
    level = _gan_tmp24
    return (gandora_std.enum.max([level, 0]), popped, hung)


def _advance(lt, stack, level):
    def _gan_fn7(t, s):
        if _gan_truthy(_opener_tok_p(t)):
            return [{"kind": _struct_kind(t), "body": level + 1, "open": level}] + s
        elif _gan_truthy(_closer_p(t)):
            return _pop_to_opener(s)
        else:
            return s
    stack2 = gandora_std.enum.reduce(lt, stack, _gan_fn7)
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
    _gan_case26 = s
    match _gan_case26:
        case [] as _gan_l27 if isinstance(_gan_l27, list):
            return []
        case [{"kind": "clause"}, *rest] as _gan_l28 if isinstance(_gan_l28, list):
            return _pop_to_opener(rest)
        case [_, *rest] as _gan_l29 if isinstance(_gan_l29, list):
            return rest
        case _:
            raise GanMatchError("no case clause matched: " + repr(_gan_case26))


def _render_tokens(lt, lines):
    _gan_loop30 = (lt, "", None)
    _gan_res31 = None
    while True:
        match _gan_loop30:
            case (toks, out, prev) as _gan_t32 if isinstance(_gan_t32, tuple):
                pass
            case _:
                raise GanMatchError("loop state did not match: " + repr(_gan_loop30))
        _gan_case33 = toks
        match _gan_case33:
            case [] as _gan_l34 if isinstance(_gan_l34, list):
                _gan_res31 = out
                break
            case [t, *rest] as _gan_l35 if isinstance(_gan_l35, list):
                txt = _tok_text(t, lines)
                if _gan_truthy((prev is None)):
                    _gan_tmp36 = txt
                elif _gap(prev, t) == 0:
                    _gan_tmp36 = txt
                else:
                    _gan_tmp36 = " " + txt
                piece = _gan_tmp36
                _gan_loop30 = (rest, out + piece, t)
                continue
            case _:
                raise GanMatchError("no case clause matched: " + repr(_gan_case33))
        break
    return _gan_res31


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
    _gan_loop37 = (lt, [])
    _gan_res38 = None
    while True:
        match _gan_loop37:
            case (toks, out) as _gan_t39 if isinstance(_gan_t39, tuple):
                pass
            case _:
                raise GanMatchError("loop state did not match: " + repr(_gan_loop37))
        _gan_case40 = toks
        match _gan_case40:
            case [] as _gan_l41 if isinstance(_gan_l41, list):
                _gan_res38 = gandora_std.enum.reverse(out)
                break
            case [t, *rest] as _gan_l42 if isinstance(_gan_l42, list):
                if _gan_truthy(_gan_and(_gan_and(_tok_is(t, "op", "&"), lambda: not (_gan_truthy(gandora_std.enum.empty_p(rest)))), lambda: gandora_std.map.get(gandora_std.enum.at(rest, 0), "kind") == "pyref")):
                    _gan_case43 = _split_capture(rest)
                    match _gan_case43:
                        case (body, tail) as _gan_t44 if isinstance(_gan_t44, tuple):
                            open = _synth("(", t)
                            close = _synth(")", gandora_std.enum.at(body, -1))
                            _gan_loop37 = (tail, [close] + (gandora_std.enum.reverse(body) + ([open, t] + out)))
                            continue
                        case None:
                            _gan_loop37 = (rest, [t] + out)
                            continue
                        case _:
                            raise GanMatchError("no case clause matched: " + repr(_gan_case43))
                else:
                    _gan_loop37 = (rest, [t] + out)
                    continue
            case _:
                raise GanMatchError("no case clause matched: " + repr(_gan_case40))
        break
    return _gan_res38


def _split_capture(toks):
    _gan_loop45 = (toks, [], "ref")
    _gan_res46 = None
    while True:
        match _gan_loop45:
            case (left, taken, state) as _gan_t47 if isinstance(_gan_t47, tuple):
                pass
            case _:
                raise GanMatchError("loop state did not match: " + repr(_gan_loop45))
        _gan_case48 = (state, left)
        match _gan_case48:
            case ("ref", [t, *rest] as _gan_l49) as _gan_t50 if isinstance(_gan_l49, list) and isinstance(_gan_t50, tuple):
                if gandora_std.map.get(t, "kind") == "pyref":
                    _gan_loop45 = (rest, [t] + taken, "dot")
                    continue
                else:
                    _gan_res46 = None
                    break
            case ("dot", [t, *rest] as _gan_l51) as _gan_t52 if isinstance(_gan_l51, list) and isinstance(_gan_t52, tuple):
                if _gan_truthy(_tok_is(t, "op", ".")):
                    _gan_loop45 = (rest, [t] + taken, "name")
                    continue
                elif _gan_truthy(_tok_is(t, "op", "/")):
                    _gan_loop45 = (rest, [t] + taken, "arity")
                    continue
                else:
                    _gan_res46 = None
                    break
            case ("name", [t, *rest] as _gan_l53) as _gan_t54 if isinstance(_gan_l53, list) and isinstance(_gan_t54, tuple):
                if (gandora_std.map.get(t, "kind") == "ident") or (gandora_std.map.get(t, "kind") == "upident"):
                    _gan_loop45 = (rest, [t] + taken, "dot")
                    continue
                else:
                    _gan_res46 = None
                    break
            case ("arity", [t, *rest] as _gan_l55) as _gan_t56 if isinstance(_gan_l55, list) and isinstance(_gan_t56, tuple):
                if gandora_std.map.get(t, "kind") == "int":
                    _gan_res46 = (gandora_std.enum.reverse([t] + taken), rest)
                    break
                else:
                    _gan_res46 = None
                    break
            case (_, [] as _gan_l57) as _gan_t58 if isinstance(_gan_l57, list) and isinstance(_gan_t58, tuple):
                _gan_res46 = None
                break
            case _:
                raise GanMatchError("no case clause matched: " + repr(_gan_case48))
        break
    return _gan_res46


def _synth(op, near):
    return {"kind": "op", "value": op, "synthetic": True, "line": gandora_std.map.get(near, "line"), "col": gandora_std.map.get(near, "end_col"), "end_line": gandora_std.map.get(near, "line"), "end_col": gandora_std.map.get(near, "end_col")}
