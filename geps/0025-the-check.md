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
updated: 2026-08-07
revision: 4
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
printed. `ganc build` (plumbing) remains ungated. `gan run` gates on
the same verdict with suggestions suppressed — errors stop a run, the
teaching pass belongs to `check`/`build`.

**GEP-0025-R004 (the traffic light):** the `lsc check` verdict leads
with two booleans: `"ok"` (no errors — the program compiles) and
`"clean"` (ok, and no warnings, and no suggestions). An agent's
loop is: red (`ok: false`) → fix errors; yellow (`ok` but not
`clean`) → read the suggestions; green (`clean: true`) → submit.

**GEP-0025-R005 (the trust line):** an idiomatic project MUST verdict
zero suggestions — noise teaches agents to ignore the Advisor. The
reference bar: the language's own std, toolchain, tour, and playground
stay suggestion-free under their own check. Rules earn their place by
surviving that bar: `$builtins` is exempt from the pyimport-repetition
hint (it is the ambient namespace), `rescue _e ->` is a deliberate
swallow and not a bare-rescue offence, `Mod.t()` in `@spec`/`@type`
lines is a type spelling and never a member typo, `$mod.Type()` on
`@spec` lines is the host-type spelling the spec grammar itself
mandates (GEP-0017-R002) and never counts toward pyimport repetition, single-call `fn`
wraps are only flagged when the callee is capturable (`&f/1` — never
for `g.(x)` dot-calls), and test modules (`use Test` or `TestX`
naming) are consumers, not library surface — annotation-coverage and
doctest nags do not apply to them.

**GEP-0025-R006 (the project surface):** the verdict covers the
configured source roots plus top-level `tests/*.gan` — exactly what
`gan test` runs; deeper directories under `tests/` are fixtures and
are not advised. Test files receive the Advisor pass only; their
compile diagnostics belong to `gan test`.

**GEP-0025-R007 (consolidation and anchors):** identical suggestion
messages from many files collapse to one entry annotated with the
spread (`(also in N other file(s))`), and every suggestion carries the
1-based line of its first evidence, so an agent can jump straight to
it. Literal masking preserves delimiters and balances nested
parentheses in sigil bodies — `~python(next((...), None))` never
leaks its `None` into a migration hint.

**GEP-0025-R008 (artifact verification):** the verdict includes a
resolution pass over the compiled artifacts: the generated Python is
checked with ty restricted to resolution rules — an undefined name, an
unresolvable import, or a member no module provides is runtime-fatal
fact, so it reports as an **error**, mapped back to the `.gan` source
(demangled name, source line, did-you-mean). Type-flow opinions
(operator support, argument types) stay out of the gate; they never
block a build. The layer degrades to silence when ty is unavailable.
Native extensions expose their surface through shipped `.pyi` stubs
(gandora-core ships one).

**GEP-0025-R009 (build is the verdict):** there is no separate check
command — `gan build` runs the whole verdict (diagnostics, advice,
artifact verification) and stops before writing artifacts on any
error. `gan run` gates on the same verdict with suggestions
suppressed. `gan lsc check` remains the JSON surface of the same
verdict for tools and agents. The retired `gan check` spelling prints
a migration note and delegates to build.

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
For revision 2 it MUST also cover: the `ok`/`clean` traffic light;
each R005 exemption (a test module, a `rescue _e`, a `Mod.t()` spec,
a dot-call `fn` wrap, and `$builtins` repetition each verdict clean);
nested-paren sigil masking; and cross-file consolidation with line
anchors.

## Change History

- Revision 4, 2026-08-07: The practice pass gains the pipeline taste,
  and it is written down: three adjacent nested calls (`f(g(h(x)))`)
  suggest a `|>` pipeline (assertion lines and macro-guarded
  expressions such as `safe/2` exempt; `@spec`/`@type` lines excluded
  — types are calls by design); a bare single-variable
  `for x <- xs, do: f(x)` suggests `Enum.map` when the project
  resolves the stdlib (`for` keeps filter/pattern-skip/`into:`/await
  bodies); the existing map+filter chain rule is corrected to fire
  only on adjacent filter-then-map (map-then-filter has no `for`
  spelling); host interop that GEP-0010-R011 wraps gets one
  consolidated per-file hint pointing at `Path`/`File`/`System`
  (the wrapper module itself exempt). The rules' long form is
  docs/practices.md; `gan lsc doc practices` serves the digest, and
  the `gan agent` briefing and gan-mcp composer prompt carry it.

- Revision 1, 2026-08-04: Initial version — supersedes GEP-0023.
- Revision 3, 2026-08-04: R008 artifact verification (ty resolution
  rules over the compiled Python, mapped back to source; type-flow
  never gates); R009 build subsumes check — one verdict, one command.
- Revision 2, 2026-08-04: R004 traffic light (`ok`/`clean`); R005
  zero-noise trust line with the surviving-rule refinements; R006
  project surface includes top-level tests; R007 cross-file
  consolidation, line anchors, balanced sigil masking.
