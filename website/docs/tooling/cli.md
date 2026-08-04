# CLI & Editors

## `gan` — the task runner

```console
gan build [--strict]     # the verdict + compile to outDir
gan run <file> [args]    # compile to .gandora/cache and execute
gan test                 # @example doctests + tests/*.gan (pytest under the hood)
gan fmt [--check|--diff] # canonical formatting; `-` reads stdin
gan doc Enum.take        # docs in the terminal (+ --locale zh-CN)
gan repl                 # interactive; state carries across lines
gan exec "1 |> to_string()"
gan init my-app          # new uv-compatible project
```

`gan run` executes with the *project* Python — `.venv/bin/python` when
present — so interop sees your real dependencies. Unknown subcommands
delegate to `gan-<name>` plugins, then to `ganc`.

## `ganc` — the stage-0 compiler

The Rust compiler underneath: `ganc build`, `ganc run`, `ganc test`,
`ganc expand` (macro output), `ganc compile`. Plumbing — it does not
gate on the verdict; `gan` is the porcelain that does.

## `gan lsc` — language intelligence as JSON

Every fact the toolchain knows, one JSON value per query — built for
agents and shells:

```console
gan lsc check --root .          # the verdict: {ok, clean, diagnostics, suggestions}
gan lsc doc Enum.map            # docs: spec, prose, translations, examples
gan lsc doc for                 # language-construct cards (for, spec, with, ...)
gan lsc symbols Enum            # every function with rendered heads
gan lsc references Stats.mean   # project-wide call sites
gan lsc compile src/x.gan       # the generated Python, as text
gan lsc expand src/x.gan        # post-macro AST
gan lsc pydoc numpy.array       # Python-side docs via jedi
```

## The language server

`gan-lsp` speaks LSP over stdio: diagnostics on every edit, hover
(docs, types, the compiled recursion shape ♻/⚠, localized parameter
tables), go-to-definition, references, rename, workspace symbols,
completion, signature help, quick fixes (`@allow` inserts, `_`
prefixes), and whole-document formatting.

**VS Code**: install `gandora-<version>.vsix` from the
[releases page](https://github.com/mjason/gandora/releases) — LSP
client plus command palette (Run File, Show Compiled Python with
refresh-on-save, Expand Macros, Build, Test, REPL), snippets, tasks
with a problem matcher.

## Documentation language

Docs are bilingual at the source (`@doc` + `@doc_trans`). What you
*see* follows your preference: `gandora.local.jsonc` (per-developer,
gitignored) `{"docLocale": "zh-CN"}` → the `GAN_DOC_LOCALE`
environment variable → English. `--locale all` shows every language.
