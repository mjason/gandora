"""
  gan-lsp: a Language Server Protocol server written in Gandora
  (GEP-0015). v1: lifecycle + push diagnostics via gandora_core.
"""

import builtins
import gandora_core as core
import json
import sys
import urllib.parse as urlparse
import gandora_std.enum
import gandora_std.map
import gandora_std.string


def _gan_truthy(value):
    return value is not None and value is not False

class GanMatchError(Exception):
    pass


def main():
    _gan_loop0 = {"root": None, "running": True}
    _gan_res1 = None
    while True:
        state = _gan_loop0
        _gan_case2 = _read_message()
        match _gan_case2:
            case "eof":
                _gan_res1 = 0
                break
            case msg:
                try:
                    _gan_tmp3 = _handle(msg, state)
                except Exception as e:
                    _log(f"handler error: {builtins.type(e).__name__}: {str(e)}")
                    _gan_tmp3 = state
                next = _gan_tmp3
                if _gan_truthy(gandora_std.map.get(next, "running")):
                    _gan_loop0 = next
                    continue
                else:
                    _gan_res1 = 0
                    break
        break
    return sys.exit(0)


def _read_message():
    _gan_loop5 = None
    _gan_res6 = None
    while True:
        n = _gan_loop5
        line = sys.stdin.buffer.readline()
        if builtins.len(line) == 0:
            _gan_res6 = "eof"
            break
        elif line == builtins.bytes("\r\n", "ascii"):
            _gan_res6 = n
            break
        else:
            text = line.decode("ascii").strip()
            if _gan_truthy(gandora_std.string.starts_with_p(gandora_std.string.downcase(text), "content-length:")):
                _gan_loop5 = gandora_std.string.to_integer(gandora_std.string.trim(gandora_std.enum.at(text.split(":"), 1)))
                continue
            else:
                _gan_loop5 = n
                continue
        break
    _gan_tmp4 = _gan_res6
    len = _gan_tmp4
    if len == "eof":
        return "eof"
    elif _gan_truthy((len is None)):
        return "eof"
    else:
        body = sys.stdin.buffer.read(len)
        return json.loads(body.decode("utf-8"))


def _send_message(payload):
    body = json.dumps(payload).encode("utf-8")
    header = builtins.bytes(f"Content-Length: {builtins.len(body)}\r\n\r\n", "ascii")
    sys.stdout.buffer.write(header)
    sys.stdout.buffer.write(body)
    return sys.stdout.buffer.flush()


def _respond(id, result):
    return _send_message({"jsonrpc": "2.0", "id": id, "result": result})


def _respond_error(id, code, message):
    return _send_message({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})


def _notify(method, params):
    return _send_message({"jsonrpc": "2.0", "method": method, "params": params})


def _log(message):
    return _notify("window/logMessage", {"type": 2, "message": "gan-lsp: " + message})


def _handle(msg, state):
    method = gandora_std.map.get(msg, "method")
    id = gandora_std.map.get(msg, "id")
    _gan_case7 = method
    match _gan_case7:
        case "initialize":
            root = _root_path(gandora_std.map.get(msg, "params"))
            _respond(id, {"capabilities": {"textDocumentSync": 1}, "serverInfo": {"name": "gan-lsp", "version": core.version()}})
            return gandora_std.map.put(state, "root", root)
        case "initialized":
            return state
        case "textDocument/didOpen":
            doc = gandora_std.map.get(gandora_std.map.get(msg, "params"), "textDocument")
            _publish(gandora_std.map.get(doc, "uri"), gandora_std.map.get(doc, "text"), state)
            return state
        case "textDocument/didChange":
            params = gandora_std.map.get(msg, "params")
            uri = gandora_std.map.get(gandora_std.map.get(params, "textDocument"), "uri")
            changes = gandora_std.map.get(params, "contentChanges")
            text = gandora_std.map.get(gandora_std.enum.at(changes, -1), "text")
            _publish(uri, text, state)
            return state
        case "textDocument/didClose":
            uri = gandora_std.map.get(gandora_std.map.get(gandora_std.map.get(msg, "params"), "textDocument"), "uri")
            _notify("textDocument/publishDiagnostics", {"uri": uri, "diagnostics": []})
            return state
        case "shutdown":
            _respond(id, None)
            return state
        case "exit":
            return gandora_std.map.put(state, "running", False)
        case _:
            if _gan_truthy((id is None)):
                return state
            else:
                _respond_error(id, -32601, f"method not found: {method}")
                return state


def _root_path(params):
    uri = gandora_std.map.get(params, "rootUri")
    if _gan_truthy((uri is None)):
        return gandora_std.map.get(params, "rootPath")
    else:
        return _uri_to_path(uri)


def _uri_to_path(uri):
    return urlparse.unquote(gandora_std.string.replace(uri, "file://", ""))


def _publish(uri, text, state):
    path = _uri_to_path(uri)
    diags = core.diagnostics(text, path, gandora_std.map.get(state, "root"))
    def _gan_fn0(d):
        line = gandora_std.enum.max([gandora_std.map.get(d, "line") - 1, 0])
        col = gandora_std.enum.max([gandora_std.map.get(d, "col") - 1, 0])
        return {"range": {"start": {"line": line, "character": col}, "end": {"line": line, "character": col}}, "severity": _severity(gandora_std.map.get(d, "severity")), "source": "gan", "message": gandora_std.map.get(d, "message")}
    lsp = gandora_std.enum.map(diags, _gan_fn0)
    return _notify("textDocument/publishDiagnostics", {"uri": uri, "diagnostics": lsp})


def _severity(*_gan_args):
    match _gan_args:
        case ("error",):
            return 1
        case (_,):
            return 2
    raise GanMatchError("no clause of severity/1 matched " + repr(_gan_args))


if __name__ == "__main__":
    main()
