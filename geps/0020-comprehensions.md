---
gep: 20
title: Comprehensions
description: Elixir's for comprehension — generators, filters, and into — compiled to Python comprehensions.
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
created: 2026-08-03
updated: 2026-08-03
revision: 1
requires: [1, 19]
replaces: []
superseded-by: null
resolution: null
translations:
  zh: local/zh/0020-comprehensions.md
---

# GEP-0020: Comprehensions

## Abstract

Elixir's `for`:

```elixir
for x <- xs, x > 0, y <- ys, do: {x, y}
```

compiles to the Python comprehension
`[(x, y) for x in xs if x > 0 for y in ys]`. Generators bind
patterns, bare expressions between clauses are filters, and
`into: %{}` produces a dict comprehension. With GEP-0019 recursion
and Enum, this completes the iteration story that lets `loop` retire
(GEP-0014 revision 2).

## Motivation

`for` is core Elixir surface (GEP-0001-R005 demands the surface where
supported) and maps one-to-one onto Python's best-loved construct —
the compiled output is exactly what a Python reviewer would have
written by hand.

## Scope

List and map comprehensions with generators, filters, and `into:`.
`reduce:`, `uniq:`, and binary generators are future revisions.

## Specification

**GEP-0020-R001:** `for gen_or_filter, ..., do: body` is an
expression. Each `pattern <- enumerable` is a generator; any other
clause expression is a filter evaluated under the bindings so far.
Clauses nest left to right. The `do:` body (shorthand or block)
produces one element per passing combination.

**GEP-0020-R002:** A generator pattern that is a plain variable binds
directly (`for x in xs`). Any other pattern compiles to a match that
**skips non-matching elements** (Elixir semantics), realized as a
filter on a match guard — never a `GanMatchError`.

**GEP-0020-R003:** Without `into:`, the result is a list, compiled to
a list comprehension. `into: %{}` requires the body to be a two-tuple
`{k, v}` and compiles to a dict comprehension. Other `into:` targets
are compile errors naming this rule.

**GEP-0020-R004:** Comprehension variables are scoped to the
comprehension (Python 3 comprehension scoping); bindings do not leak.
A comprehension body is not a tail position (GEP-0019-R001).

## Rationale

Compiling to native comprehensions keeps the zero-runtime promise and
peak readability at once. Pattern-skipping generators need care: a
tuple pattern becomes a structural guard, matching Elixir's
filter-not-crash semantics.

## Backwards Compatibility

`for` was previously an unsupported-construct diagnostic. Additive.

## Security and Determinism

Local control flow only; deterministic output.

## Tooling and AI Usage

Agents SHOULD prefer `for` over `Enum.map`/`filter` chains when a
single comprehension reads better, and MUST NOT use `for` for side
effects (use `Enum.each`).

## Rejected Alternatives

### Desugaring to Enum calls

`for x <- xs, do: f(x)` as `Enum.map` costs a std dependency in the
output where a native comprehension is clearer and faster.

## Conformance

Tests MUST cover: single and multiple generators, filters between
and after generators, pattern generators that skip, `into: %{}`,
nested comprehensions, non-leaking scope, and the non-tuple-body
`into: %{}` error.

## Change History

- Revision 1, 2026-08-03: Initial version.
