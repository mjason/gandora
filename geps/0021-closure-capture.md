---
gep: 21
title: Closure Capture Semantics
description: Closures capture the creation-time values of enclosing locals, realized as keyword-only default-argument snapshots.
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
  zh: local/zh/0021-closure-capture.md
---

# GEP-0021: Closure Capture Semantics

## Abstract

Elixir bindings are immutable: a closure sees the values its free
variables had when the closure was created, and a later rebinding
cannot reach inside it. Python lambdas capture *variables*, not
values. The compiler closes this gap with Python's canonical idiom —
default-argument snapshots, keyword-only so arity stays strict:

```elixir
x = 1
f = fn -> x end
x = 2
f.()                 # 1, as in Elixir
```

```python
x = 1
f = lambda *, x=x: x
x = 2
f()                  # 1
```

Without this, tail-call optimization (GEP-0019) amplifies the gap
into wrong answers: rebinding parameters in a `while` loop mutates
what every escaped closure sees.

## Motivation

`collect(3, [])` below must return closures over 3, 2, 1 — each
iteration's own binding, exactly what real recursion's frames give:

```elixir
def collect(0, acc), do: acc
def collect(n, acc), do: collect(n - 1, [fn -> n end] ++ acc)
```

Compiled to a TCO loop with plain lambdas, all closures shared one
rebound `n`. The same divergence exists without TCO whenever a
variable is rebound after a closure captures it, and in `for`
comprehension bodies (Python comprehension variables are late-bound
too). One rule fixes the whole class at the source: capture is
by value, at creation.

## Scope

Compilation of `fn` and `&` capture expressions. Free-variable
analysis is per enclosing function scope; module-level names
(functions, imports) are stable and are not snapshot.

## Specification

**GEP-0021-R001:** A closure's free variables that are locals of the
enclosing function scope (parameters, `=`/`<-` pattern bindings,
`->` clause bindings) are captured **by value at creation time**.
Realization: each such name becomes a keyword-only default argument
`name=name` on the generated lambda or hoisted `def`. Names the
closure itself binds — its parameters, its body's assignments, which
Python scoping already makes local — are not captures. Module-level
names are not captures.

**GEP-0021-R002:** Snapshots MUST NOT change the closure's calling
arity: they follow a bare `*` (`lambda x, *, n=n:`,
`def _gan_fn0(*_gan_args, a=a):`), so an extra positional argument
raises `TypeError` exactly as before.

**GEP-0021-R003:** The rule applies uniformly: single-expression
lambdas, hoisted multi-clause `fn` defs, `&(...)` captures, closures
in `for` comprehension bodies (the snapshot default is evaluated in
the comprehension scope, per iteration), and nested closures (each
level snapshots from the scope above, so chains stay faithful).

**GEP-0021-R004:** With GEP-0019 loops the snapshot restores frame
identity: a closure created in one iteration keeps that iteration's
values across later rebinds. Tail-call optimization therefore never
needs to be skipped and `recur` stays unrestricted.

## Rationale

The alternatives were rejected for changing more than they fix.
Skipping TCO when a closure captures a parameter preserves semantics
by giving up the constant-stack guarantee — a silent downgrade the
explicit `recur` contract cannot tolerate. SSA-style renaming of
rebound locals makes generated Python unrecognizable. The
default-argument snapshot is the idiom every Python reviewer already
knows for exactly this problem, costs one token per captured name,
and keeps the zero-runtime promise.

## Backwards Compatibility

Programs whose closures never observed a rebinding are byte-for-byte
unchanged in behavior; generated signatures gain keyword-only
defaults. Programs that did observe one were diverging from Elixir —
this is the fix.

## Security and Determinism

None beyond GEP-0001-R024; output remains deterministic.

## Tooling and AI Usage

Agents SHOULD treat closures as value snapshots (Elixir semantics)
and MUST NOT rely on Python late-binding behavior through generated
code.

## Rejected Alternatives

### Skip TCO for capturing functions

Safe but silently stack-bound; contradicts `recur`'s checked
constant-stack assertion (GEP-0019-R005).

### SSA renaming of rebound locals

Correct and general, but the generated Python stops looking like
what a reviewer would write.

## Conformance

Tests MUST cover: rebinding after creation in straight-line code;
closures escaping a TCO loop (per-iteration values); multi-clause
hoisted `fn`; `&` captures; comprehension-body closures; nested
closures; arity strictness with snapshots present; and a
module-level name left unsnapshot.

## Change History

- Revision 1, 2026-08-03: Initial version.
