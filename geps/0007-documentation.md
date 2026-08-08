---
gep: 7
title: Documentation
description: The Gandora documentation model — Elixir-style @doc (text, metadata, hidden), a dedicated @example channel for doctests, and prose-only translations.
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
  - Tooling
created: 2026-08-01
updated: 2026-08-08
revision: 7
requires: [1]
replaces: []
superseded-by: null
resolution: null
translations:
  zh: local/zh/0007-documentation.md
---

# GEP-0007: Documentation

## Abstract

Documentation has three channels, each with one attribute family and no
overlap. `@doc`/`@moduledoc` follow Elixir: a Markdown string is the
text, a keyword list is metadata (`since:`, `deprecated:`, ...),
`false` hides, and repeated lines accumulate — the compiler stores text
verbatim and never parses inside it. `@example` is the dedicated
channel for examples: repeatable blocks whose `gan>` lines are compiled
into native Python doctests, appended to the generated docstring, run
by `gan test`, and rendered in every locale. `@doc_trans`/
`@moduledoc_trans` add prose-only translations. One copy of every
example; no text extraction heuristics anywhere.

## Motivation

Earlier revisions tried to keep examples inside the doc text
(Elixir-style) while also localizing that text, which forced the
compiler to parse prose — first with locale keywords, then heading
conventions, then structural extraction and an inclusion directive.
Review rejected parsing prose as a paradigm: documentation text should
be opaque data. Giving examples their own attribute removes every
heuristic: the doc text is never inspected, translations are plain
prose, and the example channel is the only thing the compiler
understands.

## Scope

The three channels, docstring assembly, doctest compilation, and the
`gan doc` / `gan test` commands. HTML rendering, `@spec`/`@typedoc`,
and doc coverage tooling are deferred.

## Terminology

- **Doc unit**: the documentation of one module or function: default
  text, translations, metadata, hidden flag, example blocks.
- **Doctest line**: a line whose content begins with `gan> `, holding
  one Gandora expression; the following non-blank line is its expected
  output.
- **Example block**: the string given to one `@example` attribute.

## Specification

### @doc — text, metadata, hidden (Elixir-style)

**GEP-0007-R001:** `@doc` and `@moduledoc` accept, per attribute line:
a Markdown string (the text), a keyword list (metadata), or `false`
(hidden). Multiple lines before one definition accumulate into one doc
unit, in any order. A second text string, or a repeated metadata key,
is a compile error. Text is opaque: the compiler stores it verbatim
and MUST NOT parse, reflow, or transform its content.

**GEP-0007-R002:** Metadata values MUST be literals (string, boolean,
integer, float, atom). The keys `since` and `deprecated` are
well-known and additionally surface in the generated docstring as
trailing `Since: <v>` / `Deprecated: <v>` lines; all metadata is
otherwise tooling-facing only.

**GEP-0007-R003:** `@doc false` suppresses the docstring and hides the
definition; `gan doc` MUST say the target is hidden rather than print
nothing.

### @example — the one channel the compiler understands

**GEP-0007-R004:** `@example "..."` attaches one example block to the
next definition; the attribute MAY be repeated and MAY be used with or
without `@doc`. Blocks keep source order. Inside a block, doctest
lines are compiled (R008); all other lines pass through verbatim, so a
block may carry its own headings and captions.

**GEP-0007-R005:** The generated docstring is assembled as: the doc
text verbatim, then each compiled example block, then the well-known
metadata trailer (R002), the parts separated by one blank line. A
doctest line inside `@doc`/`@moduledoc` text is NOT compiled or tested
— examples belong in `@example`, and `gan check` MUST warn when doc
text contains a `gan> ` line.

### Translations — prose only

**GEP-0007-R006:** `@doc_trans` and `@moduledoc_trans` carry one or
more `<locale>: "markdown"` pairs; locale keys spell BCP 47 tags with
`_` for `-` (`zh_CN` ≡ `zh-CN`). The attribute MAY be repeated; a
duplicate locale is a compile error. Translations are tooling metadata
read from source (or a package's shipped sources) and MUST NOT appear
in generated code.

**GEP-0007-R007:** A doctest line inside a translation is a compile
error directing the author to `@example`. Translations therefore can
never carry a second, untested copy of an example.

### Rendering and testing

**GEP-0007-R008:** Doctest compilation: the expression after `gan> `
is compiled to Python and emitted as `>>> <compiled>` with indentation
preserved; its expected line passes through. Expected output is
compared by Python's doctest runner and is therefore the `repr` of the
result — what `inspect/1` prints. Doctest expressions MUST be
single-line and MUST NOT use macros in v0; a failing compile is a
compile error carrying the definition's location.

**GEP-0007-R009:** `gan doc <Module>[.<function>] [--locale <tag>]`
renders: metadata first (`[deprecated]` prominently), then the text
for the locale selected by RFC 4647 lookup (exact, shortened prefixes,
then default), then every example block with its `gan>` prompts as
authored. It resolves modules from project sources and installed
package markers (GEP-0006-R006), reads statically, and never imports
Python.

**GEP-0007-R010:** `gan test` compiles the project into the build
cache and runs every generated non-macro-only module through Python's
standard doctest runner, importing by dotted module name with the
GEP-0001-R021 interpreter, `-P`, and the cache on `PYTHONPATH`. It
reports per-module results and exits 0 only when all pass. Emitted
doctests are standard: `python -m doctest` and pytest's
`--doctest-modules` MUST work on generated files without `gan`.

**GEP-0007-R011:** The default documentation channels — `@doc`,
`@moduledoc`, `@param` — are written in English; text containing CJK
code points there is a compile error, caught in the check phase and
teaching the `_trans` spelling. Localized prose has its own channels
(`@doc_trans`, `@moduledoc_trans`, `@param_trans`, R001A/GEP-0018-R003)
and stays untouched, as do `@example` bodies and comments. The reason
is retrieval (GEP-0031): the lightweight `@doc` sentence is the
semantic key the code map ranks by — a corpus that mixes languages in
one channel ranks in neither, while one language per channel lets an
agent search by meaning and lets every locale keep its own full-weight
prose beside it.

## Rationale

Three channels with disjoint jobs make every earlier failure mode
structurally impossible: text is opaque (no parsing paradigm),
examples exist exactly once in a channel built for them, and
translations are plain data. The cost is one departure from Elixir —
examples move from the doc text into `@example` — which review judged
cheaper than parsing prose; the R005 warning catches Elixir-habit
`gan>` lines left in doc text.

Compiling `gan>` into native Python doctests keeps the no-runtime
property and reuses a mature runner; expected-output-as-repr is honest
to GEP-0001-R009's data mapping.

## Backwards Compatibility

Revision 5 supersedes revisions 1–4: `@doc_meta`/`@moduledoc_meta` and
the `<!-- examples -->` directive are removed; doctests in doc text are
no longer compiled (warned instead); `@example` is the sole doctest
carrier. `@doc` string/keyword/false forms and `@doc_trans` are
unchanged.

## Security and Determinism

Doctest compilation is ordinary expression compilation; nothing
executes at compile time. `gan test` executes the user's own generated
code, exactly as `gan run` does.

## Tooling and AI Usage

Agents should write Markdown `@doc` prose, put every runnable example
in `@example`, attach metadata with `@doc since:`/`deprecated:`, keep
translations prose-only, and run `gan test` after edits.

## Rejected Alternatives

### Doctests inline in doc text (revisions 1–4)

Elixir's own layout, but with localized texts it forces either
duplicated examples in translations (rot) or compiler parsing of prose
(extraction heuristics, inclusion directives). Review rejected prose
parsing as a paradigm; the dedicated channel needs none of it.

### Locale keywords inside @doc

Buried the primary text behind a `default:` label and made every
translated head one long attribute.

### Embedding all locales in the docstring

Bloats every artifact with text most consumers cannot read; tooling
reads locales from source instead.

## Open Questions

None for this revision.

## Conformance

Tests MUST cover: accumulation of string/keyword/false `@doc` lines in
either order with duplicate-text and duplicate-key diagnostics; the
metadata trailer; docstring assembly order (text, examples, trailer);
`@example` with and without `@doc`; the R005 warning for `gan>` in doc
text; the R007 translation diagnostic; doctest compilation with
preserved indentation; locale fallback and example rendering in
`gan doc --locale`; a failing doctest detected by `gan test`; and a
generated module run by `python -m doctest` directly.

## Change History

- Revision 7, 2026-08-08: R011 — the default doc channels are English,
  enforced as a check-phase error; `_trans` channels, `@example`
  bodies, and comments are untouched. The default `@doc` sentence is
  the semantic key GEP-0031's code map ranks by.

- Revision 6, 2026-08-02: R009 — built-ins carry embedded bilingual
  docs in gan doc.

- Revision 5, 2026-08-01: Final model — opaque doc text, dedicated
  @example channel, prose-only translations; removed the extraction
  and directive machinery of revision 4.
- Revision 4, 2026-08-01: Structural example extraction with inclusion
  directives (superseded).
- Revision 3, 2026-08-01: Shared @example blocks (restored in rev 5).
- Revision 2, 2026-08-01: Split translations into @doc_trans.
- Revision 1, 2026-08-01: Initial version.
