---
gep: 15
title: The Language Server
description: gan-lsp — an LSP server written in Gandora on pygls, plus gan-lsc, the isomorphic JSON command line for AI agents; both in one gan-plugin package.
author: MJ
status: Accepted
type: Standards Track
areas:
  - Tooling
created: 2026-08-02
updated: 2026-08-02
revision: 2
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

Protocol framing, lifecycle, and diagnostics. Hover, completion,
definition, and symbols are later revisions once span-range
enrichment lands; they MUST reuse the same library queries.

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

- Revision 2, 2026-08-02: Adopted pygls for protocol machinery; added
  the gan-lsc isomorphic JSON console (R001A).
- Revision 1, 2026-08-02: Initial version.
