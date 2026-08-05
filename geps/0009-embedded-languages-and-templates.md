---
gep: 9
title: Embedded Languages and Templates
description: Two embedded tiers by symbol — ~text templates and the $python code splice — with EEx-style <%= %> value splicing for text; data is ordinary Gandora maps.
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
  - Interop
created: 2026-08-01
updated: 2026-08-02
revision: 7
requires: [5]
replaces: []
superseded-by: null
resolution: null
translations:
  zh: local/zh/0009-embedded-languages-and-templates.md
---

# GEP-0009: Embedded Languages and Templates

## Abstract

Embedded content splits into two tiers, each with its own symbol
matching what the body *is*: `~<lang>` sigils (`~markdown`, `~sql`,
`~prompt`, ...) are **text** — raw bodies tagged with a language,
evaluating to strings with `<%= expr %>` value splices; `$python(...)`
is **code** — one Python expression entering the program verbatim,
with code splices. Data needs no embedding: Gandora maps are the data
literal. One splice mechanism covers the family; the symbol tells the
reader (and the AI) the semantics at a glance.

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

**GEP-0009-R003:** the code splice is spelled **`$python(expr)`** —
`$` is the Python world (GEP-0003), and this form is a whole Python
expression rather than a module member. The body is spliced verbatim
into the generated code as one expression, with each `<%= expr %>`
replaced by the compiled expression text (parenthesized). It is the
only embedded body that enters the program as code. `~python(...)`
remains valid as an ordinary R004 *text* sigil (a body tagged
python); the Advisor flags it with the `$python` recipe in case the
author meant code.

**GEP-0009-R004:** Every other embedded sigil evaluates to a string:
the body text with each splice replaced by the runtime value of its
expression, formatted as by `#{}` interpolation (compiling to an
f-string when splices are present, a plain literal otherwise).

**GEP-0009-R006:** `~p` is the blessed spelling of R004 for prose
destined for an AI model — a member of the GEP-0005-R010 functional
whitelist alongside `~w ~s ~r`: `~p(...)` for one-liners,
`~p"""..."""` for blocks. The body is raw — quotes, braces,
backslashes, and JSON need no escaping — while `<%= expr %>` still
splices values. Tooling teaches this name; longer tags such as
`~prompt` remain ordinary R004 text sigils with identical semantics.

**GEP-0009-R007 (repealed in revision 6):** the `%json` data literal
was withdrawn the day it shipped: Gandora maps (`%{}`) are the one
data spelling — richer (atom keys, arbitrary expressions) and already
native to the language and its AI writers. A pasted JSON document
becomes a map by swapping `:` for `=>`; the Advisor teaches that
rewrite on sight, and runtime JSON text is `$json.loads(s)`. One way
to write data beats a second, weaker way.

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

- Revision 7, 2026-08-05: R006 — the prompt sigil is `~p`, joining the
  short functional whitelist (GEP-0005-R010); `~prompt` stays an
  ordinary text tag.

- Revision 6, 2026-08-04: R007 repealed — `%json` withdrawn; maps are
  the one data spelling, the Advisor teaches the JSON→map rewrite.

- Revision 5, 2026-08-04: the tier/symbol split — `~` is uniformly
  text (any name, including python/json, is just a language tag),
  `$python(...)` is the code splice (was `~python`), `%json` is the
  data literal (was `~json`, never released); the Advisor carries the
  migration recipes; unknown `%name` data literals error.

- Revision 4, 2026-08-04: R007 — `~json` compile-time data literal
  (JSONC-tolerant, splice-free, zero runtime).

- Revision 3, 2026-08-04: R006 — `~prompt` blessed as the prose
  sigil for AI prompts (raw body, no escaping, `<%= %>` splices).

- Revision 2, 2026-08-02: R005 — `"""` sigil bodies follow the
  GEP-0001-R026 heredoc dedent.

- Revision 1, 2026-08-01: Initial version.
