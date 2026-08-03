"""GEP-0015 conformance: a scripted stdio session against gan-lsp.

Covers: initialize handshake (full text sync announced), didOpen with an
erroneous buffer -> publishDiagnostics with the right span, didChange
fixing it -> empty list, didClose clearing, shutdown/exit with code 0.

Usage: python lsp_conformance.py [path-to-gan-lsp ...]
Defaults to `gan lsp` (the plugin-delegation path editors use).
"""

import json
import os
import subprocess
import sys
import tempfile

CMD = [
    os.path.abspath(a) if os.path.sep in a and os.path.exists(a) else a
    for a in (sys.argv[1:] or ["gan", "lsp"])
]


class Session:
    def __init__(self, cmd, cwd):
        self.proc = subprocess.Popen(
            cmd,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            cwd=cwd,
        )

    def send(self, msg):
        data = json.dumps(msg).encode()
        assert self.proc.stdin is not None
        self.proc.stdin.write(b"Content-Length: %d\r\n\r\n%s" % (len(data), data))
        self.proc.stdin.flush()

    def read_msg(self):
        assert self.proc.stdout is not None and self.proc.stderr is not None
        headers = b""
        while b"\r\n\r\n" not in headers:
            chunk = self.proc.stdout.read(1)
            if not chunk:
                raise EOFError(
                    "server closed stdout; stderr: "
                    + self.proc.stderr.read().decode()[:2000]
                )
            headers += chunk
        length = int(
            [
                line
                for line in headers.split(b"\r\n")
                if line.lower().startswith(b"content-length")
            ][0].split(b":")[1]
        )
        return json.loads(self.proc.stdout.read(length))

    def wait_for(self, pred):
        while True:
            msg = self.read_msg()
            if pred(msg):
                return msg


def main():
    failures = []

    def check(cond, label):
        print(("PASS " if cond else "FAIL ") + label)
        if not cond:
            failures.append(label)

    with tempfile.TemporaryDirectory() as root:
        uri = f"file://{root}/src/main.gan"
        s = Session(CMD, cwd=root)

        s.send(
            {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "processId": None,
                    "rootUri": f"file://{root}",
                    "capabilities": {},
                },
            }
        )
        resp = s.wait_for(lambda m: m.get("id") == 1)
        sync = resp["result"]["capabilities"].get("textDocumentSync")
        check("result" in resp, "initialize returns a result")
        check(
            isinstance(sync, dict) and sync.get("change") == 1,
            f"announces full text sync (GEP-0015-R002), got {sync!r}",
        )
        caps = resp["result"]["capabilities"]
        for cap, rule in [
            ("hoverProvider", "R005"),
            ("definitionProvider", "R006"),
            ("documentFormattingProvider", "R007"),
            ("documentSymbolProvider", "R008"),
            ("completionProvider", "R008"),
            ("signatureHelpProvider", "R010"),
        ]:
            check(bool(caps.get(cap)), f"announces {cap} (GEP-0015-{rule})")
        s.send({"jsonrpc": "2.0", "method": "initialized", "params": {}})

        bad = "defmodule Main do\n  def broken( do\nend\n"
        s.send(
            {
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {
                    "textDocument": {
                        "uri": uri,
                        "languageId": "gandora",
                        "version": 1,
                        "text": bad,
                    }
                },
            }
        )
        diag = s.wait_for(
            lambda m: m.get("method") == "textDocument/publishDiagnostics"
        )
        items = diag["params"]["diagnostics"]
        check(diag["params"]["uri"] == uri, "diagnostics arrive for the opened uri")
        check(len(items) >= 1, f"the parse error is published ({len(items)} item(s))")
        check(
            bool(items) and items[0]["range"]["start"]["line"] == 1,
            "the span points at the broken line",
        )

        good = 'defmodule Main do\n  def main(), do: IO.puts("ok")\nend\n'
        s.send(
            {
                "jsonrpc": "2.0",
                "method": "textDocument/didChange",
                "params": {
                    "textDocument": {"uri": uri, "version": 2},
                    "contentChanges": [{"text": good}],
                },
            }
        )
        diag2 = s.wait_for(
            lambda m: m.get("method") == "textDocument/publishDiagnostics"
        )
        check(diag2["params"]["diagnostics"] == [], "diagnostics clear after the fix")

        messy = 'defmodule Main do\n      def main(), do: IO.puts("ok")\nend\n'
        s.send(
            {
                "jsonrpc": "2.0",
                "method": "textDocument/didChange",
                "params": {
                    "textDocument": {"uri": uri, "version": 3},
                    "contentChanges": [{"text": messy}],
                },
            }
        )
        s.wait_for(lambda m: m.get("method") == "textDocument/publishDiagnostics")
        s.send(
            {
                "jsonrpc": "2.0",
                "id": 4,
                "method": "textDocument/formatting",
                "params": {
                    "textDocument": {"uri": uri},
                    "options": {"tabSize": 2, "insertSpaces": True},
                },
            }
        )
        fmt = s.wait_for(lambda m: m.get("id") == 4).get("result")
        check(
            bool(fmt) and fmt[0]["newText"] == good,
            "formatting returns the canonical document (GEP-0015-R007)",
        )

        # recursion-shape hover badge (GEP-0019-R006)
        os.makedirs(f"{root}/src", exist_ok=True)
        with open(f"{root}/gandora.jsonc", "w") as f:
            f.write('{"source": ["src"], "outDir": "dist"}\n')
        recur_src = (
            "defmodule Main do\n"
            "  def down(0), do: :done\n"
            "  def down(n), do: down(n - 1)\n"
            "end\n"
        )
        with open(f"{root}/src/main.gan", "w") as f:
            f.write(recur_src)
        s.send(
            {
                "jsonrpc": "2.0",
                "method": "textDocument/didChange",
                "params": {
                    "textDocument": {"uri": uri, "version": 4},
                    "contentChanges": [{"text": recur_src}],
                },
            }
        )
        s.wait_for(lambda m: m.get("method") == "textDocument/publishDiagnostics")
        s.send(
            {
                "jsonrpc": "2.0",
                "id": 5,
                "method": "textDocument/hover",
                "params": {
                    "textDocument": {"uri": uri},
                    "position": {"line": 1, "character": 7},
                },
            }
        )
        hov = s.wait_for(lambda m: m.get("id") == 5).get("result") or {}
        hover_text = (hov.get("contents") or {}).get("value", "")
        check(
            "while" in hover_text and "constant stack" in hover_text,
            "hover shows the compiled recursion shape (GEP-0019-R006)",
        )

        # stack-recursion warning arrives as a Warning diagnostic on the
        # def line (GEP-0019-R007)
        fact_src = (
            "defmodule Main do\n"
            "  def fact(0), do: 1\n"
            "  def fact(n), do: n * fact(n - 1)\n"
            "end\n"
        )
        s.send(
            {
                "jsonrpc": "2.0",
                "method": "textDocument/didChange",
                "params": {
                    "textDocument": {"uri": uri, "version": 5},
                    "contentChanges": [{"text": fact_src}],
                },
            }
        )
        wdiag = s.wait_for(
            lambda m: m.get("method") == "textDocument/publishDiagnostics"
        )
        witems = wdiag["params"]["diagnostics"]
        check(
            any(
                "GEP-0019-R007" in d["message"]
                and d["severity"] == 2
                and d["range"]["start"]["line"] == 1
                for d in witems
            ),
            "stack recursion warns on the def line (GEP-0019-R007)",
        )

        s.send(
            {
                "jsonrpc": "2.0",
                "method": "textDocument/didClose",
                "params": {"textDocument": {"uri": uri}},
            }
        )
        diag3 = s.wait_for(
            lambda m: m.get("method") == "textDocument/publishDiagnostics"
        )
        check(diag3["params"]["diagnostics"] == [], "diagnostics clear on close")

        s.send({"jsonrpc": "2.0", "id": 2, "method": "shutdown", "params": None})
        s.wait_for(lambda m: m.get("id") == 2)
        s.send({"jsonrpc": "2.0", "method": "exit", "params": None})
        code = s.proc.wait(timeout=10)
        check(code == 0, f"clean exit (code {code})")

    print("=" * 40)
    print("ALL PASS" if not failures else f"{len(failures)} FAILURES: {failures}")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
