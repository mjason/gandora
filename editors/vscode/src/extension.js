// Gandora for VS Code: the LSP client (GEP-0015-R004) plus the
// commands that make a compile-to-Python language pleasant — compiled
// output preview, macro expansion, run/test/REPL terminals.
const { execFile } = require("child_process");
const path = require("path");
const vscode = require("vscode");
const { LanguageClient } = require("vscode-languageclient/node");

let client;

const INSTALL_HINT =
  "Add `gandora-tool[dev]` to the project's dev dependencies (uv add --dev " +
  '"gandora-tool[dev]") or install globally: uv tool install gandora-tool ' +
  "gandora-lsp — or set `gandora.gan.path`.";

const PREVIEW_SCHEME = "gandora-preview";

function ganCommand() {
  return vscode.workspace.getConfiguration("gandora").get("gan.path") || "gan";
}

function workspaceRoot() {
  return vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
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

function lsc(args, cwd) {
  return new Promise((resolve, reject) => {
    execFile(
      ganCommand(),
      ["lsc", ...args],
      { timeout: 30000, cwd, maxBuffer: 16 * 1024 * 1024 },
      (err, stdout, stderr) => {
        if (err) reject(new Error(stderr || err.message));
        else resolve(stdout);
      }
    );
  });
}

// ---- compiled-Python / expanded-AST preview -------------------------------

// gandora-preview:/compile/<fsPath>.py  |  gandora-preview:/expand/<fsPath>.json
function previewUri(kind, fsPath) {
  const ext = kind === "compile" ? ".py" : ".json";
  return vscode.Uri.from({
    scheme: PREVIEW_SCHEME,
    path: `/${kind}${fsPath}${ext}`,
  });
}

function previewTarget(uri) {
  const m = uri.path.match(/^\/(compile|expand)(\/.*?)(\.py|\.json)$/);
  return m ? { kind: m[1], fsPath: m[2] } : null;
}

class PreviewProvider {
  constructor() {
    this.emitter = new vscode.EventEmitter();
    this.onDidChange = this.emitter.event;
  }

  async provideTextDocumentContent(uri) {
    const target = previewTarget(uri);
    if (!target) return "// unrecognized preview target";
    const root = workspaceRoot() ?? path.dirname(target.fsPath);
    try {
      const out = await lsc([target.kind, target.fsPath, "--root", root], root);
      if (target.kind === "expand") {
        try {
          return JSON.stringify(JSON.parse(out), null, 2);
        } catch {
          return out;
        }
      }
      return out;
    } catch (e) {
      const comment = target.kind === "compile" ? "#" : "//";
      return `${comment} ${String(e.message).trim().split("\n").join(`\n${comment} `)}`;
    }
  }
}

async function openPreview(provider, kind) {
  const editor = vscode.window.activeTextEditor;
  if (!editor || editor.document.languageId !== "gandora") return;
  if (editor.document.isDirty) await editor.document.save();
  const uri = previewUri(kind, editor.document.uri.fsPath);
  provider.emitter.fire(uri);
  const doc = await vscode.workspace.openTextDocument(uri);
  await vscode.languages.setTextDocumentLanguage(
    doc,
    kind === "compile" ? "python" : "json"
  );
  await vscode.window.showTextDocument(doc, {
    viewColumn: vscode.ViewColumn.Beside,
    preserveFocus: true,
    preview: false,
  });
}

// ---- terminals ------------------------------------------------------------

function runInTerminal(name, commandLine) {
  let t = vscode.window.terminals.find((term) => term.name === name);
  if (!t) {
    t = vscode.window.createTerminal({ name, cwd: workspaceRoot() });
  }
  t.show(true);
  t.sendText(commandLine);
}

// ---- activation -----------------------------------------------------------

async function startClient(output) {
  const cmd = ganCommand();
  const cwd = workspaceRoot();
  output.appendLine(`[gandora] activating: command '${cmd}', workspace ${cwd ?? "(none)"}`);
  const problem = await preflight(cmd, cwd);
  if (problem) {
    output.appendLine(`[gandora] preflight failed: ${problem}`);
    output.show(true);
    vscode.window.showErrorMessage(`Gandora LSP not started: ${problem}`);
    return null;
  }
  output.appendLine("[gandora] preflight ok, starting `" + cmd + " lsp`");
  // an explicit setting feeds the env; gandora.local.jsonc still wins
  // (GEP-0015-R015: closer scope, closer heart)
  const locale = vscode.workspace.getConfiguration("gandora").get("doc.locale");
  const env = locale
    ? { ...process.env, GAN_DOC_LOCALE: locale }
    : { ...process.env };
  const c = new LanguageClient(
    "gandora",
    "Gandora Language Server",
    { command: cmd, args: ["lsp"], options: { cwd, env } },
    {
      documentSelector: [{ scheme: "file", language: "gandora" }],
      outputChannel: output,
    }
  );
  await c.start();
  output.appendLine("[gandora] language server running");
  return c;
}

async function activate(context) {
  const output = vscode.window.createOutputChannel("Gandora Language Server");
  client = await startClient(output);

  const provider = new PreviewProvider();
  context.subscriptions.push(
    vscode.workspace.registerTextDocumentContentProvider(PREVIEW_SCHEME, provider)
  );

  // locale changes need a server restart to take effect
  context.subscriptions.push(
    vscode.workspace.onDidChangeConfiguration(async (e) => {
      if (e.affectsConfiguration("gandora.doc.locale")) {
        await vscode.commands.executeCommand("gandora.restartServer");
      }
    })
  );

  // keep previews in sync with saves
  context.subscriptions.push(
    vscode.workspace.onDidSaveTextDocument((doc) => {
      if (doc.languageId !== "gandora") return;
      const refresh = vscode.workspace
        .getConfiguration("gandora")
        .get("preview.refreshOnSave");
      if (!refresh) return;
      for (const kind of ["compile", "expand"]) {
        provider.emitter.fire(previewUri(kind, doc.uri.fsPath));
      }
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("gandora.restartServer", async () => {
      if (client) {
        await client.stop();
        client = null;
      }
      client = await startClient(output);
      if (client) {
        vscode.window.setStatusBarMessage("Gandora: language server restarted", 3000);
      }
    }),

    vscode.commands.registerCommand("gandora.showCompiledPython", () =>
      openPreview(provider, "compile")
    ),

    vscode.commands.registerCommand("gandora.expandMacros", () =>
      openPreview(provider, "expand")
    ),

    vscode.commands.registerCommand("gandora.runFile", async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor || editor.document.languageId !== "gandora") return;
      if (editor.document.isDirty) await editor.document.save();
      const rel = vscode.workspace.asRelativePath(editor.document.uri, false);
      runInTerminal("gan run", `${ganCommand()} run ${rel}`);
    }),

    vscode.commands.registerCommand("gandora.checkProject", () =>
      runInTerminal("gan check", `${ganCommand()} check`)
    ),

    vscode.commands.registerCommand("gandora.runTests", () =>
      runInTerminal("gan test", `${ganCommand()} test`)
    ),

    vscode.commands.registerCommand("gandora.openRepl", () =>
      runInTerminal("gan repl", `${ganCommand()} repl`)
    )
  );

  // gan tasks: build / check / test / fmt --check with the problem matcher
  context.subscriptions.push(
    vscode.tasks.registerTaskProvider("gan", {
      provideTasks() {
        return ["build", "check", "test", "fmt --check src"].map((task) => {
          const t = new vscode.Task(
            { type: "gan", task },
            vscode.TaskScope.Workspace,
            task,
            "gan",
            new vscode.ShellExecution(`${ganCommand()} ${task}`),
            "$gandora"
          );
          if (task === "build") t.group = vscode.TaskGroup.Build;
          if (task === "test" || task === "check") t.group = vscode.TaskGroup.Test;
          return t;
        });
      },
      resolveTask(task) {
        const def = task.definition;
        if (!def || !def.task) return undefined;
        return new vscode.Task(
          def,
          vscode.TaskScope.Workspace,
          def.task,
          "gan",
          new vscode.ShellExecution(`${ganCommand()} ${def.task}`),
          "$gandora"
        );
      },
    })
  );
}

function deactivate() {
  return client ? client.stop() : undefined;
}

module.exports = { activate, deactivate };
