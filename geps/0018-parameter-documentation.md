---
gep: 18
title: Parameter Documentation
description: The @param attribute — validated, translatable per-parameter prose that generates the Elixir "## Parameters" section and feeds signature help.
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
  - Tooling
created: 2026-08-02
updated: 2026-08-02
revision: 1
requires: [7, 15, 17]
replaces: []
superseded-by: null
resolution: null
translations:
  zh: local/zh/0018-parameter-documentation.md
---

# GEP-0018: Parameter Documentation

## Abstract

`@param` documents one parameter of the next definition:

```elixir
@param name, "String that represents the name of the person."
@param locale, "BCP 47 tag used for casing rules."
@spec split(string(), string()) :: list(string())
def split(name, locale \\ "en") do
```

The names are validated against the clause heads at compile time —
parameter docs cannot rot. Renderers generate the Elixir-style
`## Parameters` section from them, translations travel through
`@param_trans`, and the language server serves the active parameter's
own description in signature help. A hand-written `## Parameters`
section inside `@doc` remains valid Elixir and is never parsed; the
attribute is the structured path, the convention is the compatible
one.

## Motivation

Parameter prose is the one slice of documentation that both humans
and tools need *positionally*: hover wants a section, signature help
wants exactly the description of the argument under the cursor, and
`gan lsc` consumers want it as data. Prose conventions cannot be
validated or addressed; a dedicated channel can — the same judgment
that gave `@example` its own attribute (GEP-0007).

## Scope

The `@param` and `@param_trans` attributes, their validation,
rendering, and tooling surface, plus named `@spec` arguments. Doc
comments for struct fields and module attributes are out of scope.

## Specification

**GEP-0018-R001:** `@param name, "text"` precedes a definition (in
any order relative to `@doc`, `@spec`, and `@example`) and documents
the parameter `name` of the following definition group. `@param` MAY
repeat, once per parameter, in any subset and order; text is Markdown
prose. Repeating a name or placing `@param` where no definition
follows is a compile error.

**GEP-0018-R002:** Every `@param` name MUST occur as a variable in at
least one clause head of the following definition (defaults
included, `_`-prefixed names excluded); otherwise the compile error
names the parameter and the definition. Parameters without `@param`
are simply undocumented — coverage is not required.

**GEP-0018-R003:** `@param_trans name, locale: "text"` adds
translations for a previously declared `@param name`, with the
GEP-0007 locale rules (BCP 47 keys, prose only). Declaring a
translation for an undeclared parameter is a compile error.

**GEP-0018-R004:** Renderers generate the section from the
attributes: `gan doc` and hover append `## Parameters` (or its
locale's heading for translated output) listing `- name: text` in
clause-head order; the generated Python docstring carries the
default-locale section after the prose and before examples. A
definition with `@param` attributes SHOULD NOT also hand-write a
`## Parameters` section; renderers do not deduplicate.

**GEP-0018-R005:** The documentation surface carries the data
structurally: `gandora_core.doc` returns
`params: [{name, entries: {locale: text}}]` in clause-head order,
`gan lsc doc` passes it through, and signature help
(GEP-0015-R010) attaches each parameter's default-locale description
to its `ParameterInformation`.

**GEP-0018-R006:** `@spec` heads MAY name their arguments in Elixir's
form — `@spec split(name :: string(), locale :: string()) ::
list(string())`. Names are informational (shown in rendered specs);
the types compile per GEP-0017 unchanged. A named spec argument whose
name conflicts with the clause head position's variable is not an
error — the clause head wins for tooling labels.

## Rationale

Validated names are the feature: a renamed parameter breaks the build
instead of silently orphaning its documentation. Generating the
Elixir section from the attribute keeps one source of truth while
matching the ecosystem's rendered convention exactly. Parsing the
hand-written section was rejected for the same reason it was for
doctests: prose extraction breaks on localization and formatting
variance.

## Backwards Compatibility

Additive. Hand-written `## Parameters` sections keep working
untouched.

## Security and Determinism

Attributes are compile-time data; rendering is deterministic.

## Tooling and AI Usage

Agents SHOULD emit `@param` for every non-obvious parameter of public
functions and read parameter docs from `gan lsc doc` rather than
parsing Markdown.

## Rejected Alternatives

### Parsing the `## Parameters` convention

Unvalidatable and fragile across locales and list styles — the
GEP-0007 anti-parsing decision applies verbatim.

### Parameter docs inside `@spec`

Overloads the type channel with prose and breaks the GEP-0017 rule
that every spec element is a type expression.

## Conformance

Tests MUST cover: rendering into docstring, `gan doc`, and hover;
clause-head-order listing; the unknown-name, duplicate-name, and
orphan-translation errors; `@param_trans` locale lookup; signature
help carrying the active parameter's text; and named `@spec`
arguments compiling identically to unnamed ones.

## Change History

- Revision 1, 2026-08-02: Initial version.
