---
gep: 9
title: Embedded Languages and Templates
description: The ~<lang> sigil family generalized beyond ~python, with EEx-style <%= %> splicing for values and code.
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
  - Interop
created: 2026-08-01
updated: 2026-08-02
revision: 3
requires: [5]
replaces: []
superseded-by: null
resolution: null
translations:
  zh: local/zh/0009-embedded-languages-and-templates.md
---

# GEP-0009: Embedded Languages and Templates

## Abstract

`~python` is one instance of a paradigm, not a special case: any
`~<lang>` sigil (`~markdown`, `~sql`, `~html`, ...) embeds a raw body
tagged with its language. Every embedded body supports EEx-style
splicing: `<%= expr %>` inserts a Gandora expression. For `~python`
the splice is code (compiled expression text, as before, now
parameterizable); for every other language the sigil evaluates to a
string with spliced runtime values. One mechanism covers the embedded-language family and EEx's template
marker.

## Motivation

Raw embedded bodies (GEP-0005) deliberately suppress `#{}`
interpolation because `#` and `{` mean things in most target
languages. But real embedded content needs parameters — a SQL query's
table name, a Python expression over a Gandora binding with a computed
piece, a Markdown report with values. EEx's `<%= %>` marker is
collision-free in practice, familiar, and works identically across
languages.

## Scope

The embedded sigil family, `<%= %>` splicing, and language tagging.
EEx statement/loop tags (`<% %>`), `.eex` template files, layouts, and
per-language validation are deferred.

## Terminology

- **Embedded sigil**: a sigil whose name is not one of the GEP-0005
  built-ins (`w`, `s`, `r`); its name is the language tag.
- **Splice**: the `<%= expr %>` marker holding one Gandora expression.

## Specification

**GEP-0009-R001:** A sigil with any name other than the built-ins is
an embedded sigil. Its body is raw (GEP-0005-R002 backslash rule; no
`#{}` interpolation). The unknown-sigil diagnostic of GEP-0005-R009 is
repealed. The language tag is compile-time metadata for tooling
(highlighting, future validation); the compiler does not interpret the
body language.

**GEP-0009-R002:** Inside every embedded body, `<%= expr %>` holds one
Gandora expression, parsed and compiled with the bindings in scope at
the sigil. Whitespace inside the marker is ignored; `<%%=` escapes a
literal `<%=`. Splices MUST be single expressions; a failing parse is
a compile error naming the sigil.

**GEP-0009-R003:** `~python` keeps its GEP-0005-R007 semantics — the
body is spliced verbatim into the generated code as one expression —
with each `<%= expr %>` replaced by the compiled expression text
(parenthesized). It remains the only sigil whose body enters the
program as code.

**GEP-0009-R004:** Every other embedded sigil evaluates to a string:
the body text with each splice replaced by the runtime value of its
expression, formatted as by `#{}` interpolation (compiling to an
f-string when splices are present, a plain literal otherwise).

**GEP-0009-R006:** `~prompt` is the blessed spelling of R004 for
prose destined for an AI model: `~prompt(...)` for one-liners,
`~prompt"""..."""` for blocks. The body is raw — quotes, braces,
backslashes, and JSON need no escaping — while `<%= expr %>` still
splices values. Tooling (docs cards, the manual) teaches this name;
any other R004 sigil name behaves identically.

**GEP-0009-R005:** Multi-line embedded bodies use the GEP-0005
delimiters including `"""`; splicing works identically. A `"""` body
follows the heredoc dedent semantics of GEP-0001-R026 — the opening
newline is dropped and the closing delimiter's indentation is stripped
from every line — so templates indent naturally inside modules while
producing flush-left values. Outside that lexical trim, generated
output MUST preserve body text byte-for-byte outside splices.

## Rationale

One marker for all languages beats per-language interpolation rules,
and `<%= %>` is the established Elixir spelling (EEx). Code-splice for
`~python` versus value-splice for everything else matches what each
body *is*: `~python` bodies execute, other bodies are data. Language
tags without validation keep the compiler small while giving tooling a stable hook.

## Backwards Compatibility

Repeals the unknown-sigil error (additive: previously-rejected
programs become valid). `~python` bodies containing the literal text
`<%=` must switch to `<%%=`; no other existing surface changes.

## Security and Determinism

Splices are ordinary compiled expressions; embedded bodies remain
inert text at compile time. `~python` retains exactly the audit
surface it had: author-written code visible in the output.

## Tooling and AI Usage

Agents should use `~sql`/`~markdown`/etc. for embedded content instead
of concatenating strings, parameterize with `<%= %>`, and never build
`~python` code from untrusted strings — splices interpolate *values*
into other languages but *code* into Python.

## Rejected Alternatives

### Enabling #{} in embedded bodies

Collides with target-language syntax (`#` comments, `{}` blocks,
Python dict/set displays) — the reason raw bodies exist.

### A full EEx engine now

Statement tags, layouts, and file templates are a library-sized
surface; the value/code splice covers the language-level need, and a
future GEP can add `EEx`-class tooling on top without changing this
contract.

## Open Questions

None for this revision.

## Conformance

Tests MUST cover: an arbitrary-language sigil evaluating to its exact
body; value splices (including expressions with pipes and interop
calls) compiling to f-strings; `~python` code splices; the `<%%=`
escape; multi-line `"""` bodies; the single-expression diagnostic; and
byte preservation outside splices.

## Change History

- Revision 3, 2026-08-04: R006 — `~prompt` blessed as the prose
  sigil for AI prompts (raw body, no escaping, `<%= %>` splices).

- Revision 2, 2026-08-02: R005 — `"""` sigil bodies follow the
  GEP-0001-R026 heredoc dedent.

- Revision 1, 2026-08-01: Initial version.
