---
gep: 25
title: The Check
description: One verdict — compiler diagnostics plus Advisor suggestions — printed by `gan check`, returned as JSON by `gan lsc check`, and gating `gan build`.
author: MJ
status: Accepted
type: Standards Track
areas:
  - Tooling
created: 2026-08-04
updated: 2026-08-04
revision: 1
requires: [12, 13, 22]
replaces: [23]
superseded-by: null
resolution: null
translations:
  zh: local/zh/0025-the-check.md
---

# GEP-0025: The Check

## Abstract

`gan check` is the compiler's whole verdict for a project: every
diagnostic (errors and GEP-0022 lints) **and** every Advisor
suggestion — practice gaps, cross-language migration hints, and
edit-distance did-you-means against real symbols. `gan lsc check`
returns the same verdict as one JSON object
`{diagnostics, suggestions}`. **`gan build` runs check first**: errors
stop the build before any artifact is written; warnings and
suggestions print and let it proceed — the heavy-compiler contract,
the way Rust behaves. The GEP-0023 sandbox (`gan try`) is retired: its
teaching engine lives here, at project scope, under the commands that
already exist.

## Motivation

A verdict per snippet (`try`) duplicated what `check` should have
been. One spelling — check — serves the human at the terminal, the
agent over JSON, the editor via the same diagnostics, and the build
gate, with nothing extra to learn.

## Scope

The check verdict and the build gate. Execution stays with
`gan run`/`gan test`/`gan repl`.

## Specification

**GEP-0025-R001 (the verdict):** `gan check` prints, per file:
compiler diagnostics (spanned; errors and warnings) followed by
Advisor suggestions, each tagged `practice` | `migration` |
`did_you_mean`. Exit code 1 when any error exists; warnings and
suggestions never fail the check. `gan lsc check` emits the identical
verdict as `{"diagnostics": [...], "suggestions": [...]}` with paths
on every entry, and the same exit-code contract.

**GEP-0025-R002 (the Advisor):** The suggestion engine is
`Advisor` in gandora-tool — one implementation for the runner and
`lsc`. It masks string/heredoc/sigil/comment content before any
textual check (prose never trips a pattern), deduplicates by message,
and consults real candidates: module symbols for member typos (with a
gandora-std introspection fallback so venv-less projects still get
`Enum.mpa → Enum.map`), the file's own identifiers for undefined
variables, and the keyword list for construct typos.

**GEP-0025-R003 (the build gate):** `gan build` runs the R001 verdict
first. Any error aborts with `build aborted: check failed` and exit
1; otherwise the build proceeds with warnings and suggestions already
printed. `ganc build` (plumbing) remains ungated.

## Rationale

Folding the sandbox into check follows the tool's own lesson: an
agent's loop is write → verdict → fix → build, and the verdict
belongs to the command every developer already runs. Moving Advisor
into gandora-tool puts the teaching engine below every consumer
(runner, lsc, future editor surfaces) without a new package.

## Backwards Compatibility

**Breaking**: `gan try` and `gan lsc try`/`review` are removed;
`gan lsc check` changes shape from a list to
`{diagnostics, suggestions}`. GEP-0023 is superseded by this GEP.

## Security and Determinism

Check executes nothing; the build gate only reorders existing steps.

## Tooling and AI Usage

The agent loop: write → `gan check` (or `lsc check` for JSON) → fix
every diagnostic, apply every suggestion → `gan test` → `gan build`.
Concept lookups stay with `gan lsc doc <construct>`.

## Rejected Alternatives

### Keeping `try` alongside check

Two names for one verdict; the snippet-execution half was `gan run`
with extra steps.

### Gating `ganc build` too

Plumbing stays predictable for scripts; porcelain (`gan build`)
carries the policy.

## Conformance

A BDD suite over `gan lsc check` MUST cover: the clean-module silence
guarantee; error/warning exit-code contract; every suggestion kind;
lint pass-through with taught corrections; the hostile-input gauntlet
(never a crash, always JSON); and the build gate's abort-on-error.

## Change History

- Revision 1, 2026-08-04: Initial version — supersedes GEP-0023.
