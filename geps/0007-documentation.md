---
gep: 7
title: Documentation
description: Markdown @doc with localized variants, hidden docs, doctests compiled to native Python doctests, and the gan doc / gan test commands.
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
  - Tooling
created: 2026-08-01
updated: 2026-08-01
revision: 3
requires: [1]
replaces: []
superseded-by: null
resolution: null
translations:
  zh: local/zh/0007-documentation.md
---

# GEP-0007: Documentation

## Abstract

`@doc` and `@moduledoc` are Markdown, as in Elixir. Separate
`@doc_trans <locale>: "..."` attributes add localized variants without
cluttering the primary text, following the Osiris model where the default text is the runtime fallback and locales
are tooling metadata. `@doc false` hides a function from documentation.
Example sessions written with the `gan>` prompt compile into native
Python doctests inside the generated docstrings, so `gan test` (and any
standard Python doctest runner) executes them. `gan doc` prints
documentation for a module or function in a requested locale.

## Motivation

Elixir treats documentation as a first-class, testable artifact
(hexdocs' writing-documentation guide); Osiris adds locale maps so one
codebase can present docs in the reviewer's language. Gandora should
provide both without inventing runtime machinery — and its compile-to-
Python identity offers a shortcut: Python already has a doctest runner
in the standard library, so compiled examples become ordinary Python
doctests.

## Scope

The `@doc`/`@moduledoc` value forms, Markdown semantics, locale lookup,
doctest syntax and compilation, and the `gan doc` and `gan test`
commands. HTML rendering, `@spec`/`@typedoc`, `since:` metadata, and
doc coverage tooling are deferred.

## Terminology

- **Doc map**: the set of locale-tagged texts attached by one
  `@doc`/`@moduledoc`.
- **Default text**: the `default:` entry (or the whole value when a bare
  string is given).
- **Doctest line**: a line beginning with the prompt `gan> ` inside doc
  text, followed by one line of expected output.

## Specification

### Value forms and Markdown

**GEP-0007-R001:** `@doc` and `@moduledoc` accept a string (the default
text) or `false`. Doc text is Markdown; the compiler stores it verbatim
and MUST NOT reflow or reformat it.

**GEP-0007-R001A:** Localized variants attach with `@doc_trans` (after
the `@doc` they translate) and `@moduledoc_trans` (after `@moduledoc`),
carrying one or more `<locale>: "text"` pairs whose keys spell BCP 47
tags with `_` for `-` (`zh_CN` ≡ `zh-CN`). The attribute MAY be repeated
for additional locales; a duplicate locale, a `default:` key, or a
`*_trans` without its preceding doc attribute is a compile error.

**GEP-0007-R001B:** `@example "..."` (repeatable, before the `def` it
documents, usable with or without `@doc`) declares a shared,
language-neutral example block. Example blocks are appended to the
default prose in the generated docstring, have their `gan>` lines
compiled (R006), and are shown by `gan doc` in every locale.
Translations are prose-only: a `gan>` line inside `@doc_trans` /
`@moduledoc_trans` is a compile error directing the author to
`@example`, so examples are written and tested exactly once.

**GEP-0007-R002:** The default text (after doctest compilation, R006)
becomes the generated Python docstring. Localized texts are tooling
metadata read from source (or from a package's shipped sources); they
MUST NOT appear in generated code.

**GEP-0007-R003:** `@doc false` suppresses the docstring and marks the
function hidden; `gan doc` MUST say so rather than print nothing.

### Locale lookup and gan doc

**GEP-0007-R004:** `gan doc <Module>[.<function>] [--locale <tag>]`
prints the doc text for the requested locale using RFC 4647 lookup:
exact tag match (case-insensitive), then progressively shortened
prefixes, then the default text. Without `--locale` the default text is
printed.

**GEP-0007-R005:** `gan doc` resolves modules from project sources and
from installed package markers (GEP-0006-R006), reads statically, and
never imports Python.

### Doctests

**GEP-0007-R006:** Inside doc text, a doctest line `gan> <expr>`
contains one Gandora expression; the following non-prompt line is the
expected output. The compiler MUST compile the expression to Python and
emit the pair into the generated docstring as a standard Python doctest
(`>>> <compiled expr>` followed by the expected line, indentation
preserved). Non-doctest lines pass through unchanged.

**GEP-0007-R007:** Expected output is compared by Python's doctest
runner, so it is the `repr` of the result — identical to what
`inspect/1` prints (`{:ok, 1}` shows as `('ok', 1)`). Doctest
expressions MUST be single-line and MUST NOT use macros in v0; a
doctest expression that fails to compile is a compile error carrying
the function's location.

**GEP-0007-R008:** `gan test` compiles the project into the build cache
and runs every generated non-macro-only module through Python's
standard doctest runner with the project interpreter (GEP-0001-R021
selection, `-P`, cache on `PYTHONPATH`). It reports per-module results,
exits 0 when all pass and 1 otherwise. Because emitted doctests are
standard, `python -m doctest` and pytest's `--doctest-modules` MUST
work on the generated files without `gan`.

## Rationale

Compiling `gan>` examples into Python doctests keeps the no-runtime
property (the docstring is the artifact) and reuses a mature runner
instead of building one. The expected-output-is-repr rule is honest to
GEP-0001-R009's data mapping: documentation shows exactly what Python
consumers of the value will see.

Locales-as-tooling-metadata copies Osiris: one runtime fallback in the
artifact, richer language views served from source by tools, so the
generated Python stays lean and deterministic.

## Backwards Compatibility

Additive. Existing bare-string docs are the `default:` case. Text
containing `gan> ` at line start previously passed through verbatim;
it now compiles — a breaking edge only for docs that accidentally used
the prompt.

## Security and Determinism

Doctest compilation is ordinary expression compilation; nothing
executes at compile time. `gan test` executes the user's own generated
code, exactly as `gan run` does.

## Tooling and AI Usage

Agents should document public functions with Markdown `@doc`, include
`gan>` examples for non-obvious behavior, run `gan test` after edits,
and use `gan doc Mod.fun --locale <tag>` to read localized docs instead
of guessing.

## Rejected Alternatives

### Locale keywords inside @doc (`@doc default: ..., zh_CN: ...`)

The first design. It buries the primary text behind a `default:` label
and turns every translated function head into one long attribute;
separate `@doc_trans` lines keep the common case (English only)
zero-ceremony and translations independently editable.

### A Gandora-side doctest runner

Re-implements what Python ships, needs a runner in every environment,
and diverges from what pytest users already run in CI.

### Embedding all locales in the docstring

Bloats every artifact with text most consumers cannot read and makes
output depend on translation edits; tooling reads locales from source
instead.

### iex>-compatible prompt

Reusing `iex>` would suggest Elixir semantics (inspect formatting,
charlists) that Gandora deliberately does not have; `gan>` marks the
boundary.

## Open Questions

None for this revision.

## Conformance

Tests MUST cover: bare-string, keyword-form, and `false` values;
locale lookup incl. prefix fallback and default; the R003 missing-
default diagnostic; doctest compilation of expressions (pipes, interop
calls) with preserved indentation; a failing doctest detected by
`gan test`; and `python -m doctest` running a generated module
directly.

## Change History

- Revision 3, 2026-08-01: Added shared @example blocks (R001B);
  translations are prose-only so examples cannot rot untested.
- Revision 2, 2026-08-01: Replaced the locale-keyword form of @doc with
  separate @doc_trans / @moduledoc_trans attributes (R001A).
- Revision 1, 2026-08-01: Initial version.
