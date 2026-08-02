---
gep: 15
title: The Language Server
description: gan-lsp — diagnostics, documentation hover, definition, symbols, completion, and formatting, written in Gandora on pygls; plus gan-lsc, the isomorphic JSON command line for AI agents.
author: MJ
status: Accepted
type: Standards Track
areas:
  - Tooling
created: 2026-08-02
updated: 2026-08-02
revision: 5
requires: [12, 13, 14]
replaces: []
superseded-by: null
resolution: null
translations:
  zh: local/zh/0015-the-language-server.md
---

# GEP-0015: The Language Server

## Abstract

`gan-lsp` is a Language Server Protocol implementation written in
Gandora: a `gan lsp` plugin (GEP-0013-R003) whose distribution is the
`gandora-lsp` package. Version 1 implements the protocol lifecycle and
push diagnostics — every edit is compiled in-memory through
`gandora_core.diagnostics` and errors/warnings appear in the editor
with their spans. It is the reference ecosystem tool: a long-running
Gandora server built on `loop` (GEP-0014), `try/rescue`, interop byte
I/O, and the compiler library.

## Motivation

Editor diagnostics are the highest-value language service and require
no symbol infrastructure — exactly the compiler library's shape today.
Writing the server in Gandora continues the GEP-0012 program: the
toolchain is the language's proof of completeness, and the LSP is its
first long-running process.

## Scope

Protocol framing, lifecycle, diagnostics, documentation hover,
go-to-definition, document symbols, `Module.` completion, and
whole-document formatting. Rename and find-references await
span-range enrichment; they MUST reuse the same library queries.

## Specification

**GEP-0015-R001:** The server is written in Gandora on **pygls**: the
framework owns protocol machinery (framing, JSON-RPC, typed
`lsprotocol` structures) and the Gandora module owns language logic,
attaching handlers with `@decorate @server.feature(...)` on a
`LanguageServer` held in a module attribute. It is distributed as
`gandora-lsp` exposing the `gan-lsp` entry point, so `gan lsp`
reaches it through plugin delegation. pygls is a dependency of this
package only — the core toolchain stays dependency-lean.

**GEP-0015-R001A:** The same package exposes `gan-lsc` — the Language
Server Console, the AI-facing isomorphic surface. Every query prints
exactly one JSON value on stdout (quoted-term tuples appear as JSON
arrays) and exits per GEP-0001-R023. Version 2 queries: `version`,
`diagnostics <file>`, `ast <file>`, `expand <file>`,
`compile <file>`, `resolve <module>`, each accepting `--root <dir>`.
`gan lsc ...` reaches it through the same delegation. LSP
capabilities MUST remain expressible as lsc queries.

**GEP-0015-R002:** Version 1 handles: `initialize` (announcing full
text sync), `initialized`, `textDocument/didOpen`, `didChange`,
`didClose` (clearing diagnostics), `shutdown`, and `exit`. Unknown
requests receive MethodNotFound; unknown notifications are ignored.

**GEP-0015-R003:** On open and on every change, the server runs
`gandora_core.diagnostics(text, path, root)` — root from the
workspace folder — and publishes `textDocument/publishDiagnostics`,
mapping severities and 1-based compiler spans to 0-based zero-length
LSP ranges. A request that raises is answered with an error response
(or logged, for notifications); the server MUST NOT die on bad input
(GEP-0014 `try/rescue`).

**GEP-0015-R005:** `textDocument/hover` serves the GEP-0007
documentation channel: the reference under the cursor (a
`Module.function` chain, a module name, or a bare local name resolved
against the file's own module) is looked up through
`gandora_core.doc`, and the hover renders the clause signatures
(rendered heads, guards and defaults included), the default-locale
prose, every available translation, metadata, and `@example` blocks
as Markdown. A `$module` reference shows the module's spec origin
(never importing it); language constructs (`def`, `loop`, `quote`,
...) show embedded one-paragraph reference cards. Undocumented or
`@doc false` targets produce no hover.

**GEP-0015-R006:** `textDocument/definition` resolves the same cursor
targets through `gandora_core.definition` to the defining source —
the `defmodule` line for modules, the first matching clause for
functions — across project sources and installed packages' shipped
`.gan` sources.

**GEP-0015-R007:** `textDocument/formatting` runs the GEP-0016 engine
(`Fmt.format_text`) over the buffer and returns one whole-document
edit, guarded by the same GEP-0016-R006 verification; an unformattable
or unchanged buffer yields no edits.

**GEP-0015-R008:** `textDocument/documentSymbol` lists the module and
its definitions (rendered heads as detail); `textDocument/completion`
triggered on `.` after a module path completes its public functions
and macros with signatures and doc summaries, via
`gandora_core.symbols`.

**GEP-0015-R009:** The Python side of the boundary is first-class:
for `$module` references and `pyimport` aliases, hover shows the
Python docstring, completion lists module members, definition jumps
into the Python source, and signature help shows the full Python
signature — resolved by jedi in the project's own environment,
statically (the server never imports user modules to answer a query).

**GEP-0015-R010:** `textDocument/signatureHelp` (triggered on `(` and
`,`) serves the innermost open call: Gandora targets show every
clause head as a signature with per-parameter labels, `@spec` lines
(GEP-0017) as documentation, and the active parameter tracked by
argument position; Python targets are served per R009.

**GEP-0015-R004:** The repository ships a minimal VS Code client in
`editors/vscode` that spawns `gan lsp` for `.gan` files; it contains
no language logic.

## Rationale

pygls over hand-rolled framing trades a package-local dependency for
the LSP ecosystem's maintenance of protocol details — the revision-1
hand-rolled server proved the language could do it; the framework
makes the tool cheaper to grow. `lsc` exists so agents get language
intelligence as plain JSON without speaking JSON-RPC.

Diagnostics-first matches both user value and library maturity;
richer features await range spans rather than shipping half-wrong.
The plugin route means the runner needs no LSP knowledge, and any
editor speaking LSP needs only the `gan lsp` command.

## Backwards Compatibility

Additive; new distribution.

## Security and Determinism

The server compiles buffers it is sent and executes nothing.

## Tooling and AI Usage

Editors use `gan lsp`. AI agents should prefer `gandora_core`
directly; the LSP exists for humans' editors.

## Rejected Alternatives

### Implementing the server in Rust

Abandons the ecosystem proof; the library boundary exists to make
this a Gandora program.

## Open Questions

None for this revision.

## Conformance

Tests MUST cover a scripted stdio session: initialize handshake, a
didOpen with an erroneous buffer producing publishDiagnostics with
the right span, a didChange that fixes it producing an empty list,
didClose clearing, and shutdown/exit terminating with code 0.

## Change History

- Revision 5, 2026-08-02: Added R009 (jedi-backed Python-side hover/
  completion/definition/signatures) and R010 (signature help); hover
  shows GEP-0017 specs.

- Revision 4, 2026-08-02: Hover gained signatures, `$module` and
  construct cards (R005 expanded); added definition (R006),
  formatting through GEP-0016 (R007), and symbols/completion (R008),
  over new `gandora_core.definition`/`symbols` APIs.

- Revision 3, 2026-08-02: Added R005 — documentation hover over the
  `gandora_core.doc` lookup (new core API).

- Revision 2, 2026-08-02: Adopted pygls for protocol machinery; added
  the gan-lsc isomorphic JSON console (R001A).
- Revision 1, 2026-08-02: Initial version.
