---
gep: 10
title: The Standard Library
description: The standard library as an ordinary Gandora package — gandora-std — written in Gandora, versioned independently of the compiler, resolved through package markers.
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

The standard library — `Enum`, `String`, `Map`, `List`, `Keyword` — is
an ordinary Gandora package, `gandora-std`, written in Gandora and
published through the GEP-0006 channel. It is not embedded in the
compiler: projects install it explicitly (`uv add gandora-std`),
version it in `pyproject.toml` like any dependency, and upgrade it
without touching the compiler. Two small, general mechanisms make the
nice names work: a package may compile its output under a Python
package prefix (`pyPackage`), and marker module names are authoritative
for runtime resolution — so `Enum.map(xs, f)` compiles to
`gandora_std.enum.map(xs, f)` while shadowing no Python module. The
library is bilingual and doctested through GEP-0007; `gan doc Enum.map
--locale zh` works once the package is installed.

## Motivation

Interop makes every Python API reachable, but Elixir developers and AI
agents reach for `xs |> Enum.map(f) |> Enum.filter(p)` by instinct, and
Python's function-first APIs (`map(f, xs)`) read backwards in
pipelines. A data-first stdlib restores the idiom. Writing it in
Gandora proves the language hosts its own library layer; shipping it as
a package keeps the compiler free of embedded content, gives the
library its own release cadence, and makes it exactly as inspectable,
documentable, and testable as any user package — its functions are
living examples of idiomatic Gandora.

## Scope

The package, the `pyPackage` output prefix, marker-based runtime
resolution, and the initial module set. Lazy streams, protocols-based Enumerable,
`Kernel` migration of the GEP-0001 builtins, and stdlib macros are
deferred.

## Terminology

- **Stdlib module**: a single-segment CamelCase module (`Enum`,
  `String`, ...) provided by the `gandora-std` package.
- **Output prefix**: the Python package a project's compiled modules
  are placed under, configured by `pyPackage`.

## Specification

**GEP-0010-R001:** The standard library is the Gandora package
`gandora-std`, authored in Gandora, published and consumed through
GEP-0006 with no special-casing in the compiler. It lives in the
compiler repository under `std/` and is released to PyPI in lockstep
with each compiler release, and `gan init` SHOULD add
it to new projects' dependencies once published there — but it remains
an ordinary, explicitly declared, independently upgradable dependency.

**GEP-0010-R002:** `gandora.jsonc` accepts an optional `pyPackage`
string (amending GEP-0001-R019): compiled output lands under that
Python package (`enum.gan` → `gandora_std/enum.py` when
`"pyPackage": "gandora_std"`), and sibling references resolve within
it. `gandora-std` uses the prefix so its modules never shadow Python's
(`enum`, `string`).

**GEP-0010-R003:** Marker module names are authoritative for runtime
resolution (GEP-0006-R005A): a reference to a module not defined in the
project resolves through the installed markers to its recorded Python
path, then falls back to the mechanical GEP-0001-R014 mapping. Project
modules always win over installed names. Thus `Enum.map(xs, f)`
compiles to `gandora_std.enum.map(xs, f)` after `uv add gandora-std`.

**GEP-0010-R004:** Stdlib functions are data-first: the subject is the
first parameter, so every function is `|>`-ready. Semantics follow
Elixir where the GEP-0001-R009 data mapping permits, and Python
otherwise; each divergence MUST be recorded in the function's `@doc`.

**GEP-0010-R005:** The initial set: `Enum` (map, filter, reject,
reduce, sum, count, sort, sort_by, reverse, join, at, take, drop, zip,
with_index, member?, all?, any?, empty?, uniq, flat_map, each, min,
max), `String` (upcase, downcase, capitalize, split, split_on, trim,
replace, contains?, starts_with?, ends_with?, length, slice,
pad_leading, pad_trailing, to_integer, to_float), `Map` (get, put,
delete, keys, values, merge, has_key?, to_list, new, update), `List`
(first, last, flatten, wrap, duplicate, insert_at, delete_at),
`Keyword` (get, put, keys, values, has_key?). Additions accumulate
under this GEP by revision.

**GEP-0010-R006:** Every stdlib function MUST carry a default `@doc`,
a `zh-CN` `@doc_trans`, and — for functions whose behavior is not
obvious from the name — an `@example` doctest. `gan doc` resolves the
installed package's shipped sources (GEP-0007-R009); the package's own
`gan test` runs every doctest.

**GEP-0010-R007:** The GEP-0001-R006 Kernel-style builtins (`length`,
`hd`, `div`, ...) remain compiler-inlined; the stdlib does not wrap
them.

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

A library embedded in the compiler couples every library fix to a
compiler release and blurs the boundary that keeps both reviewable —
the package boundary is the discipline. As an ordinary dependency the
stdlib is pinned, diffed, and upgraded by `uv` like everything else,
and the no-hidden-runtime property holds in its strongest form: the
only Gandora-related thing a deployment contains is what
`pyproject.toml` declares.

The two supporting mechanisms are deliberately general rather than
stdlib-specific: any package can claim an output prefix, and any
package's marker names already had to be authoritative for macros —
extending that to runtime resolution completes GEP-0006 symmetrically.

## Backwards Compatibility

Additive: `pyPackage` is a new optional field; marker-based runtime
resolution only affects references that previously failed at runtime.
Single-segment stdlib names are claimed by convention, not compiler
reservation — a project module named `Enum` simply shadows the package
(project modules win).

## Security and Determinism

Marker-based resolution reads static files only (GEP-0006-R006's
rule). The stdlib compiles like any package; nothing executes at
compile time.

## Tooling and AI Usage

Agents should prefer stdlib calls over raw interop for list/string/map
work (`Enum.map(xs, f)` over `:builtins.map`), read semantics with
`gan doc Enum.sort_by`, and treat the stdlib sources as the canonical
idiom reference.

## Rejected Alternatives

### Embedding the stdlib in the compiler binary

Couples library releases to compiler releases, grows the trusted
artifact with content, and blurs the compiler/library boundary — the
accretion path. Rejected in review of revision 1's draft.

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

Tests MUST cover: `pyPackage` output placement and sibling
resolution; marker-based runtime resolution with project-module
precedence; pipe usage of each module against an installed
`gandora-std`; `gan doc` on stdlib targets in both locales; and the
package's own `gan test` passing every doctest.

## Change History

- Revision 1, 2026-08-02: Initial version.
