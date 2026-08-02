// The thinnest possible client: spawn `gan lsp` (GEP-0015-R004).
const { execFile } = require("child_process");
const vscode = require("vscode");
const { LanguageClient } = require("vscode-languageclient/node");

let client;

const INSTALL_HINT =
  "Add `gandora-tool[dev]` to the project's dev dependencies (uv add --dev " +
  '"gandora-tool[dev]") or install globally: uv tool install gandora-tool ' +
  "gandora-lsp — or set `gandora.gan.path`.";

function ganCommand() {
  return vscode.workspace.getConfiguration("gandora").get("gan.path") || "gan";
}

// Preflight through the same delegation machinery `gan lsp` uses
// (gan-lsc ships in the same package as gan-lsp), so a missing runner
// or missing plugin surfaces as one actionable message instead of a
// languageclient crash loop.
function preflight(cmd, cwd) {
  return new Promise((resolve) => {
    execFile(cmd, ["lsc", "version"], { timeout: 10000, cwd }, (err, stdout) => {
      if (err) {
        resolve(
          err.code === "ENOENT"
            ? `'${cmd}' was not found on the extension host's PATH. ${INSTALL_HINT}`
            : `'${cmd} lsc version' failed (${String(err.message).split("\n")[0]}). ${INSTALL_HINT}`
        );
      } else if (!stdout.includes("version")) {
        resolve(`'${cmd} lsc version' gave unexpected output. ${INSTALL_HINT}`);
      } else {
        resolve(null);
      }
    });
  });
}

async function activate() {
  const output = vscode.window.createOutputChannel("Gandora Language Server");
  const cmd = ganCommand();
  const cwd = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  output.appendLine(`[gandora] activating: command '${cmd}', workspace ${cwd ?? "(none)"}`);
  const problem = await preflight(cmd, cwd);
  if (problem) {
    output.appendLine(`[gandora] preflight failed: ${problem}`);
    output.show(true);
    vscode.window.showErrorMessage(`Gandora LSP not started: ${problem}`);
    return;
  }
  output.appendLine("[gandora] preflight ok, starting `" + cmd + " lsp`");
  client = new LanguageClient(
    "gandora",
    "Gandora Language Server",
    { command: cmd, args: ["lsp"], options: { cwd } },
    {
      documentSelector: [{ scheme: "file", language: "gandora" }],
      outputChannel: output,
    }
  );
  await client.start();
  output.appendLine("[gandora] language server running (diagnostics + hover)");
}

function deactivate() {
  return client ? client.stop() : undefined;
}

module.exports = { activate, deactivate };
