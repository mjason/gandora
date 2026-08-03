---
gep: 19
title: Self-Recursion Tail-Call Optimization
description: Tail-position self-calls compile to loops — Elixir's natural recursive style runs in constant stack on a VM without TCO.
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
created: 2026-08-03
updated: 2026-08-03
revision: 2
requires: [1, 14]
replaces: []
superseded-by: null
resolution: null
translations:
  zh: local/zh/0019-tail-call-optimization.md
---

# GEP-0019: Self-Recursion Tail-Call Optimization

## Abstract

A call to the enclosing function in tail position compiles to
parameter rebinding and a loop:

```elixir
def sum_to(0, acc), do: acc
def sum_to(n, acc), do: sum_to(n - 1, acc + n)
```

runs in constant stack for any `n`, and `recur(args)` in a function
body is the explicit, compiler-checked spelling of the same jump
(R005). Elixir's natural recursive style becomes safe on Python's
~1000-frame limit. Non-tail recursion (`n * fact(n - 1)`) is
untouched — it needs its stack.

## Motivation

GEP-0014 gave long-running code `loop/recur` because Python has no
TCO, but idiomatic Elixir is written as tail recursion, and porting
it today means rewriting every such function. Self-recursive tail
calls are statically recognizable and compile to the same `while`
shape `loop` already uses — the natural style should simply work.

## Scope

Direct self-recursion only. Mutual recursion stays call-stack-bound
(the trampoline was rejected in GEP-0014 and stays rejected).

## Specification

**GEP-0019-R001:** A call is in tail position when it is the final
expression of a clause body, of a branch of `if`/`unless`/`case`/
`cond` in tail position, or of a block in tail position. Expressions
inside `try`, anonymous functions, and argument positions are
never tail positions.

**GEP-0019-R002:** When any clause of a definition group contains a
tail call to the group's own name with an arity the group defines,
the group compiles to a loop: the dispatch (or the single clause's
body) is wrapped in `while True:`, and each tail self-call becomes a
rebinding of the parameter tuple plus `continue`. All other exits
`return` as before. Observable behavior is unchanged except stack
depth; a million-iteration self-recursion MUST pass in tests.

**GEP-0019-R003:** Self-calls not in tail position compile as
ordinary calls. The optimization never changes evaluation order or
argument evaluation semantics; arguments are evaluated fully before
rebinding.

**GEP-0019-R004:** The optimization applies to `def`, `defp`, and
default-parameter delegates alike, and composes with `@spec`
annotations (GEP-0017) unchanged.

**GEP-0019-R005:** `recur(args)` in a function body is the
**explicit** spelling of the same jump: it rebinds the
enclosing function's parameters and restarts it. It MUST be in tail
position and its arity MUST match a clause of the group — both are
compile errors otherwise, which is the guarantee the implicit form
cannot give (a refactor that moves a call out of tail position simply
loses the optimization silently; a moved `recur` breaks the build).
Division of labor: natural Elixir tail recursion is optimized
implicitly for portability; `recur` is the spelling that *asserts*
constant stack.

## Rationale

Rebinding-plus-`continue` is exactly the compilation `loop/recur`
already has; extending it to tail self-calls adds no new runtime
shape, keeps generated code readable (`while True:` + `match`), and
makes the dominant Elixir idiom portable verbatim. Mutual recursion
is excluded because it requires either a trampoline (rejected:
opaque, slower) or whole-program call-graph rewriting.

## Backwards Compatibility

Semantics-preserving; only stack consumption changes.

## Security and Determinism

None beyond GEP-0001-R024; output remains deterministic.

## Tooling and AI Usage

Agents SHOULD write natural tail recursion for unbounded iteration,
use function-level `recur` when constant stack is a requirement worth
asserting, and reach for `for` (GEP-0020) or Enum when the iteration
is a mapping rather than a state machine.

## Rejected Alternatives

### Trampolining for mutual recursion

Re-rejected per GEP-0014: hides the mechanism, costs allocation per
hop, and reads poorly in generated output.

### A `@tailrec`-style opt-in annotation

The analysis is static and safe; requiring an annotation would make
the natural spelling silently stack-bound when forgotten. The
assertion role an annotation would play is filled by explicit
`recur` (R005), which is checked rather than advisory.

## Conformance

Tests MUST cover: single-clause and multi-clause tail recursion at a
million iterations; guarded clauses; tail calls under `if`/`case`/
`cond`; a non-tail self-call left unoptimized (observable via
recursion limit); defaults interacting with rebinding; and identical
results before and after optimization for a reference function.

## Change History

- Revision 2, 2026-08-03: `loop` retired (GEP-0014-R007) — removed it
  from R001's exclusion list, R005's scoping language, and the
  tooling advice; iteration guidance now points at `for`/Enum.
- Revision 1, 2026-08-03: Initial version.
