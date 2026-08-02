# Gandora for VS Code

Language support for `.gan` files:

- **LSP diagnostics** via `gan lsp` (install `gandora-tool` +
  `gandora-lsp`; the client only spawns the command)
- **Full syntax highlighting**, including **embedded-language
  injection** for the `~<lang>` sigil family (GEP-0009): `~python`
  bodies highlight as Python, `~sql` as SQL, `~markdown`/`~html`/
  `~json` likewise, with `<%= ... %>` splices switching back to
  Gandora inside any of them — and `@doc`/`@example` heredocs render
  as Markdown with `gan>` doctest lines marked.

Install: grab the `.vsix` from the GitHub release (or `npm install &&
npx @vscode/vsce package` here) and run **Extensions: Install from
VSIX...**.
