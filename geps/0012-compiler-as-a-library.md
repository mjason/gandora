---
gep: 12
title: The Compiler as a Library
description: gandora-core — the compiler as a Rust library and Python extension module, with a fixed quoted-term encoding, so tooling (LSP, REPL, the task runner) is written in Gandora itself.
author: MJ
status: Accepted
type: Standards Track
areas:
  - Tooling
  - Interop
created: 2026-08-02
updated: 2026-08-02
revision: 1
requires: [1, 2, 6]
replaces: []
superseded-by: null
resolution: null
translations:
  zh: local/zh/0012-compiler-as-a-library.md
---

# GEP-0012: The Compiler as a Library

## Abstract

The compiler becomes three artifacts built from one Rust codebase: the
`gandora-core` library crate (the single source of language rules), the
`gan` binary (a thin CLI over it), and the `gandora-core` Python
extension wheel (PyO3/abi3) that any Python process — and therefore any
Gandora program, via interop — can import. `:gandora_core.parse(src)`
returns the quoted term as native Gandora data in the Elixir encoding,
so pattern matching works on the language's own syntax in-process. This
is the foundation on which the task runner, REPL, LSC, and LSP are
written in Gandora itself (GEP-0013+), the way `mix` is written in
Elixir and `cargo` in Rust.

## Motivation

Language intelligence must have exactly one implementation. Keeping it
reachable only through a binary forces tooling either into Rust
(closing the ecosystem) or into subprocess-and-JSON plumbing (slow,
schema-heavy). Elixir's answer — expose the compiler as a library
(`Code.string_to_quoted/1`) and build the toolchain in the language —
fits Gandora unusually well: the quoted AST shape is already a public
contract (GEP-0002-R002) and already maps onto Python data
(GEP-0001-R009). Exporting it is fulfilling existing promises, not
inventing a format.

## Scope

The three artifacts, the Python API surface, the quoted-term encoding,
and version discipline. The task runner, plugin protocol, REPL, and
LSP are GEP-0013+; control-flow additions they need are GEP-0014.

## Terminology

- **Core**: the `gandora-core` Rust library crate.
- **Extension**: the `gandora_core` Python module (cdylib wheel).
- **Quoted term**: a Gandora syntax tree in the GEP-0002-R002 shape,
  encoded per R005 below.

## Specification

### Artifacts

**GEP-0012-R001:** The repository is a Cargo workspace: `crates/core`
(the compiler as a library, no third-party dependencies),
`crates/gan` (the CLI binary, thin over core), and `crates/core-py`
(the PyO3 bindings). Every language rule — lexing, parsing, expansion,
resolution, code generation, diagnostics — lives in core only.

**GEP-0012-R002:** PyPI distributes `gandora-core` (abi3 cdylib wheel,
importable as `gandora_core`) alongside `gandora-lang` and
`gandora-std`, all released in lockstep with the same version number.

### The Python API

**GEP-0012-R003:** Version 1 of the extension exposes exactly:

- `version() -> str` — the core version.
- `parse(source, path="nofile")` — the quoted term of a source file.
- `expand(source, path="nofile", root=None)` — the quoted term after
  macro expansion; with `root`, project and installed macros resolve
  as in a build.
- `diagnostics(source, path="nofile", root=None)` — a list of
  `{message, line, col, severity}` dicts from the full pipeline
  (parse, expand, generate), `severity` being `"error"` or
  `"warning"`; an empty list means the source compiles.
- `compile_string(source, path="nofile", root=None)` — the generated
  Python source of one module.
- `compile_snippet(source, root=None)` — compiles a statement
  sequence for interactive use: returns Python code that executes the
  snippet and leaves the value of its final expression in the variable
  `_`; the REPL/`exec` primitive.
- `resolve(root, module_name)` — how a module reference resolves:
  `{kind: "project"|"installed"|"mechanical", python, source}` per
  GEP-0006-R005A precedence.

Errors raise `gandora_core.CompileError` carrying `message`, `path`,
`line`, and `col`. Additions to this surface land as revisions of this
GEP.

### The quoted-term encoding

**GEP-0012-R004:** Quoted terms are encoded in Elixir's own quoted
convention, realized in the GEP-0001-R009 data mapping:

| Term | Python encoding |
| --- | --- |
| integer, float, boolean, nil | itself |
| atom, plain string | `str` (the runtime mapping; see R006) |
| interpolated string | `("__interp__", meta, [part, ...])` |
| variable | `(name, meta, context)` — context `None` or an int |
| module alias | `("__aliases__", meta, [segment, ...])` |
| list | `list` |
| 2-tuple | Python 2-`tuple` |
| n-tuple (n ≠ 2) | `("{}", meta, [item, ...])` |
| map | `("%{}", meta, [(key, value), ...])` |
| keyword pair | `(key, value)` 2-tuple |
| local call | `(name, meta, [arg, ...])` |
| remote/method call | `((".", meta, [base, name]), meta, [arg, ...])` |
| anonymous call `f.(x)` | `((".", meta, [f]), meta, [arg, ...])` |
| block | `("__block__", meta, [stmt, ...])` |

`meta` is a dict with at least `line` and `col` (0 meaning unknown).
The 3-tuple is reserved for nodes; 2-tuples are always data — the same
disambiguation Elixir uses.

**GEP-0012-R005:** The encoding is a public compatibility contract at
the same level as GEP-0002-R002; changes require a revision here and
follow the version discipline of R007.

**GEP-0012-R006:** Recorded limitation: because atoms and plain
strings share one runtime representation, version 1 does not
distinguish `:abc` from `"abc"` in quoted terms. Tooling that needs
the distinction awaits a revision; the limitation MUST be stated in
the extension's documentation.

### Version discipline

**GEP-0012-R007:** Tools built on the extension MUST check
`gandora_core.version()` against the version they were built for and
warn on mismatch. Lockstep releases (one version across the three
distributions) remain the policy; the extension itself never breaks
the R003 surface within a major version.

## Rationale

One codebase compiled three ways removes source-level drift by
construction; what remains is distribution-level skew, which lockstep
versions plus a mandatory runtime check reduce to a warning instead of
silent divergence. Choosing Elixir's exact quoted conventions
(`__aliases__`, `{}`/`%{}` wrappers, 3-tuples-are-nodes) rather than a
bespoke schema means every Elixir metaprogramming intuition — and
every existing document about our own macro system — applies to the
exported data unchanged.

`compile_snippet` leaving its result in `_` mirrors the REPL
convention of every major language and keeps the primitive free of
I/O policy: callers decide how to display `_`.

## Backwards Compatibility

The `gan` CLI surface is unchanged; the binary becomes thin over core.
No existing GEP contract changes; this GEP adds artifacts.

## Security and Determinism

The extension compiles text to data and code as text; it executes
nothing (`compile_snippet` returns code — running it is the caller's
explicit act). Determinism guarantees of GEP-0001-R024 carry over
verbatim since it is the same code.

## Tooling and AI Usage

Gandora tooling should `pyimport gandora_core` and pattern-match
quoted terms directly. AI agents get the same power through
`:gandora_core` interop in `gan exec` snippets, or via the GEP-0013
`lsc` surface. Nobody should parse Gandora text with regexes again.

## Rejected Alternatives

### Subprocess + JSON AST export

Works, but pays serialization and process spawn per query, and forces
a second (JSON) schema beside the quoted encoding. The extension gives
the same single truth in-process; a CLI view survives in GEP-0013 as a
thin consumer for shells.

### Reimplementing analysis in Gandora over text

Two parsers, guaranteed drift — rejected throughout this GEP line.

### A bespoke AST schema

Elixir's conventions are proven, documented, and already ours by
inheritance (GEP-0002-R002); inventing differently would tax every
user twice.

## Open Questions

None for this revision.

## Conformance

Tests MUST cover: importing the extension from plain Python and from
Gandora interop; parse/expand/diagnostics/compile_string round-trips
against known sources; the encoding table entry by entry (including
node-vs-data tuple disambiguation and interpolated strings);
`compile_snippet` leaving `_`; `resolve` across the three kinds;
CompileError fields; and `version()` equality with the crate version.

## Change History

- Revision 1, 2026-08-02: Initial version.
