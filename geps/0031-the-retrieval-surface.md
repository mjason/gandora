---
gep: 31
title: The Retrieval Surface
description: An AI-native code-perception layer — the code map, filename search, content search, and ranged reads over the project, its documentation, and the .gan sources of every installed package — served by gan-lsc and forwarded by gan-mcp.
author: MJ
status: Accepted
type: Standards Track
areas:
  - Tooling
  - AI Integration
created: 2026-08-08
updated: 2026-08-08
revision: 1
requires: [6, 15, 26, 28]
replaces: []
superseded-by: null
resolution: null
translations:
  zh: local/zh/0031-the-retrieval-surface.md
---

# GEP-0031: The Retrieval Surface

## Abstract

Before an agent can edit code it has to find code. This GEP gives
Gandora an AI-native perception layer built from what the toolchain
already knows: a **code map** (`gan lsc map`) that lays out the
project, the standard library, and every installed package with source
paths and rendered heads; **filename search** (`find`) and **content
search** (`grep`, regex or BM25-ranked) over a corpus that includes
project sources, project documentation, and the `.gan` sources every
package ships (GEP-0006); and **ranged reads** (`read`) that fetch one
block — a function by name, or a path with a line range — instead of a
whole file. Each capability is one `gan lsc` query returning one JSON
value, and `gan-mcp` forwards all four as tools, so shells and MCP
clients see the same answers.

## Motivation

Generic retrieval stacks reach for tree-sitter and an indexer; Gandora
does not need either. The compiler is the parser (`symbols`,
`wsymbols`, `definition` already return names, heads, lines, and
paths), and GEP-0006 already made every published package carry its
`.gan` sources for macro consumers — which means the standard library
and third-party dependencies are *source-searchable by construction*,
bilingual `@doc` prose included. What was missing is the surface: one
map an agent reads to know where everything lives, and three verbs —
find, grep, read — precise enough that it never has to page whole
files through its context.

Documentation is deliberately part of the corpus, not an afterthought:
a project's `docs/**/*.md` and README answer "how do I use this"
questions that no symbol table can, and `@doc`/`@moduledoc` prose
travels inside the shipped `.gan` sources, so one grep spans code and
its documentation in the same sweep.

## Scope

The `map`, `find`, `grep`, and `read` queries in `gan-lsc`; the corpus
definition; the optional `fd`/`rg` acceleration with mandatory
fallbacks; the four forwarding tools in `gan-mcp`. Persistent indexes,
embeddings, and cross-project search are out of scope.

## Terminology

- **Corpus**: the set of files retrieval operates on (R002).
- **Dep sources**: the `.gan` files an installed package ships under
  its `_gan/` directory, enumerated through `gandora.toml` markers.
- **Block**: one definition together with the annotation lines above
  it.

## Specification

**GEP-0031-R001:** The retrieval surface is four `gan-lsc` queries —
`map`, `find`, `grep`, `read` — each returning one JSON value on
stdout (GEP-0015-R001A). `gan-mcp` MUST forward them as the tools
`gan_map`, `gan_search_files`, `gan_search_content`, and `gan_read`,
consulting no model (GEP-0028). Both surfaces return the same value
for the same question.

**GEP-0031-R002:** The corpus is, in this order: the project's Gandora
sources (the configured source roots plus top-level `tests/`); the
project's documentation (`README*` and `*.md` under the root and
`docs/`, recursively); and — behind the `--deps` flag on `find`/`grep`
— the `.gan` sources of every installed package, discovered through
the `gandora.toml` markers under the project's `.venv` (GEP-0006).
Build output, caches, and `.venv` internals other than dep sources are
never in the corpus.

**GEP-0031-R003:** `map` returns the atlas: for the project, each
module with its path and rendered public heads; for the standard
library and each installed package, each module with the path of its
shipped `.gan` source and the same rendered heads. A head is an
orientation line — the signature with the definition's `@doc` first
sentence appended (`def map(xs, f) — Applies \`f\` to every
element.`) — so the map answers "what is here and what does it do"
in one pull. The atlas also lists the documentation files. Everything
is sorted; paths inside the project are project-relative and dep
paths are absolute. `map <Mod>` narrows the atlas to modules whose
name contains `<Mod>`.

**GEP-0031-R004:** `find <pattern>` matches file *names* in the corpus
(substring or glob when the pattern carries `*`/`?`). When `fd` is on
`PATH` it MAY accelerate the project walk; the pure fallback MUST
return the same set. Output is the sorted path list.

**GEP-0031-R005:** `grep <pattern>` searches file *contents*. The
default mode is a regular expression, accelerated by `rg` when present
with an identical-shape pure fallback: a list of `{path, line, text}`
matches in path order. `grep --ranked <words>` is the query mode for
prose: files are scored with BM25 (k1 = 1.2, b = 0.75, whole file as
document, `[a-z0-9_]+` tokens, lowercased) and the answer is the top
files, each with its best matching lines. Both modes cap their output
(200 matches; 10 ranked files) and MUST say so via a `truncated` flag
— silent truncation reads as completeness.

**GEP-0031-R006:** `read` fetches exactly one of: `read <path> <from>
<to>` — the 1-based inclusive line range of a corpus file; `read
<Mod>` — the whole source of a module, project or installed; `read
<Mod.fun>` — the block of one definition: its annotations
(`@doc`/`@param`/`@spec`/`@example`/custom attributes and comments
immediately above) through its last clause, located via the compiler's
own `definition`/`symbols` line data, bounded by the next symbol.
The answer carries `{path, from, to, text}` so the agent can cite and
edit what it read.

**GEP-0031-R007:** Determinism and containment: results are sorted or
score-ordered with documented tie-breaks (path order), never
wall-clock- or environment-dependent; external binaries are an
optimization, never a requirement; queries read files and markers
only — no writes, no network, no model.

## Rationale

One implementation, two surfaces: the queries live in `gan-lsc`
because that is where language intelligence already answers as JSON,
and `gan-mcp` forwards rather than reimplements, exactly as the doc
and pack tools do (GEP-0026/0028). BM25 gets a real formula rather
than "relevance" hand-waving because an agent tuning its own queries
needs the ranking to be predictable; the whole-file document choice
keeps the math honest without an index. `read` exists because the
expensive failure mode of agent retrieval is paging a 600-line file to
use six lines of it — the compiler knows where the six lines are.

## Backwards Compatibility

Purely additive: four new queries, four new tools.

## Security and Determinism

Retrieval reads the project, its docs, and installed markers/sources —
the same trust domain the compiler already reads at build time.
`fd`/`rg` run with fixed argument lists; their absence changes
performance, not answers (R004/R005/R007).

## Tooling and AI Usage

The intended loop: `gan_map` once to orient; `gan_search_files` /
`gan_search_content` to locate; `gan_read` to fetch precisely the
block to change; then the write→`gan build`→`gan test` loop
(GEP-0026). The briefing and pack stay prompt-sized; the map is the
larger atlas an agent pulls on demand.

## Rejected Alternatives

### Tree-sitter grammars and an external indexer

A second parser drifts from the real one, and an index is a cache
with an invalidation problem. The compiler already parses everything
in the corpus, and package sources are on disk by GEP-0006 — walking
them is fast enough at project scale.

### Compiled-artifact search

Searching the generated Python finds mangled names and misses macros
and docs. The `.gan` sources are the artifact of record for humans
and agents alike; the compiled output is for the interpreter.

## Conformance

Tests MUST cover: map shape over a project with std and at least one
installed package; find with substring and glob, with and without
`fd`; grep regex parity between `rg` and the fallback, the ranked
mode's ordering and its `truncated` flag; read by range, by module,
and by definition including the annotation block; and the gan-mcp
forwarding of all four.

## Change History

- Revision 1, 2026-08-08: Initial version.
