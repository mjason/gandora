# Gandora for VS Code

Language support for `.gan` files:

- **LSP diagnostics** via `gan lsp` (install `gandora-tool` +
  `gandora-lsp`; the client only spawns the command)
- **Full syntax highlighting**, including **embedded-language
  injection** for the `~<lang>` sigil family (GEP-0009): `~python`
  bodies highlight as Python, `~sql` as SQL, `~markdown`/`~html`/
  `~json`/`~jsonc`/`~toml`/`~gan` likewise, with `<%= ... %>` splices
  switching back to Gandora **at any depth** — even inside the
  embedded language's own strings (an injection grammar, the ERB/EEx
  technique) — and `@doc`/`@example` heredocs render as Markdown with
  `gan>` doctest lines highlighted as Gandora.

`~toml` colors need a TOML grammar in your editor (e.g. the
Even Better TOML extension); the other embedded languages are
VS Code built-ins.

Install: grab the `.vsix` from the GitHub release (or `npm install &&
npx @vscode/vsce package` here) and run **Extensions: Install from
VSIX...**.
