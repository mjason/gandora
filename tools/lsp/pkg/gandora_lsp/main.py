"""gan-lsp: the Gandora language server on pygls (GEP-0015).
pygls owns protocol machinery; this module owns language logic:
push diagnostics on every edit, documentation hover (signatures,
translations, examples, `$module` references, language constructs),
go-to-definition, document symbols, `Module.` completion, and
whole-document formatting through the GEP-0016 engine.
"""

import builtins
import collections.abc
import gandora_core as core
import importlib.util
import lsprotocol.types
import lsprotocol.types as types
import os
import pathlib
import pygls.lsp.server
import re
import gandora_lsp.construct_docs
import gandora_lsp.py_intel
import gandora_std.enum
import gandora_std.map
import gandora_std.string
import gandora_tool.fmt


def _gan_truthy(value):
    return value is not None and value is not False

def _gan_and(value, then):
    return then() if _gan_truthy(value) else value

def _gan_or(value, then):
    return value if _gan_truthy(value) else then()

class GanMatchError(Exception):
    pass

server = pygls.lsp.server.LanguageServer("gan-lsp", "0.18.7", text_document_sync_kind=types.TextDocumentSyncKind.Full)

word_re = re.compile("[A-Za-z_][A-Za-z0-9_.!?]*")


@server.feature(types.TEXT_DOCUMENT_DID_OPEN)
def did_open(params: lsprotocol.types.DidOpenTextDocumentParams) -> object:
    """Publishes diagnostics for a document on open."""
    return _publish(params.text_document.uri, params.text_document.text)


@server.feature(types.TEXT_DOCUMENT_DID_CHANGE)
def did_change(params: lsprotocol.types.DidChangeTextDocumentParams) -> object:
    """Re-publishes diagnostics on every edit (full sync)."""
    text = gandora_std.enum.at(builtins.list(params.content_changes), -1).text
    return _publish(params.text_document.uri, text)


@server.feature(types.TEXT_DOCUMENT_DID_CLOSE)
def did_close(params: lsprotocol.types.DidCloseTextDocumentParams) -> object:
    """Clears a document's diagnostics on close."""
    return _send_diagnostics(params.text_document.uri, [])


def _send_diagnostics(uri, items):
    return server.text_document_publish_diagnostics(types.PublishDiagnosticsParams(uri=uri, diagnostics=items))


def _publish(uri, text):
    path = gandora_std.string.replace(uri, "file://", "")
    root = server.workspace.root_path
    try:
        _gan_tmp0 = core.diagnostics(text, path, root)
    except Exception as _e:
        _gan_tmp0 = []
    diags = _gan_tmp0
    def _gan_fn0(d):
        line = gandora_std.enum.max([gandora_std.map.get(d, "line") - 1, 0])
        col = gandora_std.enum.max([gandora_std.map.get(d, "col") - 1, 0])
        pos = types.Position(line=line, character=col)
        return types.Diagnostic(range=types.Range(pos, pos), message=gandora_std.map.get(d, "message"), severity=_severity(gandora_std.map.get(d, "severity")), source="gan")
    lsp = gandora_std.enum.map(diags, _gan_fn0)
    return _send_diagnostics(uri, lsp)


def _severity(*_gan_args):
    match _gan_args:
        case ("error",):
            return types.DiagnosticSeverity.Error
        case (_,):
            return types.DiagnosticSeverity.Warning
    raise GanMatchError("no clause of severity/1 matched " + repr(_gan_args))


def _doc_and_line(params):
    doc = server.workspace.get_text_document(params.text_document.uri)
    line = gandora_std.enum.at(doc.source.split("\n"), params.position.line)
    return (doc, line)


def _word_at(line: str, character: int) -> tuple[str, int] | None:
    def _gan_fn1(m, *, character=character):
        _gan_val1 = m.span()
        match _gan_val1:
            case (s, e) as _gan_t2 if isinstance(_gan_t2, tuple):
                pass
            case _:
                raise GanMatchError("no match of right-hand side value: " + repr(_gan_val1))
        return (s <= character) and (character < e)
    hit = gandora_std.enum.find(word_re.finditer(line), _gan_fn1)
    if (hit is None):
        return None
    else:
        _gan_val3 = hit.span()
        match _gan_val3:
            case (s, _e) as _gan_t4 if isinstance(_gan_t4, tuple):
                pass
            case _:
                raise GanMatchError("no match of right-hand side value: " + repr(_gan_val3))
        return (hit.group(0).rstrip("."), s)


def _module_of(source: str) -> str | None:
    m = re.compile("defmodule\\s+([A-Za-z0-9_.]+)").search(source)
    if (m is None):
        return None
    else:
        return m.group(1)


def _target_of(source: str, token: str) -> str | None:
    if _gan_truthy(gandora_std.string.match_p(token, re.compile("^[A-Z]"))):
        return token
    else:
        mod = _module_of(source)
        if (mod is None):
            return None
        else:
            return f"{mod}.{token}"


@server.feature(types.TEXT_DOCUMENT_HOVER)
def hover(params: lsprotocol.types.HoverParams) -> object:
    """Documentation hover for the reference under the cursor (GEP-0015-R005)."""
    _gan_val5 = _doc_and_line(params)
    match _gan_val5:
        case (doc, line) as _gan_t6 if isinstance(_gan_t6, tuple):
            pass
        case _:
            raise GanMatchError("no match of right-hand side value: " + repr(_gan_val5))
    hit = _word_at(line, params.position.character)
    if (hit is None):
        return None
    else:
        _gan_val7 = hit
        match _gan_val7:
            case (token, start) as _gan_t8 if isinstance(_gan_t8, tuple):
                pass
            case _:
                raise GanMatchError("no match of right-hand side value: " + repr(_gan_val7))
        pyref = (start > 0) and (gandora_std.string.slice(line, start - 1, 1) == "$")
        bounded = (start > 1) and (gandora_std.string.slice(line, start - 2, 2) == "$(")
        postfix = (start > 0) and (gandora_std.string.slice(line, start - 1, 1) == ".") and not (_gan_truthy(pyref))
        if _gan_truthy(bounded):
            _gan_tmp9 = ("import " + token, token)
        else:
            _gan_tmp9 = gandora_lsp.py_intel.target(doc.source, token, pyref)
        py = _gan_tmp9
        if _gan_truthy(_gan_and(postfix, lambda: (py is None))):
            return _markdown_hover(f"`.{token}()` — calls the Python method `{token}` on the value " + f"to its left; `|> .{token}()` pipes into it (GEP-0001 postfix chain).")
        elif not ((py is None)):
            _gan_val10 = py
            match _gan_val10:
                case (import_line, expr) as _gan_t11 if isinstance(_gan_t11, tuple):
                    pass
                case _:
                    raise GanMatchError("no match of right-hand side value: " + repr(_gan_val10))
            md = gandora_lsp.py_intel.hover_markdown(_root_path(), import_line, expr)
            if not ((md is None)):
                return _markdown_hover(md)
            elif _gan_truthy(pyref):
                return _pyref_hover(token)
            else:
                return None
        elif not ((gandora_lsp.construct_docs.card(token) is None)):
            return _markdown_hover(f"`{token}`\n\n{gandora_lsp.construct_docs.card(token)}")
        else:
            target = _target_of(doc.source, token)
            info = _lookup_doc(target)
            h = _render_hover(target, info)
            if _gan_truthy(_gan_and((h is None), lambda: gandora_std.string.match_p(token, re.compile("^[a-z_]")))):
                _gan_tmp12 = _imported_hover(doc.source, token)
            else:
                _gan_tmp12 = h
            h = _gan_tmp12
            if _gan_truthy(_gan_and((h is None), lambda: gandora_std.string.match_p(token, re.compile("^[a-z_]")))):
                return _var_hover(doc, token, params.position.line)
            else:
                return h


def _imported_hover(source, token):
    mods = gandora_std.enum.uniq(re.compile("(?m)^\\s*(?:import|require|use)\\s+([A-Z][A-Za-z0-9_.]*)").findall(source))
    def _gan_fn2(mod, acc, *, token=token):
        if (acc is None):
            return _render_hover(f"{mod}.{token}", _lookup_doc(f"{mod}.{token}"))
        else:
            return acc
    return gandora_std.enum.reduce(mods, None, _gan_fn2)


def _var_hover(doc, token, cursor_line):
    mod = _module_of(doc.source)
    try:
        _gan_tmp14 = core.compile_string(doc.source, gandora_std.string.replace(doc.uri, "file://", ""), _root_path())
    except Exception as _e:
        _gan_tmp14 = None
    compiled = _gan_tmp14
    if (compiled is None) or (mod is None):
        return None
    else:
        try:
            _gan_tmp15 = core.symbols(mod, _root_path())
        except Exception as _e:
            _gan_tmp15 = []
        syms = _gan_tmp15
        holder = gandora_std.enum.at(gandora_std.enum.filter(syms, lambda s, *, cursor_line=cursor_line: gandora_std.map.get(s, "line") <= (cursor_line + 1)), -1)
        if (holder is None):
            return None
        else:
            base = py_name(gandora_std.map.get(holder, "name"))
            py_var = py_name(token)
            t = gandora_lsp.py_intel.infer_type(compiled, [base, "_" + base], py_var)
            if (t is None) or (t == py_var) or (t == token):
                return None
            else:
                return _markdown_hover(f"`{token}`: `{t}` *(inferred)*")


def py_name(name: str) -> str:
    """The identifier a Gandora name compiles to in generated Python.

## Parameters

  - name: The Gandora function or variable name.

    >>> py_name("valid?")
    'valid_p'
"""
    return name.replace("?", "_p").replace("!", "_bang")


def _root_path() -> str:
    return server.workspace.root_path


def _lookup_doc(target):
    if (target is None):
        return None
    else:
        try:
            return core.doc(target, server.workspace.root_path)
        except Exception as _e:
            return None


def _pyref_hover(token):
    mod = gandora_std.enum.at(token.split("."), 0)
    try:
        _gan_tmp16 = importlib.util.find_spec(mod)
    except Exception as _e:
        _gan_tmp16 = None
    spec = _gan_tmp16
    if (spec is None):
        return _markdown_hover(f"`${mod}` — Python module (not found in this environment)")
    else:
        return _markdown_hover(f"`${mod}` — Python module\n\n`{spec.origin}`")


def _markdown_hover(md):
    return types.Hover(contents=types.MarkupContent(kind=types.MarkupKind.Markdown, value=md))


def _doc_locale() -> str:
    try:
        _gan_tmp17 = core.local_pref(_root_path(), "docLocale")
    except Exception as _e:
        _gan_tmp17 = None
    local = _gan_tmp17
    if (local is None):
        return os.environ.get("GAN_DOC_LOCALE", "default")
    else:
        return local


def _locale_section(info: collections.abc.Mapping[str, object], locale: str, heading: str) -> str:
    prose = gandora_std.map.get(gandora_std.map.get(info, "entries", {}), locale)
    if (prose is None) and (locale != "default"):
        _gan_tmp18 = None
    else:
        _gan_tmp18 = prose
    prose = _gan_tmp18
    parts = [heading, prose] + _param_lines(info, locale)
    return gandora_std.enum.join(gandora_std.enum.reject(parts, lambda p: (p is None) or (p == "")), "\n\n")


def _render_hover(target, info):
    if _gan_truthy(_gan_or((info is None), lambda: info.get("hidden"))):
        return None
    else:
        _gan_case20 = gandora_std.map.get(info, "tco")
        match _gan_case20:
            case "loop":
                _gan_tmp19 = ["♻ *tail recursion → compiles to a `while` loop (constant stack)*"]
            case "stack":
                _gan_tmp19 = ["⚠ *self-recursive → native call stack (not tail position)*"]
            case _:
                _gan_tmp19 = []
        tco = _gan_tmp19
        sigs = gandora_std.enum.join(gandora_std.enum.uniq(gandora_std.map.get(info, "signatures", [])), "\n")
        if sigs == "":
            _gan_tmp21 = f"### {target}"
        else:
            _gan_tmp21 = f"```elixir\n{sigs}\n```"
        header = _gan_tmp21
        entries = gandora_std.map.get(info, "entries", {})
        pref = _doc_locale()
        locales = gandora_std.enum.sort(gandora_std.enum.filter(entries.keys(), lambda loc: loc != "default"))
        if (pref == "all") or (pref == ""):
            _gan_tmp22 = [_locale_section(info, "default", "")] + gandora_std.enum.map(locales, lambda loc, *, info=info: f"---\n\n**{loc}**\n\n" + _locale_section(info, loc, ""))
        else:
            main = _locale_section(info, pref, "")
            if gandora_std.string.trim(main) == "":
                _gan_tmp22 = [_locale_section(info, "default", "")]
            else:
                _gan_tmp22 = [main]
        sections = _gan_tmp22
        def _gan_fn3(*_gan_args):
            match _gan_args:
                case ((k, v) as _gan_t23,) if isinstance(_gan_t23, tuple):
                    return f"`{k}`: {v}"
            raise GanMatchError("no clause of _gan_fn3/1 matched " + repr(_gan_args))
        meta = gandora_std.enum.map(gandora_std.map.get(info, "meta", []), _gan_fn3)
        examples = gandora_std.enum.map(gandora_std.map.get(info, "examples", []), lambda ex: f"```elixir\n{ex.strip()}\n```")
        body = gandora_std.enum.join(gandora_std.enum.reject(tco + (sections + (meta + examples)), lambda part: (part is None) or (gandora_std.string.trim(part) == "")), "\n\n")
        if (gandora_std.string.trim(body) == "") and (sigs == ""):
            return None
        else:
            return _markdown_hover(f"{header}\n\n{body}")


def _param_lines(info: collections.abc.Mapping[str, object], locale: str) -> list[str]:
    entries = gandora_std.map.get(info, "params", [])
    if _gan_truthy(gandora_std.enum.empty_p(entries)):
        return []
    else:
        if _gan_truthy(gandora_std.string.starts_with_p(locale, "zh")):
            _gan_tmp24 = "**参数**"
        else:
            _gan_tmp24 = "**Parameters**"
        heading = _gan_tmp24
        def _gan_fn4(p, *, locale=locale):
            texts = gandora_std.map.get(p, "entries", {})
            text = gandora_std.map.get(texts, locale, gandora_std.map.get(texts, "default", ""))
            _gan_fstr25 = gandora_std.map.get(p, "name")
            return f"- `{_gan_fstr25}` — {text}"
        lines = gandora_std.enum.join(gandora_std.enum.map(entries, _gan_fn4), "\n")
        return [heading + ("\n\n" + lines)]


def _param_doc(info, label):
    name = (lambda s: s.rstrip(","))(builtins.str(gandora_std.enum.at(label.split(" "), 0)))
    entry = gandora_std.enum.find(gandora_std.map.get(info, "params", []), lambda p, *, name=name: gandora_std.map.get(p, "name") == name)
    if (entry is None):
        return None
    else:
        texts = gandora_std.map.get(entry, "entries", {})
        pref = _doc_locale()
        return gandora_std.map.get(texts, pref, gandora_std.map.get(texts, "default"))


@server.feature(types.TEXT_DOCUMENT_DEFINITION)
def definition(params: lsprotocol.types.DefinitionParams) -> object:
    """Go-to-definition for Gandora and Python references (GEP-0015-R006)."""
    _gan_val26 = _doc_and_line(params)
    match _gan_val26:
        case (doc, line) as _gan_t27 if isinstance(_gan_t27, tuple):
            pass
        case _:
            raise GanMatchError("no match of right-hand side value: " + repr(_gan_val26))
    hit = _word_at(line, params.position.character)
    if (hit is None):
        return None
    else:
        _gan_val28 = hit
        match _gan_val28:
            case (token, start) as _gan_t29 if isinstance(_gan_t29, tuple):
                pass
            case _:
                raise GanMatchError("no match of right-hand side value: " + repr(_gan_val28))
        pyref = (start > 0) and (gandora_std.string.slice(line, start - 1, 1) == "$")
        bounded = (start > 1) and (gandora_std.string.slice(line, start - 2, 2) == "$(")
        if _gan_truthy(bounded):
            _gan_tmp30 = ("import " + token, token)
        else:
            _gan_tmp30 = gandora_lsp.py_intel.target(doc.source, token, pyref)
        py = _gan_tmp30
        if not ((py is None)):
            _gan_val31 = py
            match _gan_val31:
                case (import_line, expr) as _gan_t32 if isinstance(_gan_t32, tuple):
                    pass
                case _:
                    raise GanMatchError("no match of right-hand side value: " + repr(_gan_val31))
            loc = gandora_lsp.py_intel.goto(_root_path(), import_line, expr)
            if (loc is None):
                return None
            else:
                pos = types.Position(line=gandora_std.map.get(loc, "line0"), character=gandora_std.map.get(loc, "col0"))
                return types.Location(uri="file://" + gandora_std.map.get(loc, "path"), range=types.Range(pos, pos))
        else:
            target = _target_of(doc.source, token)
            if (target is None):
                _gan_tmp33 = None
            else:
                try:
                    _gan_tmp33 = core.definition(target, _root_path())
                except Exception as _e:
                    _gan_tmp33 = None
            loc = _gan_tmp33
            if (loc is None):
                return None
            else:
                pos = types.Position(line=gandora_std.enum.max([gandora_std.map.get(loc, "line") - 1, 0]), character=gandora_std.enum.max([gandora_std.map.get(loc, "col") - 1, 0]))
                return types.Location(uri="file://" + gandora_std.map.get(loc, "path"), range=types.Range(pos, pos))


@server.feature(types.TEXT_DOCUMENT_DOCUMENT_SYMBOL)
def document_symbol(params: lsprotocol.types.DocumentSymbolParams) -> object:
    """The document's module and definitions as an outline (GEP-0015-R008)."""
    doc = server.workspace.get_text_document(params.text_document.uri)
    mod = _module_of(doc.source)
    if (mod is None):
        _gan_tmp34 = []
    else:
        try:
            _gan_tmp34 = core.symbols(mod, server.workspace.root_path)
        except Exception as _e:
            _gan_tmp34 = []
    syms = _gan_tmp34
    def _gan_fn5(s):
        line = gandora_std.enum.max([gandora_std.map.get(s, "line") - 1, 0])
        pos = types.Position(line=line, character=0)
        rng = types.Range(pos, pos)
        return types.DocumentSymbol(name=gandora_std.map.get(s, "name"), detail=gandora_std.map.get(s, "head"), kind=_symbol_kind(gandora_std.map.get(s, "kind")), range=rng, selection_range=rng)
    children = gandora_std.enum.map(syms, _gan_fn5)
    if (mod is None):
        return []
    else:
        zero = types.Position(line=0, character=0)
        return [types.DocumentSymbol(name=mod, kind=types.SymbolKind.Module, range=types.Range(zero, zero), selection_range=types.Range(zero, zero), children=children)]


def _symbol_kind(*_gan_args):
    match _gan_args:
        case ("defmacro",):
            return types.SymbolKind.Constructor
        case ("defp",):
            return types.SymbolKind.Method
        case (_,):
            return types.SymbolKind.Function
    raise GanMatchError("no clause of symbol_kind/1 matched " + repr(_gan_args))


def _fun_target_at(doc: object, line: str, character: int) -> str | None:
    hit = _word_at(line, character)
    if (hit is None):
        return None
    else:
        _gan_val35 = hit
        match _gan_val35:
            case (token, _start) as _gan_t36 if isinstance(_gan_t36, tuple):
                pass
            case _:
                raise GanMatchError("no match of right-hand side value: " + repr(_gan_val35))
        last = gandora_std.enum.at(token.split("."), -1)
        if _gan_truthy(gandora_std.string.match_p(last, re.compile("^[a-z_]"))):
            return _target_of(doc.source, token)
        else:
            return None


def _ref_ranges(target: str) -> list[dict]:
    fun = gandora_std.enum.at(target.split("."), -1)
    try:
        _gan_tmp37 = core.references(target, _root_path())
    except Exception as _e:
        _gan_tmp37 = []
    refs = _gan_tmp37
    def _gan_fn6(r, *, fun=fun):
        path = gandora_std.map.get(r, "path")
        line0 = gandora_std.enum.max([gandora_std.map.get(r, "line") - 1, 0])
        col0 = gandora_std.enum.max([gandora_std.map.get(r, "col") - 1, 0])
        try:
            _gan_tmp38 = pathlib.Path(path).read_text(encoding="utf-8")
        except Exception as _e:
            _gan_tmp38 = ""
        text = _gan_tmp38
        l = gandora_std.enum.at(text.split("\n"), line0)
        if (l is None):
            _gan_tmp39 = ""
        else:
            _gan_tmp39 = l
        l = _gan_tmp39
        idx = l.find(fun, col0)
        if idx < 0:
            _gan_tmp40 = l.find(fun)
        else:
            _gan_tmp40 = idx
        idx = _gan_tmp40
        if idx < 0:
            _gan_tmp41 = col0
        else:
            _gan_tmp41 = idx
        start = _gan_tmp41
        return {"path": path, "line": line0, "start": start, "stop": start + gandora_std.string.length(fun), "is_def": gandora_std.map.get(r, "is_def")}
    return gandora_std.enum.map(refs, _gan_fn6)


@server.feature(types.TEXT_DOCUMENT_REFERENCES)
def references(params: lsprotocol.types.ReferenceParams) -> object:
    """Every project reference to the function under the cursor (GEP-0015-R012)."""
    _gan_val42 = _doc_and_line(params)
    match _gan_val42:
        case (doc, line) as _gan_t43 if isinstance(_gan_t43, tuple):
            pass
        case _:
            raise GanMatchError("no match of right-hand side value: " + repr(_gan_val42))
    target = _fun_target_at(doc, line, params.position.character)
    if (target is None):
        return None
    else:
        include_defs = params.context.include_declaration
        return [types.Location(uri="file://" + gandora_std.map.get(r, "path"), range=types.Range(types.Position(line=gandora_std.map.get(r, "line"), character=gandora_std.map.get(r, "start")), types.Position(line=gandora_std.map.get(r, "line"), character=gandora_std.map.get(r, "stop")))) for r in _ref_ranges(target) if _gan_truthy(_gan_or(include_defs, lambda: not (_gan_truthy(gandora_std.map.get(r, "is_def")))))]


@server.feature(types.TEXT_DOCUMENT_RENAME)
def rename(params: lsprotocol.types.RenameParams) -> object:
    """Workspace-wide rename of a project-defined function (GEP-0015-R012)."""
    _gan_val44 = _doc_and_line(params)
    match _gan_val44:
        case (doc, line) as _gan_t45 if isinstance(_gan_t45, tuple):
            pass
        case _:
            raise GanMatchError("no match of right-hand side value: " + repr(_gan_val44))
    target = _fun_target_at(doc, line, params.position.character)
    new_name = params.new_name
    if (target is None):
        return None
    elif not (_gan_truthy(gandora_std.string.match_p(new_name, re.compile("^[a-z_][A-Za-z0-9_]*[?!]?$")))):
        return None
    else:
        ranges = _ref_ranges(target)
        if _gan_truthy(gandora_std.enum.any_p(ranges, lambda r: gandora_std.map.get(r, "is_def"))):
            def _gan_fn7(r, acc, *, new_name=new_name):
                uri = "file://" + gandora_std.map.get(r, "path")
                edit = types.TextEdit(range=types.Range(types.Position(line=gandora_std.map.get(r, "line"), character=gandora_std.map.get(r, "start")), types.Position(line=gandora_std.map.get(r, "line"), character=gandora_std.map.get(r, "stop"))), new_text=new_name)
                return gandora_std.map.put(acc, uri, gandora_std.map.get(acc, uri, []) + [edit])
            edits = gandora_std.enum.reduce(ranges, {}, _gan_fn7)
            return types.WorkspaceEdit(changes=edits)
        else:
            return None


@server.feature(types.WORKSPACE_SYMBOL)
def workspace_symbol(params: lsprotocol.types.WorkspaceSymbolParams) -> object:
    """Project-wide symbol search (GEP-0015-R013)."""
    try:
        _gan_tmp46 = core.wsymbols(params.query, _root_path())
    except Exception as _e:
        _gan_tmp46 = []
    syms = _gan_tmp46
    def _gan_fn8(s):
        line = gandora_std.enum.max([gandora_std.map.get(s, "line") - 1, 0])
        pos = types.Position(line=line, character=0)
        return types.SymbolInformation(name=gandora_std.map.get(s, "module") + ("." + gandora_std.map.get(s, "name")), kind=_symbol_kind(gandora_std.map.get(s, "kind")), location=types.Location(uri="file://" + gandora_std.map.get(s, "path"), range=types.Range(pos, pos)))
    return gandora_std.enum.map(syms, _gan_fn8)


@server.feature(types.TEXT_DOCUMENT_CODE_ACTION, types.CodeActionOptions(code_action_kinds=[types.CodeActionKind.QuickFix]))
def code_action(params: lsprotocol.types.CodeActionParams) -> object:
    """Quick fixes for compiler lints (GEP-0015-R014)."""
    uri = params.text_document.uri
    doc = server.workspace.get_text_document(uri)
    def _gan_fn9(d, *, doc=doc, uri=uri):
        msg = d.message
        if _gan_truthy(gandora_std.string.contains_p(msg, "GEP-0019-R007")):
            return [_allow_action(uri, doc, d, "stack_recursion")]
        elif _gan_truthy(gandora_std.string.contains_p(msg, "GEP-0022-R005")):
            return [_allow_action(uri, doc, d, "unused_function")]
        elif _gan_truthy(gandora_std.string.contains_p(msg, "GEP-0022-R002")):
            return _underscore_actions(uri, doc, d, msg)
        else:
            return []
    return gandora_std.enum.flat_map(builtins.list(params.context.diagnostics), _gan_fn9)


def _allow_action(uri, doc, d, target):
    line0 = d.range.start.line
    l = gandora_std.enum.at(doc.source.split("\n"), line0)
    if (l is None):
        _gan_tmp47 = ""
    else:
        _gan_tmp47 = l
    l = _gan_tmp47
    indent = gandora_std.string.slice(l, 0, gandora_std.string.length(l) - gandora_std.string.length(l.lstrip()))
    pos = types.Position(line=line0, character=0)
    return types.CodeAction(title="Acknowledge with @allow :" + target, kind=types.CodeActionKind.QuickFix, diagnostics=[d], edit=types.WorkspaceEdit(changes={uri: [types.TextEdit(range=types.Range(pos, pos), new_text=indent + ("@allow :" + (target + "\n")))]}))


def _underscore_actions(uri, doc, d, msg):
    m = re.compile("variable ([A-Za-z0-9_?!]+) is bound").search(msg)
    if (m is None):
        return []
    else:
        var = m.group(1)
        line0 = d.range.start.line
        l = gandora_std.enum.at(doc.source.split("\n"), line0)
        if (l is None):
            _gan_tmp48 = ""
        else:
            _gan_tmp48 = l
        l = _gan_tmp48
        hit = re.compile(f"\\b{re.escape(var)}\\b").search(l)
        if (hit is None):
            return []
        else:
            _gan_val49 = hit.span()
            match _gan_val49:
                case (s, e) as _gan_t50 if isinstance(_gan_t50, tuple):
                    pass
                case _:
                    raise GanMatchError("no match of right-hand side value: " + repr(_gan_val49))
            return [types.CodeAction(title="Rename to _" + var, kind=types.CodeActionKind.QuickFix, diagnostics=[d], edit=types.WorkspaceEdit(changes={uri: [types.TextEdit(range=types.Range(types.Position(line=line0, character=s), types.Position(line=line0, character=e)), new_text="_" + var)]}))]


@server.feature(types.TEXT_DOCUMENT_COMPLETION, types.CompletionOptions(trigger_characters=["."]))
def completion(params: lsprotocol.types.CompletionParams) -> object:
    """`Module.` member and Python attribute completion (GEP-0015-R008)."""
    _gan_val51 = _doc_and_line(params)
    match _gan_val51:
        case (doc, line) as _gan_t52 if isinstance(_gan_t52, tuple):
            pass
        case _:
            raise GanMatchError("no match of right-hand side value: " + repr(_gan_val51))
    prefix = gandora_std.string.slice(line, 0, params.position.character)
    pym = re.compile("(\\$?)([A-Za-z_][A-Za-z0-9_.]*)\\.([A-Za-z0-9_]*)$").search(prefix)
    if (pym is None):
        _gan_tmp53 = None
    else:
        _gan_tmp53 = gandora_lsp.py_intel.target(doc.source, pym.group(2) + ("." + pym.group(3)), pym.group(1) == "$")
    py = _gan_tmp53
    if not ((py is None)):
        _gan_val54 = py
        match _gan_val54:
            case (import_line, expr) as _gan_t55 if isinstance(_gan_t55, tuple):
                pass
            case _:
                raise GanMatchError("no match of right-hand side value: " + repr(_gan_val54))
        return gandora_std.enum.map(gandora_lsp.py_intel.complete(_root_path(), import_line, expr), lambda c: types.CompletionItem(label=gandora_std.map.get(c, "name"), kind=_py_completion_kind(gandora_std.map.get(c, "kind"))))
    else:
        return _gandora_completion(prefix)


def _py_completion_kind(*_gan_args):
    match _gan_args:
        case ("function",):
            return types.CompletionItemKind.Function
        case ("class",):
            return types.CompletionItemKind.Class
        case ("module",):
            return types.CompletionItemKind.Module
        case ("instance",):
            return types.CompletionItemKind.Value
        case (_,):
            return types.CompletionItemKind.Field
    raise GanMatchError("no clause of py_completion_kind/1 matched " + repr(_gan_args))


def _gandora_completion(prefix):
    m = re.compile("([A-Z][A-Za-z0-9_]*(?:\\.[A-Z][A-Za-z0-9_]*)*)\\.([a-z_][A-Za-z0-9_]*)?$").search(prefix)
    if (m is None):
        return []
    else:
        mod = m.group(1)
        if (m.group(2) is None):
            _gan_tmp56 = ""
        else:
            _gan_tmp56 = m.group(2)
        partial = _gan_tmp56
        try:
            _gan_tmp57 = core.symbols(mod, server.workspace.root_path)
        except Exception as _e:
            _gan_tmp57 = []
        syms = _gan_tmp57
        def _gan_fn10(s, acc):
            if _gan_truthy(gandora_std.enum.any_p(acc, lambda a, *, s=s: gandora_std.map.get(a, "name") == gandora_std.map.get(s, "name"))):
                return acc
            else:
                return acc + [s]
        return gandora_std.enum.map(gandora_std.enum.reduce(gandora_std.enum.filter(gandora_std.enum.filter(syms, lambda s: gandora_std.map.get(s, "kind") != "defp"), lambda s, *, partial=partial: gandora_std.map.get(s, "name").startswith(partial)), [], _gan_fn10), lambda s: types.CompletionItem(label=gandora_std.map.get(s, "name"), kind=types.CompletionItemKind.Function, detail=gandora_std.map.get(s, "head"), documentation=gandora_std.map.get(s, "doc")))


def _split_walk(*_gan_args):
    while True:
        match _gan_args:
            case ((i, depth, cur, acc) as _gan_t59, inner,) if isinstance(_gan_t59, tuple):
                if i >= gandora_std.string.length(inner):
                    return acc + [cur]
                else:
                    ch = gandora_std.string.slice(inner, i, 1)
                    if (ch == "(") or (ch == "[") or (ch == "{"):
                        _gan_args = ((i + 1, depth + 1, cur + ch, acc), inner)
                        continue
                    elif (ch == ")") or (ch == "]") or (ch == "}"):
                        _gan_args = ((i + 1, depth - 1, cur + ch, acc), inner)
                        continue
                    elif (ch == ",") and (depth == 0):
                        _gan_args = ((i + 1, depth, "", acc + [cur]), inner)
                        continue
                    else:
                        _gan_args = ((i + 1, depth, cur + ch, acc), inner)
                        continue
        raise GanMatchError("no clause of split_walk/2 matched " + repr(_gan_args))


@server.feature(types.TEXT_DOCUMENT_FORMATTING)
def formatting(params: lsprotocol.types.DocumentFormattingParams) -> object:
    """Whole-document formatting through the GEP-0016 engine (GEP-0015-R007)."""
    doc = server.workspace.get_text_document(params.text_document.uri)
    result = gandora_tool.fmt.format_text(doc.source)
    _gan_case60 = result
    match _gan_case60:
        case ("ok", new) as _gan_t61 if isinstance(_gan_t61, tuple):
            if (new == doc.source) or (gandora_tool.fmt.verify(doc.source, new) != "ok"):
                return []
            else:
                line_count = gandora_std.enum.count(doc.source.split("\n"))
                return [types.TextEdit(range=types.Range(types.Position(line=0, character=0), types.Position(line=line_count, character=0)), new_text=new)]
        case _:
            return []


@server.feature(types.TEXT_DOCUMENT_SIGNATURE_HELP, types.SignatureHelpOptions(trigger_characters=["(", ","]))
def signature_help(params: lsprotocol.types.SignatureHelpParams) -> object:
    """Signature help for the innermost open call (GEP-0015-R008)."""
    _gan_val62 = _doc_and_line(params)
    match _gan_val62:
        case (doc, line) as _gan_t63 if isinstance(_gan_t63, tuple):
            pass
        case _:
            raise GanMatchError("no match of right-hand side value: " + repr(_gan_val62))
    prefix = gandora_std.string.slice(line, 0, params.position.character)
    call = _open_call(prefix)
    if (call is None):
        return None
    else:
        _gan_val64 = call
        match _gan_val64:
            case (callee, active) as _gan_t65 if isinstance(_gan_t65, tuple):
                pass
            case _:
                raise GanMatchError("no match of right-hand side value: " + repr(_gan_val64))
        pyref = gandora_std.string.starts_with_p(callee, "$")
        token = callee.lstrip("$")
        py = gandora_lsp.py_intel.target(doc.source, token, pyref)
        if not ((py is None)):
            _gan_val67 = py
            match _gan_val67:
                case (import_line, expr) as _gan_t68 if isinstance(_gan_t68, tuple):
                    pass
                case _:
                    raise GanMatchError("no match of right-hand side value: " + repr(_gan_val67))
            _gan_tmp66 = gandora_std.enum.map(gandora_lsp.py_intel.signatures(_root_path(), import_line, expr), lambda s: _build_signature(gandora_std.map.get(s, "label"), gandora_std.map.get(s, "params"), gandora_std.map.get(s, "doc")))
        else:
            target = _target_of(doc.source, token)
            info = _lookup_doc(target)
            _gan_tmp66 = _gandora_signatures(info)
        sigs = _gan_tmp66
        if _gan_truthy(gandora_std.enum.empty_p(sigs)):
            return None
        else:
            return types.SignatureHelp(signatures=sigs, active_signature=0, active_parameter=active)


def _open_call(prefix):
    found = _scan_call((gandora_std.string.length(prefix) - 1, 0, 0), prefix)
    if (found is None):
        return None
    else:
        _gan_val69 = found
        match _gan_val69:
            case (open_at, commas) as _gan_t70 if isinstance(_gan_t70, tuple):
                pass
            case _:
                raise GanMatchError("no match of right-hand side value: " + repr(_gan_val69))
        head = gandora_std.string.slice(prefix, 0, open_at)
        m = re.compile("(\\$?[A-Za-z_][A-Za-z0-9_.!?]*)\\s*$").search(head)
        if (m is None):
            return None
        else:
            return (m.group(1), commas)


def _scan_call(*_gan_args):
    while True:
        match _gan_args:
            case ((i, depth, commas) as _gan_t71, prefix,) if isinstance(_gan_t71, tuple):
                if i < 0:
                    return None
                else:
                    ch = gandora_std.string.slice(prefix, i, 1)
                    if (ch == ")") or (ch == "]") or (ch == "}"):
                        _gan_args = ((i - 1, depth + 1, commas), prefix)
                        continue
                    elif (ch == "[") or (ch == "{"):
                        _gan_args = ((i - 1, gandora_std.enum.max([depth - 1, 0]), commas), prefix)
                        continue
                    elif (ch == ",") and (depth == 0):
                        _gan_args = ((i - 1, depth, commas + 1), prefix)
                        continue
                    elif (ch == "(") and (depth == 0):
                        return (i, commas)
                    elif ch == "(":
                        _gan_args = ((i - 1, depth - 1, commas), prefix)
                        continue
                    else:
                        _gan_args = ((i - 1, depth, commas), prefix)
                        continue
        raise GanMatchError("no clause of scan_call/2 matched " + repr(_gan_args))


def _build_signature(label, params, docline):
    return types.SignatureInformation(label=label, documentation=docline, parameters=gandora_std.enum.map(params, lambda p: types.ParameterInformation(label=p)))


def _gandora_signatures(info):
    if (info is None):
        return []
    else:
        prose = gandora_std.map.get(gandora_std.map.get(info, "entries", {}), "default")
        if (prose is None):
            _gan_tmp72 = None
        else:
            _gan_tmp72 = gandora_std.enum.at(prose.strip().split("\n"), 0)
        docline = _gan_tmp72
        specs = gandora_std.map.get(info, "specs", [])
        heads = gandora_std.enum.uniq(gandora_std.map.get(info, "signatures", []))
        if _gan_truthy(gandora_std.enum.empty_p(specs)):
            _gan_tmp73 = docline
        else:
            _gan_tmp73 = gandora_std.enum.join(specs, "\n")
        label_doc = _gan_tmp73
        def _gan_fn11(head, *, info=info, label_doc=label_doc):
            params = _head_params(head)
            return types.SignatureInformation(label=head, documentation=label_doc, parameters=gandora_std.enum.map(params, lambda p, *, info=info: types.ParameterInformation(label=p, documentation=_param_doc(info, p))))
        return gandora_std.enum.map(heads, _gan_fn11)


def _head_params(head):
    m = re.compile("\\((.*)\\)").search(head)
    if (m is None):
        return []
    else:
        inner = m.group(1)
        return _split_top(inner)


def _split_top(inner):
    parts = _split_walk((0, 0, "", []), inner)
    return gandora_std.enum.reject(gandora_std.enum.map(parts, lambda p: p.strip()), lambda p: p == "")


def main() -> None:
    """The server entry: speaks LSP over stdio."""
    return server.start_io()


if __name__ == "__main__":
    main()
