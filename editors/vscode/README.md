# Gandora for VS Code

Full language support for `.gan` files.

## Language intelligence (`gan lsp`)

- **Diagnostics as you type** — compile errors and the GEP-0022
  compiler lints (undefined variable, unused binding, unreachable
  clause, discarded comprehension, dead defp, stack recursion), with
  **quick fixes**: one click inserts `@allow :stack_recursion` /
  `@allow :unused_function` or the `_` prefix.
- **Hover** — bilingual docs, `@spec`s, parameter tables, examples,
  and the **compiled recursion shape** (♻ tail recursion → `while`
  loop vs ⚠ native call stack, GEP-0019-R006); Python-side docs via
  jedi for `$module` references; inferred types for local variables.
- **Go to definition · Find references · Rename** — project-wide,
  alias- and `&`-capture-aware, string interpolations included.
- **Workspace symbol search** (Ctrl+T), document outline, completion,
  signature help, and formatting through the GEP-0016 engine.

## Commands (palette: `Gandora:`)

- **Run Current File** — ▶ button in the editor title
- **Show Compiled Python** — the generated Python side by side,
  refreshed on every save (the zero-runtime promise, visible)
- **Expand Macros** — the post-expansion quoted AST as JSON
- **Check Project / Run Doctests / Open REPL / Restart Language Server**

## Tasks

`Terminal → Run Task → gan`: `build`, `check`, `test`,
`fmt --check src` — wired to a problem matcher, so findings land in
the Problems panel.

## Editing

- **Snippets**: `defmodule`, `defd` (the full @doc/@param/@spec
  annotation standard), `case`/`cond`/`with`/`for`/`forinto`/`fn`,
  `recurdef` (accumulator tail recursion), `@example` doctests,
  `@allow`, `pyimport`, `@decorate`, and more.
- **Indentation rules** for `do/end`, `fn`, `->`, `else/rescue/after`;
  word pattern includes `?`/`!` names.
- **Full syntax highlighting**, including **embedded-language
  injection** for the `~<lang>` sigil family (GEP-0009): `~python`
  bodies highlight as Python, `~sql` as SQL, `~markdown`/`~html`/
  `~json`/`~jsonc`/`~toml`/`~gan` likewise, with `<%= ... %>` splices
  switching back to Gandora at any depth, and `@doc`/`@example`
  heredocs rendering as Markdown with `gan>` doctest lines
  highlighted as Gandora.

`~toml` colors need a TOML grammar in your editor (e.g. the
Even Better TOML extension); the other embedded languages are
VS Code built-ins.

## Setup

The client spawns `gan lsp`. Install the toolchain first:
`uv tool install gandora-tool gandora-lsp` (or add
`gandora-tool[dev]` to the project's dev dependencies), or point
`gandora.gan.path` at the runner.

Install the extension: grab the `.vsix` from the GitHub release (or
`npm install && npx @vscode/vsce package` here) and run
**Extensions: Install from VSIX...**.
