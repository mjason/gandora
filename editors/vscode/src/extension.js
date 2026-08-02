// The thinnest possible client: spawn `gan lsp` (GEP-0015-R004).
const { execFile } = require("child_process");
const vscode = require("vscode");
const { LanguageClient } = require("vscode-languageclient/node");

let client;

const INSTALL_HINT =
  "Install the toolchain with: uv tool install gandora-tool && " +
  "uv tool install gandora-lsp — or set `gandora.gan.path`.";

function ganCommand() {
  return vscode.workspace.getConfiguration("gandora").get("gan.path") || "gan";
}

// Preflight through the same delegation machinery `gan lsp` uses
// (gan-lsc ships in the same package as gan-lsp), so a missing runner
// or missing plugin surfaces as one actionable message instead of a
// languageclient crash loop.
function preflight(cmd) {
  return new Promise((resolve) => {
    execFile(cmd, ["lsc", "version"], { timeout: 10000 }, (err, stdout) => {
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
  const cmd = ganCommand();
  const problem = await preflight(cmd);
  if (problem) {
    vscode.window.showErrorMessage(`Gandora LSP not started: ${problem}`);
    return;
  }
  client = new LanguageClient(
    "gandora",
    "Gandora Language Server",
    { command: cmd, args: ["lsp"] },
    { documentSelector: [{ scheme: "file", language: "gandora" }] }
  );
  client.start();
}

function deactivate() {
  return client ? client.stop() : undefined;
}

module.exports = { activate, deactivate };
