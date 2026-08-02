// The thinnest possible client: spawn `gan lsp` (GEP-0015-R004).
const { LanguageClient } = require("vscode-languageclient/node");

let client;

function activate() {
  client = new LanguageClient(
    "gandora",
    "Gandora Language Server",
    { command: "gan", args: ["lsp"] },
    { documentSelector: [{ scheme: "file", language: "gandora" }] }
  );
  client.start();
}

function deactivate() {
  return client ? client.stop() : undefined;
}

module.exports = { activate, deactivate };
