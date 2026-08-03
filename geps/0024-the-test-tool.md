---
gep: 24
title: The Test Tool
description: tests/*.gan with std Test assertions, compiled with the project and executed by pytest — one `gan test` runs doctests and test files alike.
author: MJ
status: Accepted
type: Standards Track
areas:
  - Tooling
created: 2026-08-03
updated: 2026-08-03
revision: 1
requires: [7, 10, 13]
replaces: []
superseded-by: null
resolution: null
translations:
  zh: local/zh/0024-the-test-tool.md
---

# GEP-0024: The Test Tool

## Abstract

`gan test` is the one testing command: it runs every `@example`
doctest (GEP-0007) and then every `test_*` function of `tests/*.gan`,
compiled together with the project's sources and executed by pytest
under the project interpreter. Assertions are ordinary std functions
(`Test.assert_eq` and friends) that raise on failure — pytest turns
the raise into a report. No test runner is invented; no runtime is
introduced.

## Motivation

Doctests document happy paths; real suites need edge cases, negative
cases, and invariants. The language had the pieces — modules compile
to plain Python functions, pytest discovers `test_*` — but no blessed
convention. Making the convention official gives every project (std
included) the same one-command story and lets agents write tests the
way they write code.

## Scope

Project-level testing. Coverage, fixtures/parametrization, and test
selection flags stay with pytest's own surface (the compiled tree is
ordinary pytest input); property-based testing is future work.

## Specification

**GEP-0024-R001:** `gan test` (and `ganc test`) runs, in order: every
`@example` doctest of the compiled project; then, when a `tests/`
directory exists, every `tests/*.gan` file — compiled together with
the project sources (full module resolution: project modules, std,
installed packages) into `.gandora/tests/`, then
`python -m pytest .gandora/tests -q` with the doctest cache on
`PYTHONPATH`. The exit code is non-zero when either phase fails. A
missing pytest is reported with the `uv add --dev pytest` remedy.

**GEP-0024-R002:** Test modules are ordinary Gandora modules whose
`test_*` public functions are the tests (pytest discovery applies).
`Test` (gandora-std) provides the assertion family: `assert_eq`,
`assert_true`, `assert_false`, `assert_nil`, `assert_raises`,
`assert_contains` — each raises with a message naming expectation and
reality. Test files follow every language rule (lints, fmt, specs on
public helpers where meaningful).

**GEP-0024-R003:** `tests/` never ships: it is outside the source
roots, so `gan build` and packaging ignore it; `.gandora/tests` is
build output.

## Rationale

Compiling tests *with* the project sources gives them the same module
resolution as the code under test — no consumer-project scaffolding,
no separate venv. pytest as the executor keeps failure UX, filtering
(`-k`), and CI integrations for free while the tests themselves stay
pure Gandora.

## Backwards Compatibility

Additive; projects without `tests/` see doctests only, as before.

## Security and Determinism

Tests execute project code under the project interpreter — the
existing `gan run`/`gan test` trust boundary.

## Tooling and AI Usage

Agents SHOULD add a `tests/*.gan` file alongside any nontrivial
change, assert edge cases doctests don't cover, and treat a red
`gan test` as a stop signal. The sandbox teaches `Test` like any std
module (`gan lsc doc Test.assert_eq`).

## Rejected Alternatives

### A bespoke test runner

pytest's discovery, reporting, and ecosystem for one dependency in
the dev group; a bespoke runner would re-implement all three.

### ExUnit-style `test "name" do` macros

A macro DSL adds surface before it adds value; named functions are
already discoverable, greppable, and lintable. Revisit if demand
appears.

## Conformance

Tests MUST cover: a passing and a failing `tests/*.gan` run through
`gan test` exit codes; module resolution of project + std modules
inside tests; the missing-pytest message; and each `Test` assertion's
success and failure paths.

## Change History

- Revision 1, 2026-08-03: Initial version.
