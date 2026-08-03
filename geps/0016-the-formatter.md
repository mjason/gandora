---
gep: 16
title: The Formatter
description: gan fmt — conservative, token-verified source formatting written in Gandora over a new lossless gandora-core token API.
author: MJ
status: Accepted
type: Standards Track
areas:
  - Tooling
created: 2026-08-02
updated: 2026-08-02
revision: 2
requires: [1, 12, 13]
replaces: []
superseded-by: null
resolution: null
translations:
  zh: local/zh/0016-the-formatter.md
---

# GEP-0016: The Formatter

## Abstract

`gan fmt` normalizes Gandora source: structural indentation, horizontal
whitespace, blank-line runs, and trailing whitespace — and nothing
else. It never joins or splits the author's lines. Its safety contract
is mechanical: the formatted text MUST tokenize to the same token
sequence (comments included) as the original, or the file is left
untouched. The formatter is a Gandora program inside `gandora-tool`,
built on a new lossless `gandora_core.tokens` API — the compiler stays
the single source of lexical truth.

## Motivation

A formatter ends style debates and gives agents a deterministic target.
The conservative scope — indentation and whitespace, verified by token
round-trip — delivers most of the value of a full pretty-printer with
none of its risk: a formatter bug cannot corrupt a program it cannot
re-tokenize identically.

## Scope

Whitespace-level formatting and the token API behind it. Line
re-wrapping, comment re-flowing, and AST-level canonicalization
(paren insertion, call-style changes) are future revisions.

## Terminology

- **Structural indent**: the column assigned to a line from the
  nesting of `do`/`end`, `fn`, brackets, and clause arrows.
- **Continuation line**: a line that continues the previous
  expression (leading `|>` or binary operator, or following a line
  that ends with `=` or a binary operator).

## Specification

**GEP-0016-R001:** `gandora_core.tokens(text, path \\ "nofile")`
returns the full lexical stream as a list of
`%{"kind" => k, "text" => t, "line" => l, "col" => c, "end_line" => el,
"end_col" => ec}` maps, including `comment` tokens and `newline`
markers, in source order. `tokens` MUST be lossless enough that the
token kinds/texts plus the whitespace between spans reconstruct the
file. Parsing continues to ignore comments.

**GEP-0016-R002:** `gan fmt [path ...]` formats `.gan` files in place
(project `source` directories when no path is given);
`gan fmt --check [path ...]` writes nothing and exits 1 if any file
would change. Output and exit codes follow GEP-0001-R023.

**GEP-0016-R003:** The formatter rewrites only:
indentation (two spaces per structural level); spaces around binary
operators and after commas outside string-like tokens; trailing
whitespace; runs of two or more blank lines (collapsed to one); and a
missing final newline. It MUST NOT join lines, split lines, or alter
the interior of strings, heredocs, or sigil bodies except for the
uniform shift of R005 and the R008 capture parenthesization.

**GEP-0016-R004:** Structural indent: `do`, `fn`, and each unclosed
bracket push a level; `end` and closing brackets pop before the line
they start; `else`, `rescue`, and `after` lines sit one level below
their block body. A line ending in `->` opens a clause body one level
deep, closed by the next clause head, `end`, or closing bracket.
Continuation lines sit one level below their anchor. Comment-only
lines take the structural indent of the position they occupy.

**GEP-0016-R005:** A multi-line string, heredoc, or sigil body moves
as a unit: every interior line and the closing delimiter shift by
exactly the indent delta applied to the opening line, preserving the
GEP-0001-R026 dedented value byte-for-byte.

**GEP-0016-R006:** Before replacing a file, the formatter MUST verify
both that the comment sequence of the new text equals the original's
(trailing whitespace aside) and that `gandora_core.parse` yields equal
quoted terms for the old and new text. On mismatch it MUST leave the
file unchanged and report an internal error naming this rule.
Formatting MUST be idempotent: `fmt(fmt(s)) == fmt(s)`.

**GEP-0016-R007:** The formatter is implemented in Gandora inside
`gandora-tool` as a native runner command (GEP-0013-R002 family); it
uses only the R001 token API — no private compiler state.

**GEP-0016-R008:** A capture whose body begins with a `$` reference
is parenthesized: `&$math.sqrt/1` becomes `&($math.sqrt/1)` — the
one token-inserting rewrite, covered by the R006 parse check. Other
capture spellings are left as written.

**GEP-0016-R009:** `gan fmt -` reads a document from stdin and writes
its canonical form to stdout — the pipe- and editor-integration
spelling. R006 verification still gates the output: on failure the
input passes through unchanged, a note goes to stderr, and the exit
code is 2. A clean run exits 0 whether or not anything changed.

**GEP-0016-R010:** `gan fmt --diff [path ...]` prints a unified diff
of what formatting would change and rewrites nothing; it exits 1 when
any file would change, 0 otherwise, mirroring `--check`'s contract
with evidence attached.

## Rationale

Token-verified conservatism over a pretty-printer: a full formatter
needs a comment-attached concrete syntax tree the compiler does not
keep, and its failure mode is silent corruption. R006 turns the worst
case into a no-op with an error message. Keeping author line breaks
respects the pipeline-heavy style the language encourages, where line
structure is meaning.

## Backwards Compatibility

Additive: one new core API, one new runner command.

## Security and Determinism

The formatter reads and writes project sources only, executes nothing,
and is deterministic over file content.

## Tooling and AI Usage

Agents SHOULD run `gan fmt` after generating code and MUST NOT depend
on formatting-sensitive diffs. `--check` is the CI surface.

## Rejected Alternatives

### A full AST pretty-printer (mix format style)

Requires a lossless concrete syntax tree with attached comments — a
second parser. Deferred until the token-level formatter proves the
demand.

### Implementing fmt in Rust

The toolchain-in-Gandora program (GEP-0013) exists precisely for
tools like this; the compiler contributes the token API and nothing
else.

## Conformance

Tests MUST cover: fixture files exercising every R003/R004/R005 rule;
idempotency on every fixture and on the repository's own sources;
R006 verification (a deliberately broken rewrite is refused); `--check`
exit codes; heredoc value preservation under re-indentation; an R009
stdin round-trip with exit codes; and an R010 diff run that leaves
files untouched.

## Change History

- Revision 2, 2026-08-03: R009 `gan fmt -` (stdin to stdout, verified,
  exit 0/2), R010 `--diff` (unified diff, no rewrite, exit 1 when
  changes exist).
- Revision 1, 2026-08-02: Initial version.
