"""gan-lsp: the Gandora language server on pygls (GEP-0015 rev 4).
pygls owns protocol machinery; this module owns language logic:
push diagnostics on every edit, documentation hover (signatures,
translations, examples, `$module` references, language constructs),
go-to-definition, document symbols, `Module.` completion, and
whole-document formatting through the GEP-0016 engine.
"""

import builtins
import gandora_core as core
import importlib.util
import lsprotocol.types
import lsprotocol.types as types
import pygls.lsp.server
import re
import gandora_std.enum
import gandora_std.map
import gandora_std.string
import gandora_tool.fmt


def _gan_truthy(value):
    return value is not None and value is not False

def _gan_or(value, then):
    return value if _gan_truthy(value) else then()

class GanMatchError(Exception):
    pass

server = pygls.lsp.server.LanguageServer("gan-lsp", "0.4.2", text_document_sync_kind=lsprotocol.types.TextDocumentSyncKind.Full)

word_re = re.compile("[A-Za-z_][A-Za-z0-9_.!?]*")

construct_docs = {"def": "Defines a public function. `def f(x), do: expr` or a `do ... end` body. Multi-clause heads dispatch by pattern, top to bottom (GEP-0001).", "defp": "Defines a private function — callable only inside its module; compiled with a leading underscore (GEP-0001).", "defmodule": "Declares the module for this file. One `defmodule` per file; the name maps to the generated Python module path (GEP-0001-R013).", "defmacro": "Defines a compile-time macro: it receives quoted arguments and returns quoted code (GEP-0002).", "defstruct": "Declares the module's struct with defaulted fields; literals `%Mod{...}`, updates `%Mod{s | ...}` and patterns work on it (GEP-0004).", "defattr": "Registers a custom annotation attribute handled by the module's `@on_definition` hook (GEP-0008).", "quote": "Returns the quoted AST of its block instead of evaluating it; `unquote` splices values back in (GEP-0002).", "unquote": "Inside `quote`, splices an evaluated value into the quoted code (GEP-0002).", "case": "Pattern-matches a value against clauses, top to bottom; the whole form is an expression (GEP-0001).", "cond": "Evaluates conditions top to bottom and takes the first truthy branch (GEP-0001).", "with": "Chains `pattern <- expr` matches; the first failure falls to `else` (GEP-0001).", "try": "Runs a body with `rescue` clauses matching Python exception types and an always-run `after` (GEP-0014).", "rescue": "Clauses of a `try`: `e in $mod.Type -> ...` matches by exception class; a bare variable catches every Exception (GEP-0014).", "after": "The cleanup section of `try`: always runs, contributes no value (GEP-0014).", "loop": "Binds a state pattern and repeats its body: `recur(new_state)` restarts, `break(value)` finishes — constant stack depth (GEP-0014).", "recur": "Rebinds the enclosing `loop` state and restarts its body (GEP-0014).", "break": "Ends the enclosing `loop` with the given value (GEP-0014).", "fn": "An anonymous function: `fn x -> x * 2 end`, called with `f.(x)`; supports multiple clauses and guards (GEP-0001).", "pyimport": "Declares a Python import at module top: `pyimport numpy, as: np` (GEP-0003).", "use": "Invokes the target module's `__using__` macro to inject code here (GEP-0008).", "require": "Makes the target module's macros available in this file (GEP-0002).", "unless": "`if` with the condition negated (GEP-0001).", "when": "A guard on a clause head or case pattern (GEP-0001)."}


@server.feature(lsprotocol.types.TEXT_DOCUMENT_DID_OPEN)
def did_open(params):
    return _publish(params.text_document.uri, params.text_document.text)


@server.feature(lsprotocol.types.TEXT_DOCUMENT_DID_CHANGE)
def did_change(params):
    text = gandora_std.enum.at(builtins.list(params.content_changes), -1).text
    return _publish(params.text_document.uri, text)


@server.feature(lsprotocol.types.TEXT_DOCUMENT_DID_CLOSE)
def did_close(params):
    return _send_diagnostics(params.text_document.uri, [])


def _send_diagnostics(uri, items):
    return server.text_document_publish_diagnostics(lsprotocol.types.PublishDiagnosticsParams(uri=uri, diagnostics=items))


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
            return lsprotocol.types.DiagnosticSeverity.Error
        case (_,):
            return lsprotocol.types.DiagnosticSeverity.Warning
    raise GanMatchError("no clause of severity/1 matched " + repr(_gan_args))


def _doc_and_line(params):
    doc = server.workspace.get_text_document(params.text_document.uri)
    line = gandora_std.enum.at(doc.source.split("\n"), params.position.line)
    return (doc, line)


def _word_at(line, character):
    def _gan_fn1(m):
        _gan_val1 = m.span()
        match _gan_val1:
            case (s, e) as _gan_t2 if isinstance(_gan_t2, tuple):
                pass
            case _:
                raise GanMatchError("no match of right-hand side value: " + repr(_gan_val1))
        return (s <= character) and (character < e)
    hit = gandora_std.enum.find(word_re.finditer(line), _gan_fn1)
    if _gan_truthy((hit is None)):
        return None
    else:
        _gan_val3 = hit.span()
        match _gan_val3:
            case (s, _e) as _gan_t4 if isinstance(_gan_t4, tuple):
                pass
            case _:
                raise GanMatchError("no match of right-hand side value: " + repr(_gan_val3))
        return (hit.group(0).rstrip("."), s)


def _module_of(source):
    m = re.compile("defmodule\\s+([A-Za-z0-9_.]+)").search(source)
    if _gan_truthy((m is None)):
        return None
    else:
        return m.group(1)


def _target_of(source, token):
    if _gan_truthy(gandora_std.string.match_p(token, re.compile("^[A-Z]"))):
        return token
    else:
        mod = _module_of(source)
        if _gan_truthy((mod is None)):
            return None
        else:
            return f"{mod}.{token}"


@server.feature(lsprotocol.types.TEXT_DOCUMENT_HOVER)
def hover(params):
    _gan_val5 = _doc_and_line(params)
    match _gan_val5:
        case (doc, line) as _gan_t6 if isinstance(_gan_t6, tuple):
            pass
        case _:
            raise GanMatchError("no match of right-hand side value: " + repr(_gan_val5))
    hit = _word_at(line, params.position.character)
    if _gan_truthy((hit is None)):
        return None
    else:
        _gan_val7 = hit
        match _gan_val7:
            case (token, start) as _gan_t8 if isinstance(_gan_t8, tuple):
                pass
            case _:
                raise GanMatchError("no match of right-hand side value: " + repr(_gan_val7))
        if (start > 0) and (gandora_std.string.slice(line, start - 1, 1) == "$"):
            return _pyref_hover(token)
        elif _gan_truthy(gandora_std.map.has_key_p(construct_docs, token)):
            return _markdown_hover(f"`{token}`\n\n{gandora_std.map.get(construct_docs, token)}")
        else:
            target = _target_of(doc.source, token)
            info = _lookup_doc(target)
            return _render_hover(target, info)


def _lookup_doc(target):
    if _gan_truthy((target is None)):
        return None
    else:
        try:
            return core.doc(target, server.workspace.root_path)
        except Exception as _e:
            return None


def _pyref_hover(token):
    mod = gandora_std.enum.at(token.split("."), 0)
    try:
        _gan_tmp9 = importlib.util.find_spec(mod)
    except Exception as _e:
        _gan_tmp9 = None
    spec = _gan_tmp9
    if _gan_truthy((spec is None)):
        return _markdown_hover(f"`${mod}` — Python module (not found in this environment)")
    else:
        return _markdown_hover(f"`${mod}` — Python module\n\n`{spec.origin}`")


def _markdown_hover(md):
    return types.Hover(contents=types.MarkupContent(kind=lsprotocol.types.MarkupKind.Markdown, value=md))


def _render_hover(target, info):
    if _gan_truthy(_gan_or((info is None), lambda: info.get("hidden"))):
        return None
    else:
        sigs = gandora_std.enum.join(gandora_std.enum.uniq(gandora_std.map.get(info, "signatures", [])), "\n")
        if sigs == "":
            _gan_tmp10 = f"### {target}"
        else:
            _gan_tmp10 = f"```elixir\n{sigs}\n```"
        header = _gan_tmp10
        entries = gandora_std.map.get(info, "entries", {})
        prose = gandora_std.map.get(entries, "default")
        def _gan_fn2(*_gan_args):
            match _gan_args:
                case ((loc, _) as _gan_t11,) if isinstance(_gan_t11, tuple):
                    return loc != "default"
            raise GanMatchError("no clause of _gan_fn2/1 matched " + repr(_gan_args))
        def _gan_fn3(*_gan_args):
            match _gan_args:
                case ((loc, text) as _gan_t12,) if isinstance(_gan_t12, tuple):
                    return f"\n---\n**{loc}**\n\n{text.strip()}"
            raise GanMatchError("no clause of _gan_fn3/1 matched " + repr(_gan_args))
        translations = gandora_std.enum.map(gandora_std.enum.filter(entries.items(), _gan_fn2), _gan_fn3)
        def _gan_fn4(*_gan_args):
            match _gan_args:
                case ((k, v) as _gan_t13,) if isinstance(_gan_t13, tuple):
                    return f"\n`{k}`: {v}"
            raise GanMatchError("no clause of _gan_fn4/1 matched " + repr(_gan_args))
        meta = gandora_std.enum.map(gandora_std.map.get(info, "meta", []), _gan_fn4)
        examples = gandora_std.enum.map(gandora_std.map.get(info, "examples", []), lambda ex: f"\n```elixir\n{ex.strip()}\n```")
        body = gandora_std.enum.join(gandora_std.enum.reject([prose] + (translations + (meta + examples)), lambda part: (part is None)), "\n")
        if (gandora_std.string.trim(body) == "") and (sigs == ""):
            return None
        else:
            return _markdown_hover(f"{header}\n\n{body}")


@server.feature(lsprotocol.types.TEXT_DOCUMENT_DEFINITION)
def definition(params):
    _gan_val14 = _doc_and_line(params)
    match _gan_val14:
        case (doc, line) as _gan_t15 if isinstance(_gan_t15, tuple):
            pass
        case _:
            raise GanMatchError("no match of right-hand side value: " + repr(_gan_val14))
    hit = _word_at(line, params.position.character)
    if _gan_truthy((hit is None)):
        return None
    else:
        _gan_val16 = hit
        match _gan_val16:
            case (token, start) as _gan_t17 if isinstance(_gan_t17, tuple):
                pass
            case _:
                raise GanMatchError("no match of right-hand side value: " + repr(_gan_val16))
        is_pyref = (start > 0) and (gandora_std.string.slice(line, start - 1, 1) == "$")
        if _gan_truthy(is_pyref):
            _gan_tmp18 = None
        else:
            _gan_tmp18 = _target_of(doc.source, token)
        target = _gan_tmp18
        if _gan_truthy((target is None)):
            _gan_tmp19 = None
        else:
            try:
                _gan_tmp19 = core.definition(target, server.workspace.root_path)
            except Exception as _e:
                _gan_tmp19 = None
        loc = _gan_tmp19
        if _gan_truthy((loc is None)):
            return None
        else:
            pos = types.Position(line=gandora_std.enum.max([gandora_std.map.get(loc, "line") - 1, 0]), character=gandora_std.enum.max([gandora_std.map.get(loc, "col") - 1, 0]))
            return types.Location(uri="file://" + gandora_std.map.get(loc, "path"), range=types.Range(pos, pos))


@server.feature(lsprotocol.types.TEXT_DOCUMENT_DOCUMENT_SYMBOL)
def document_symbol(params):
    doc = server.workspace.get_text_document(params.text_document.uri)
    mod = _module_of(doc.source)
    if _gan_truthy((mod is None)):
        _gan_tmp20 = []
    else:
        try:
            _gan_tmp20 = core.symbols(mod, server.workspace.root_path)
        except Exception as _e:
            _gan_tmp20 = []
    syms = _gan_tmp20
    def _gan_fn5(s):
        line = gandora_std.enum.max([gandora_std.map.get(s, "line") - 1, 0])
        pos = types.Position(line=line, character=0)
        rng = types.Range(pos, pos)
        return types.DocumentSymbol(name=gandora_std.map.get(s, "name"), detail=gandora_std.map.get(s, "head"), kind=_symbol_kind(gandora_std.map.get(s, "kind")), range=rng, selection_range=rng)
    children = gandora_std.enum.map(syms, _gan_fn5)
    if _gan_truthy((mod is None)):
        return []
    else:
        zero = types.Position(line=0, character=0)
        return [types.DocumentSymbol(name=mod, kind=lsprotocol.types.SymbolKind.Module, range=types.Range(zero, zero), selection_range=types.Range(zero, zero), children=children)]


def _symbol_kind(*_gan_args):
    match _gan_args:
        case ("defmacro",):
            return lsprotocol.types.SymbolKind.Constructor
        case ("defp",):
            return lsprotocol.types.SymbolKind.Method
        case (_,):
            return lsprotocol.types.SymbolKind.Function
    raise GanMatchError("no clause of symbol_kind/1 matched " + repr(_gan_args))


@server.feature(lsprotocol.types.TEXT_DOCUMENT_COMPLETION, lsprotocol.types.CompletionOptions(trigger_characters=["."]))
def completion(params):
    _gan_val21 = _doc_and_line(params)
    match _gan_val21:
        case (_doc, line) as _gan_t22 if isinstance(_gan_t22, tuple):
            pass
        case _:
            raise GanMatchError("no match of right-hand side value: " + repr(_gan_val21))
    prefix = gandora_std.string.slice(line, 0, params.position.character)
    m = re.compile("([A-Z][A-Za-z0-9_]*(?:\\.[A-Z][A-Za-z0-9_]*)*)\\.([a-z_][A-Za-z0-9_]*)?$").search(prefix)
    if _gan_truthy((m is None)):
        return []
    else:
        mod = m.group(1)
        if _gan_truthy((m.group(2) is None)):
            _gan_tmp23 = ""
        else:
            _gan_tmp23 = m.group(2)
        partial = _gan_tmp23
        try:
            _gan_tmp24 = core.symbols(mod, server.workspace.root_path)
        except Exception as _e:
            _gan_tmp24 = []
        syms = _gan_tmp24
        def _gan_fn6(s, acc):
            if _gan_truthy(gandora_std.enum.any_p(acc, lambda a: gandora_std.map.get(a, "name") == gandora_std.map.get(s, "name"))):
                return acc
            else:
                return acc + [s]
        return gandora_std.enum.map(gandora_std.enum.reduce(gandora_std.enum.filter(gandora_std.enum.filter(syms, lambda s: gandora_std.map.get(s, "kind") != "defp"), lambda s: gandora_std.map.get(s, "name").startswith(partial)), [], _gan_fn6), lambda s: types.CompletionItem(label=gandora_std.map.get(s, "name"), kind=lsprotocol.types.CompletionItemKind.Function, detail=gandora_std.map.get(s, "head"), documentation=gandora_std.map.get(s, "doc")))


@server.feature(lsprotocol.types.TEXT_DOCUMENT_FORMATTING)
def formatting(params):
    doc = server.workspace.get_text_document(params.text_document.uri)
    result = gandora_tool.fmt.format_text(doc.source)
    _gan_case26 = result
    match _gan_case26:
        case ("ok", new) as _gan_t27 if isinstance(_gan_t27, tuple):
            if (new == doc.source) or (gandora_tool.fmt.verify(doc.source, new) != "ok"):
                return []
            else:
                line_count = gandora_std.enum.count(doc.source.split("\n"))
                return [types.TextEdit(range=types.Range(types.Position(line=0, character=0), types.Position(line=line_count, character=0)), new_text=new)]
        case _:
            return []


def main():
    return server.start_io()


if __name__ == "__main__":
    main()
