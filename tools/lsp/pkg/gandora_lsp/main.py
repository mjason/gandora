"""gan-lsp: the Gandora language server on pygls (GEP-0015 rev 2).
pygls owns protocol machinery; this module owns language logic.
"""

import builtins
import gandora_core as core
import lsprotocol.types
import lsprotocol.types as types
import pygls.lsp.server
import gandora_std.enum
import gandora_std.map
import gandora_std.string


class GanMatchError(Exception):
    pass

server = pygls.lsp.server.LanguageServer("gan-lsp", "0.2.2", text_document_sync_kind=lsprotocol.types.TextDocumentSyncKind.Full)


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


def main():
    return server.start_io()


if __name__ == "__main__":
    main()
