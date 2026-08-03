---
gep: 22
title: Compiler Lints
description: Rust-style compile warnings for statically provable unsafety — undefined variables, unused bindings, unreachable clauses, discarded comprehensions, dead private functions.
author: MJ
status: Accepted
type: Standards Track
areas:
  - Compiler
created: 2026-08-03
updated: 2026-08-03
revision: 1
requires: [1, 19, 20]
replaces: []
superseded-by: null
resolution: null
translations:
  zh: local/zh/0022-compiler-lints.md
---

# GEP-0022: Compiler Lints

## Abstract

The compiler warns, like rustc, when it can statically prove code is
unsafe or dead: a variable read that nothing binds (a guaranteed
`NameError`), a binding never used, a clause that can never match, a
comprehension whose result is discarded, a private function nothing
references. Warnings are spanned diagnostics — `gan check`,
`gan build`, and editor squiggles all point at the offending
definition — and each lint either has an idiomatic fix (`_` prefix)
or an explicit acknowledgment (`@allow`, GEP-0019-R007's mechanism).

## Motivation

Python defers name resolution to runtime, so a typo'd variable is a
crash in production, not a compile error — this bit the toolchain
itself (an LSP handler shipped with a `NameError` the compiler said
nothing about). Every lint here fires only on statically provable
facts; none is a style opinion.

## Scope

Function-level dataflow lints in the compiler. Cross-module analysis,
type-based lints, and configurable lint levels are future GEPs.

## Specification

**GEP-0022-R001 (undefined variable):** A variable read in a function
that no parameter, pattern, pyimport name, or module function binds is
a warning naming the variable. When the module uses `import Mod`
(bare names become statically unknowable) the lint stands down for
that module. Pattern binding occurrences are not reads; pins (`^x`)
and sigil-template splices (`<%= x %>`, GEP-0009-R002) are.

**GEP-0022-R002 (unused binding):** A variable bound (parameter or
pattern) but never read warns, suggesting the `_` prefix. Names
starting with `_` and macro-hygienic names are exempt. Use is judged
across the whole definition group — a read in any clause counts.

**GEP-0022-R003 (unreachable clause):** A guard-less clause whose
patterns are all bare variables matches every argument; same-arity
clauses after it — and `case` clauses after a guard-less
capture/wildcard — can never run, and warn.

**GEP-0022-R004 (discarded comprehension):** A `for` comprehension in
statement position builds a collection nobody reads; agents and
humans alike MUST use `Enum.each` for side effects (GEP-0020). The
compiler warns at the comprehension.

**GEP-0022-R005 (unused private function):** A `defp` group that no
other definition, decorator, or module attribute references warns.
Acknowledge deliberate keep-alive with `@allow :unused_function`
(unknown `@allow` targets remain compile errors per GEP-0019-R007).

## Rationale

Only provable facts warn, so the lints stay trustworthy: R001 is a
certain crash, R003/R005 are certainly dead code, R004 is a certain
waste the GEP already prohibits, and R002's `_` convention is
Elixir's own. Where legitimate code can trip a lint (structural
recursion in GEP-0019-R007, kept-for-later private functions here),
the escape is an explicit annotation at the definition — greppable,
reviewable intent, never a global switch.

## Backwards Compatibility

Warnings only; no compiled output changes. Existing code may warn —
the fix is a `_` prefix, a deletion, or an `@allow`.

## Security and Determinism

None beyond GEP-0001-R024; lints are deterministic.

## Tooling and AI Usage

Agents MUST treat these warnings as defects to fix, not noise to
ignore, and SHOULD NOT add `@allow` without stating why the flagged
code is intentional.

## Rejected Alternatives

### Undefined-function (bare call) checking

Bare callee names include the whole builtin surface; enumerating it
here would couple the lint to every future builtin. Deferred.

### Configurable lint levels

One behavior everywhere keeps output portable across machines and CI;
per-site `@allow` covers the legitimate exceptions.

## Conformance

Tests MUST cover: an undefined read warning and its pyimport/`import`
suppressions; unused binding with `_` exemption and sigil-splice
reads; unreachable def and case clauses with guard and cross-arity
non-warnings; a discarded comprehension; an unused defp, its
referenced and `@allow`-acknowledged silences.

## Change History

- Revision 1, 2026-08-03: Initial version.
