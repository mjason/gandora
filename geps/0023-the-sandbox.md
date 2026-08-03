---
gep: 23
title: The Sandbox
description: One query that validates generated code — compile, lint, fuzzy-suggest, execute with a timeout — so agents learn the language by trying it.
author: MJ
status: Accepted
type: Standards Track
areas:
  - Tooling
created: 2026-08-03
updated: 2026-08-03
revision: 1
requires: [12, 15, 22]
replaces: []
superseded-by: null
resolution: null
translations:
  zh: local/zh/0023-the-sandbox.md
---

# GEP-0023: The Sandbox

## Abstract

`gan lsc try <file|-> [--no-run]` answers, as one JSON value, the
question an AI agent asks after generating Gandora code: *is this
right, and if not, what did I probably mean?* The pipeline compiles,
lints, spell-checks module members against real symbols, flags common
cross-language habits, then executes in a temp directory under a hard
timeout — capturing stdout and, for bare-statement snippets, the value
of the last expression. Wrong names get Rails-style *did-you-mean*
suggestions from edit-distance search. Nothing touches the project.

## Motivation

A new language's biggest adoption risk for AI agents is the feedback
loop: a plausible-but-wrong spelling (`Enum.mpa`, `return`, Python's
`def f():`) costs a full edit-build-run round trip to discover, and a
runtime `AttributeError` names the symptom, not the fix. One fast
query that verdicts *and teaches* turns every mistake into a lesson.

## Scope

A read-only validation query in `gan lsc`. Persistent sandboxes,
resource quotas beyond a wall-clock timeout, and network isolation are
out of scope: the sandbox runs the developer's own code with the
project's own interpreter, exactly as `gan run` would.

## Specification

**GEP-0023-R001:** `gan lsc try <file|->` (stdin with `-`) accepts a
full module or bare statements and emits one JSON object:
`ok` (boolean), `stage` (`compile` | `lint` | `run` | `ok`),
`diagnostics` (compiler errors and GEP-0022 lints, spanned),
`suggestions` (see R002/R003), `python` (the generated code — the
zero-runtime promise made inspectable), `stdout`, and `value` (the
`repr` of a snippet's last expression). `--no-run` stops after
diagnostics. Execution happens in a fresh temp directory with the
project's interpreter and a hard timeout; a module's `main/0` runs
exactly once.

**GEP-0023-R002 (did-you-mean):** Misspellings get edit-distance
suggestions from *real* candidates: `Mod.fun(...)` calls are checked
against the module's actual symbols (`Enum.mpa` → `Enum.map`),
undefined-variable lints against the snippet's own identifiers
(`valeu` → `value`), and compile-error sites against the keyword list
(`defmodul` → `defmodule`). Suggestions carry
`"kind": "did_you_mean"`.

**GEP-0023-R003 (migration hints):** Common cross-language habits are
recognized textually and answered with the Gandora spelling —
`return`, `while`, `elif`, Python `def ...():`, `lambda`,
`None`/`True`/`False`, `import`/`from ... import`, `self.`,
`&&`/`||`, augmented assignment, f-strings, `switch`, `== nil`, and
the retired `$"a.b"`. Suggestions carry `"kind": "migration"`; they
are advisory and never block a verdict.

**GEP-0023-R004 (practice hints):** The documented standards surface
as `"kind": "practice"` suggestions where the compiler stays silent:
public `def`s without `@spec`, and a `$module` referenced repeatedly
where a `pyimport` is the idiom (GEP-0003 rev 6).

## Rationale

Building the sandbox into `lsc` keeps one AI surface: the agent that
checks, reads docs, and finds references with `lsc` learns and
validates with it too. Textual mistake patterns are deliberately
simple — they exist to teach idioms, not to parse Python — and
edit-distance search over real symbol tables is what makes
suggestions trustworthy rather than hallucinated.

## Backwards Compatibility

Additive.

## Security and Determinism

The sandbox executes user-supplied code with the project interpreter —
the same trust boundary as `gan run`/`gan test`. The timeout bounds
wall clock, not capability; verdict output is deterministic apart from
the executed program's own behavior.

## Tooling and AI Usage

Agents SHOULD route generated code through `gan lsc try` before
writing it into a project, treat `did_you_mean` suggestions as the
correction to apply, and use `--no-run` when execution is not needed.
The loop: generate → `try` → apply suggestions → `try` → write.

## Rejected Alternatives

### A long-lived sandbox server

State accumulates and diverges from the project; a fresh temp dir per
query keeps verdicts reproducible and the implementation ~300 lines.

### Compiler-side did-you-mean

The compiler stays small and certain; fuzzy search belongs in tooling
where candidates (symbols, identifiers) are already indexed.

## Conformance

Tests MUST cover: a clean snippet run capturing stdout and value; a
member typo suggesting the real function; migration hints for Python
habits; undefined-variable and keyword did-you-mean; a practice hint;
single execution of `main/0`; and `--no-run`.

## Change History

- Revision 1, 2026-08-03: Initial version.
