---
gep: 11
title: Multi-Arity Functions
description: One name, many arities — clauses of different arities compile into a single dispatching Python function, and Elixir default parameters (\\) desugar into delegating clauses.
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
created: 2026-08-02
updated: 2026-08-02
revision: 1
requires: [1]
replaces: []
superseded-by: null
resolution: null
translations:
  zh: local/zh/0011-multi-arity-functions.md
---

# GEP-0011: Multi-Arity Functions

## Abstract

In Elixir, `get/2` and `get/3` are one idiom: the same name at
different arities, usually produced by default parameters
(`def get(m, key, default \\ nil)`). Gandora previously compiled each
(name, arity) group to its own Python `def`, so a second arity
silently shadowed the first. This proposal merges every clause of one
name — across arities — into a single generated Python function that
dispatches on the argument tuple (whose length distinguishes arity for
free), and adds the `\\` default-parameter syntax that desugars into
shorter-arity delegating clauses.

## Motivation

The standard library had to avoid `get/2` + `get/3` style APIs
(GEP-0010's review noted the gap), and any Elixir codebase pasted into
Gandora breaks on its first default parameter. The fix is cheap
because clause dispatch already matches on the whole argument tuple:
tuples of different lengths never collide, so multi-arity is the
existing mechanism minus an artificial restriction.

## Scope

Function definitions (`def`/`defp`). Macro arities already coexist
(they are keyed by name and arity at expansion); default parameters
for macros are deferred.

## Terminology

- **Name group**: every `def`/`defp` clause sharing one function name
  in a module, regardless of arity.
- **Default parameter**: a head parameter written `name \\ expr`.

## Specification

**GEP-0011-R001:** All clauses of a name group compile into one
generated Python function. Dispatch tries clauses in source order;
an argument tuple only matches clauses of its own length, so arities
never interfere. This amends GEP-0001-R006's multi-clause bullet to
span arities.

**GEP-0011-R002:** A head parameter MAY carry a default: `p \\ expr`.
Defaults MUST be trailing. A definition with `k` defaults defines the
arities `n-k .. n`: for each omitted suffix the compiler synthesizes a
delegating clause that calls the name with the missing defaults
appended. Default expressions are compiled at the delegation site and
evaluated at call time, left to right, only when used.

**GEP-0011-R003:** Within a name group, at most one definition may
declare defaults (Elixir's rule); a violation, a non-trailing default,
and mixing `def` with `defp` in one group are compile errors naming
the function.

**GEP-0011-R004:** `@doc`/`@example`/`@decorate` attach to the name
group via its first clause, as before (GEP-0007). The no-match error
MUST name the function with its defined arities (e.g.
`no clause of get/2,3 matched`). References that name an arity
(`&get/3`, `gan doc`) resolve to the single generated callable.

**GEP-0011-R005:** Generated shape: a name group whose only clause has
plain variable parameters and no default keeps the direct `def`; any
group with several clauses, patterns, guards, or defaults compiles to
the `*args` match dispatcher. Determinism (GEP-0001-R024) is
unchanged.

## Rationale

Reusing tuple dispatch means arity merging adds no new runtime shape —
it deletes the (name, arity) partition that created shadowing.
Desugaring `\\` into delegation (rather than Python default values)
keeps Elixir's semantics exactly: defaults may be arbitrary
expressions evaluated per call, and each shorter arity is a real
clause visible to dispatch.

## Backwards Compatibility

Previously-shadowing programs (same name, two arities) change from
silently wrong to correct. `\\` was previously a lex error. No other
surface changes.

## Security and Determinism

No new evaluation phases; delegation clauses are ordinary compiled
code.

## Tooling and AI Usage

Agents can now port Elixir signatures verbatim (`def get(m, key,
default \\ nil)`) and should prefer defaults over hand-written
delegating wrappers. The stdlib adopts them where Elixir has them
(GEP-0010 revisions).

## Rejected Alternatives

### Python default parameter values

`def get(m, key, default=None)` evaluates defaults once at import (the
classic mutable-default trap) and cannot express arity-specific
clauses; delegation preserves Elixir semantics.

### Name mangling per arity (get__2, get__3)

Breaks the public-name contract (GEP-0001-R015), captures (`&get/3`),
and Python-side callers.

## Open Questions

None for this revision.

## Conformance

Tests MUST cover: two hand-written arities of one name dispatching
correctly; defaults synthesizing every shorter arity (call-time
evaluation included); the three R003 diagnostics; docstring and
decorator attachment on the group; the no-match message listing
arities; and a stdlib function using a default (`Map.get/3`).

## Change History

- Revision 1, 2026-08-02: Initial version.
