---
gep: 10
title: The Standard Library
description: A self-hosted, compiler-embedded standard library — Enum, String, Map, List, Keyword — written in Gandora, compiled into each build, with no runtime package.
author: MJ
status: Accepted
type: Standards Track
areas:
  - Standard Library
created: 2026-08-02
updated: 2026-08-02
revision: 1
requires: [1, 3, 7]
replaces: []
superseded-by: null
resolution: null
translations:
  zh: local/zh/0010-standard-library.md
---

# GEP-0010: The Standard Library

## Abstract

Gandora ships an Elixir-shaped standard library — `Enum`, `String`,
`Map`, `List`, `Keyword` — written in Gandora itself and embedded in
the `gan` binary as source. When a build references a stdlib module,
the compiler compiles that module (from its embedded source, with its
own compiler) into the build's output under the reserved
`gandora_std/` package. Deployment therefore keeps the GEP-0001-R002
guarantee: generated code depends only on the project's own output,
never on an installed Gandora package. The library is bilingual and
doctested through GEP-0007, so `gan doc Enum.map --locale zh` works
out of the box.

## Motivation

Interop makes every Python API reachable, but Elixir developers and AI
agents reach for `xs |> Enum.map(f) |> Enum.filter(p)` by instinct, and
Python's function-first APIs (`map(f, xs)`) read backwards in
pipelines. A data-first stdlib restores the idiom. Writing it in
Gandora rather than Rust keeps the compiler small, proves the language can host its
own library layer, and makes every stdlib function a living example of
idiomatic Gandora — documented, translated, and doctested with the
language's own machinery.

## Scope

The embedding and resolution mechanism, the reserved output package,
and the initial module set. Lazy streams, protocols-based Enumerable,
`Kernel` migration of the GEP-0001 builtins, and stdlib macros are
deferred.

## Terminology

- **Stdlib module**: a single-segment CamelCase module (`Enum`,
  `String`, ...) whose source is embedded in the compiler.
- **Support package**: the `gandora_std/` Python package emitted into a
  build's output.

## Specification

**GEP-0010-R001:** Stdlib modules are authored in Gandora, embedded in
the `gan` binary as source, and compiled by the same pipeline as user
code. Their names MUST NOT be shadowable: a project module named like a
stdlib module is a compile error.

**GEP-0010-R002:** A stdlib module referenced by a build (via `alias`,
qualified calls, `import`, or another stdlib module) MUST be compiled
into the build's output as `gandora_std/<snake>.py`, transitively.
Unreferenced stdlib modules MUST NOT be emitted. The `gandora_std`
package name is reserved in output directories.

**GEP-0010-R003:** References compile to plain imports of the support
package (`Enum.map(xs, f)` → `gandora_std.enum.map(xs, f)`), preserving
GEP-0001-R002: no runtime package, no import hook; deployed output is
self-contained. The `gandora_std/` layer never shadows Python modules
(`enum`, `string`) because it is a package, not top-level modules.

**GEP-0010-R004:** Stdlib functions are data-first: the subject is the
first parameter, so every function is `|>`-ready. Semantics follow
Elixir where the GEP-0001-R009 data mapping permits, and Python
otherwise; each divergence MUST be recorded in the function's `@doc`.

**GEP-0010-R005:** The initial set: `Enum` (map, filter, reject,
reduce, sum, count, sort, sort_by, reverse, join, at, take, drop, zip,
with_index, member?, all?, any?, empty?, uniq, flat_map, each, min,
max), `String` (upcase, downcase, capitalize, split, trim, replace,
contains?, starts_with?, ends_with?, length, slice, pad_leading,
pad_trailing, to_integer, to_float), `Map` (get, put, delete, keys,
values, merge, has_key?, to_list, new, update), `List` (first, last,
flatten, wrap, duplicate, insert_at, delete_at), `Keyword` (get, put,
keys, values, has_key?). Additions accumulate under this GEP by
revision.

**GEP-0010-R006:** Every stdlib function MUST carry a default `@doc`,
a `zh-CN` `@doc_trans`, and — for functions whose behavior is not
obvious from the name — an `@example` doctest. `gan doc` MUST resolve
stdlib modules from the embedded sources without project context;
`gan test` runs the doctests of whichever stdlib modules a project
emits.

**GEP-0010-R007:** The GEP-0001-R006 Kernel-style builtins (`length`,
`hd`, `div`, ...) remain compiler-inlined; the stdlib does not wrap
them. This mirrors Elixir's Kernel-BIF split.

### Discipline

Standard libraries degrade by accretion — mixed tiers (macros,
functions, runtime helpers), partial ports tracked in coverage
matrices, scope growing per feature request. These rules exist to
prevent that failure mode and can be changed only by revising this
GEP:

**GEP-0010-R008:** The stdlib is one tier: plain, eager, data-first
functions. No stdlib macros, no runtime templates, no lazy variants,
no state. A capability needing any of those is a separate GEP, not a
stdlib addition.

**GEP-0010-R009:** Every stdlib function MUST be a thin wrapper — a
few lines over Python builtins/methods. A function needing real
algorithmic code does not belong in the stdlib; it belongs in a
package (GEP-0006).

**GEP-0010-R010:** The R005 list is the complete library. There is no
partial-parity tracking against Elixir: a function is either fully
present (documented, translated, doctested, Elixir-named,
subject-first) or absent. Additions land as GEP revisions extending
R005, never as unlisted code.

## Rationale

Self-hosting the library (not the compiler) is the highest-value
bootstrap: it exercises the whole surface — pipes, interop, docs,
doctests — on every build, while the Rust compiler stays the small
trusted core. Emitting compiled stdlib
into each build rather than shipping a `gandora-std` wheel keeps the
zero-runtime story and version-skew-free deployments: the library
version is pinned by the compiler that built the output.

The discipline section exists because library layers that blur into
compiler and macro layers become unreviewable. One tier, thin wrappers, and a closed list keep every
stdlib change a small, complete, reviewable unit.

## Backwards Compatibility

`gandora_std` becomes reserved in outputs; single-segment stdlib names
become reserved module names. Both were previously unlikely
collisions; the R001 diagnostic makes the reservation explicit.

## Security and Determinism

Embedded sources are part of the compiler artifact; identical compiler
versions emit identical support code. Compilation of stdlib modules
executes nothing, like all compilation.

## Tooling and AI Usage

Agents should prefer stdlib calls over raw interop for list/string/map
work (`Enum.map(xs, f)` over `:builtins.map`), read semantics with
`gan doc Enum.sort_by`, and treat the stdlib sources as the canonical
idiom reference.

## Rejected Alternatives

### A gandora-std PyPI wheel

Reintroduces a runtime dependency and compiler/library version skew;
the per-build support package costs a few kilobytes instead.

### Writing the stdlib in Rust codegen

Every function would grow the compiler and bypass the language's own
doc/test machinery; self-hosting keeps the core small and the library
honest.

### Top-level output modules (enum.py, string.py)

Would shadow Python's stdlib for sibling imports — the exact failure
the `-P` work fixed; a package namespace avoids the class of bug.

## Open Questions

None for this revision.

## Conformance

Tests MUST cover: emission on reference and non-emission otherwise;
transitive stdlib dependencies; the R001 shadowing diagnostic; pipe
usage of each module; `gan doc` on stdlib targets in both locales;
doctest execution of emitted stdlib modules; and a deployed-output run
with no Gandora installation.

## Change History

- Revision 1, 2026-08-02: Initial version.
