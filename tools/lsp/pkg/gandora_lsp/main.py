"""gan-lsp: the Gandora language server on pygls (GEP-0015 rev 3).
pygls owns protocol machinery; this module owns language logic:
push diagnostics on every edit, and hover serving the GEP-0007
documentation channel (default locale plus available translations).
"""

import builtins
import gandora_core as core
import lsprotocol.types
import lsprotocol.types as types
import pygls.lsp.server
import re
import gandora_std.enum
import gandora_std.map
import gandora_std.string


def _gan_truthy(value):
    return value is not None and value is not False

def _gan_or(value, then):
    return value if _gan_truthy(value) else then()

class GanMatchError(Exception):
    pass

server = pygls.lsp.server.LanguageServer("gan-lsp", "0.4.1", text_document_sync_kind=lsprotocol.types.TextDocumentSyncKind.Full)

word_re = re.compile("[A-Za-z_][A-Za-z0-9_.!?]*")


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


@server.feature(lsprotocol.types.TEXT_DOCUMENT_HOVER)
def hover(params):
    doc = server.workspace.get_text_document(params.text_document.uri)
    line = gandora_std.enum.at(doc.source.split("\n"), params.position.line)
    token = _word_at(line, params.position.character)
    if _gan_truthy((token is None)):
        _gan_tmp1 = None
    elif _gan_truthy(gandora_std.string.match_p(token, re.compile("^[A-Z]"))):
        _gan_tmp1 = token
    else:
        mod = _module_of(doc.source)
        if _gan_truthy((mod is None)):
            _gan_tmp1 = None
        else:
            _gan_tmp1 = f"{mod}.{token}"
    target = _gan_tmp1
    if _gan_truthy((target is None)):
        return None
    else:
        try:
            _gan_tmp2 = core.doc(target, server.workspace.root_path)
        except Exception as _e:
            _gan_tmp2 = None
        info = _gan_tmp2
        return _render_hover(target, info)


def _word_at(line, character):
    def _gan_fn1(m):
        _gan_val3 = m.span()
        match _gan_val3:
            case (s, e) as _gan_t4 if isinstance(_gan_t4, tuple):
                pass
            case _:
                raise GanMatchError("no match of right-hand side value: " + repr(_gan_val3))
        return (s <= character) and (character < e)
    hit = gandora_std.enum.find(word_re.finditer(line), _gan_fn1)
    if _gan_truthy((hit is None)):
        return None
    else:
        return hit.group(0).rstrip(".")


def _module_of(source):
    m = re.compile("defmodule\\s+([A-Za-z0-9_.]+)").search(source)
    if _gan_truthy((m is None)):
        return None
    else:
        return m.group(1)


def _render_hover(target, info):
    if _gan_truthy(_gan_or((info is None), lambda: info.get("hidden"))):
        return None
    else:
        entries = gandora_std.map.get(info, "entries", {})
        prose = gandora_std.map.get(entries, "default")
        def _gan_fn2(*_gan_args):
            match _gan_args:
                case ((loc, _) as _gan_t5,) if isinstance(_gan_t5, tuple):
                    return loc != "default"
            raise GanMatchError("no clause of _gan_fn2/1 matched " + repr(_gan_args))
        def _gan_fn3(*_gan_args):
            match _gan_args:
                case ((loc, text) as _gan_t6,) if isinstance(_gan_t6, tuple):
                    return f"\n---\n**{loc}**\n\n{text}"
            raise GanMatchError("no clause of _gan_fn3/1 matched " + repr(_gan_args))
        translations = gandora_std.enum.map(gandora_std.enum.filter(entries.items(), _gan_fn2), _gan_fn3)
        def _gan_fn4(*_gan_args):
            match _gan_args:
                case ((k, v) as _gan_t7,) if isinstance(_gan_t7, tuple):
                    return f"\n*{k}*: {v}"
            raise GanMatchError("no clause of _gan_fn4/1 matched " + repr(_gan_args))
        meta = gandora_std.enum.map(gandora_std.map.get(info, "meta", []), _gan_fn4)
        examples = gandora_std.enum.map(gandora_std.map.get(info, "examples", []), lambda ex: f"\n```elixir\n{ex.strip()}\n```")
        body = gandora_std.enum.join(gandora_std.enum.reject([prose] + (translations + (meta + examples)), lambda part: (part is None)), "\n")
        if gandora_std.string.trim(body) == "":
            return None
        else:
            return types.Hover(contents=types.MarkupContent(kind=lsprotocol.types.MarkupKind.Markdown, value=f"### {target}\n\n{body}"))


def main():
    return server.start_io()


if __name__ == "__main__":
    main()
