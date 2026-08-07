---
gep: 30
title: Async Functions
description: "Native `async def` and `await`, one-to-one with Python's coroutine syntax — the coroutine world speaks its own words, with no annotation and no contextual recompilation."
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
created: 2026-08-06
updated: 2026-08-07
revision: 4
requires: []
replaces: []
superseded-by: null
resolution: null
translations:
  zh: local/zh/0030-async-functions.md
---

# GEP-0030: Async Functions

## Abstract

`async def` and `await` are language syntax, compiled one-to-one to
Python's own: `async def fetch(url)` emits `async def fetch(url)`, and
`await expr` emits `await expr` — bare, with no deadline and no
wrapper. Nothing is recompiled contextually and nothing hides behind
an annotation: the coroutine world's readable Python *is* its syntax,
so Gandora writes that syntax. Deadlines, fan-out, and cancellation
are library concerns and live in `Task` (GEP-0029), which this syntax
makes expressible as an ordinary std module.

## Motivation

Revision 3 tried to keep one uncolored vocabulary across both of
Python's execution worlds: an `@async true` annotation turned a `def`
into `async def`, and inside it `Task.await(t)` recompiled to
`await asyncio.wait_for(t, 5.0)`. The design was measured before it
was replaced, and it failed on the axis that matters most — what a
reader believes the code says:

- One source line meant two different things depending on an
  annotation elsewhere. `Task.await(t)` was a thread join in one body
  and a `wait_for` in another.
- The recompilation smuggled a 5-second deadline into what looked
  like a plain join. In live testing of the MCP surface, the
  model-backed composer — grounded in Gandora's own verified corpus —
  wrote `Task.await(task)` and asserted in prose that it "waits
  without any deadline". The generated artifact said
  `asyncio.wait_for(task, 5.0)`. When the language's own tooling
  cannot read the language's intent, the construct is wrong.
- The generated Python — a reviewer-grade artifact (GEP-0001-R002) —
  was noisier than what any Python author would write: `wait_for`
  wrappers where the natural artifact is a bare `await`.
- `@async true` violated the annotation contract settled in v0.18.8:
  annotations are data that *describe* code. This one changed a
  function's calling convention.

Gleam settles the design question. Its language core has no
concurrency constructs at all; each compilation target gets a library
that exposes the target's native model thinly — `gleam/javascript/promise`
wraps the Promise JavaScript already has, `gleam/erlang/process`
exposes the BEAM directly — and no target ever simulates another
target's model (gleam_otp v1 deleted its BEAM `task` module rather
than keep an abstraction that hid the native one). Gandora has one
target containing two worlds, and the coroutine world's native model
*is syntax*: `async def` and `await`. The language therefore provides
that syntax, verbatim, and everything above it is a library.

## Scope

Covered: the `async def` / `async defp` definition forms, the `await`
prefix expression, precedence, the closure and comprehension
boundaries, doctests, the entry-point rule, and the withdrawal of the
`@async` annotation. Out of scope: `async for` / `async with` surface
syntax (recursion compiles to loops, and the awaited
`__aenter__`/`__anext__`/`__aexit__` pattern covers the lifecycles
that matter — verified against a live streaming API before revision
3), async generators, and the `Task` module itself (GEP-0029).

## Terminology

**Async body** — the body of a function defined with `async def` or
`async defp`, excluding any `fn` closure written inside it.

**Coroutine** — what calling an async function returns, in both
worlds; it runs only when awaited or scheduled.

## Specification

**GEP-0030-R001 (async def):** `async def` and `async defp` define a
function that compiles to Python's `async def`, private (`defp`)
functions keeping their name mangling. Calling one is an ordinary
call returning a coroutine object, in both worlds. All clauses of a
multi-clause function MUST carry the same modifier — mixing `def` and
`async def` clauses is a compile error, as mixing `def` and `defp`
already is. `async def main` is a compile error: the entry point
stays synchronous and enters the coroutine world with `Task.run`
(GEP-0029).

**GEP-0030-R002 (await):** `await expr` is a prefix expression
compiling to Python's `await` — bare, no deadline, no wrapper.
It binds tighter than every binary operator (including `|>`) and
looser than call, attribute, and index chains — Python's own
precedence — so `await fetch(u) |> parse()` awaits the fetch and
pipes the value, and `await a + b` awaits `a` before adding. `await`
is legal only in an async body; anywhere else it is a compile error
naming this rule.

**GEP-0030-R003 (the closure boundary):** A `fn` body is a
synchronous function even when written inside an async body — Python
lambdas cannot await, and Gandora does not pretend otherwise. `await`
inside `fn` is a compile error. Comprehension bodies compile to
native Python comprehensions in the enclosing function, so `await` is
legal there: `for t <- tasks, do: await t` is the sequential join,
emitted as `[await t for t in tasks]`.

**GEP-0030-R004 (contextual keywords):** `async` and `await` are
claimed only in their positions — `async` immediately before
`def`/`defp`, `await` immediately before an expression. Elsewhere
both remain ordinary identifiers, so no existing name breaks. Using
them as function or variable names is legal but advised against; the
Advisor MAY say so.

**GEP-0030-R005 (doctests run):** `@example` on an async function is
an ordinary doctest — written through the sync rim, it executes:
`gan> Task.run(M.fetch("x"))` is a runnable line. The
displayed-not-run exception of revision 3 is withdrawn; an example
that cannot run is once again a defect, not a category.

**GEP-0030-R006 (no inference):** A function without the modifier
never compiles to `async def`, whatever its body does. The artifact's
signature is a declaration, visible in the diff and stable under
refactors of the body.

**GEP-0030-R007 (the annotation is withdrawn):** `@async` is an
ordinary module attribute again, carrying data and changing nothing —
the revision 3 reinterpretation is withdrawn, and with it the entire
contextual compilation table: `Task.await`, `Task.async`,
`Task.await_many`, `Task.try_await` are ordinary std calls everywhere
(GEP-0029 defines what they do), and the compiler emits no hidden
helper functions.

## Rationale

**Syntax, not annotation.** Async-ness is a signature fact — it
changes what a call returns — and signatures live in the head, not in
an attribute above it. The annotation route also contradicted the
v0.18.8 settlement that annotation values are data; `@async true` was
the one annotation that rewired semantics, and it is gone.

**No default deadline.** Revision 3 gave the bare join a hidden
5-second `wait_for` to keep "one source line, one meaning" across
worlds. With `Task.await` gone the identity holds trivially — `await`
appears only in async bodies and means exactly Python's `await`. The
observed cost of the hidden deadline — a false claim produced by the
language's own AI surface, and artifact noise a reviewer must read
past — bought nothing that explicit `Task.try_await(t, ms)` does not
provide visibly. `:infinity` dies with it: the bare `await` *is* the
no-deadline spelling.

**Two worlds, stated plainly.** Sync Gandora and async Gandora are
both real, as Gleam's two targets are both real. The border is
explicit — `Task.run` enters the coroutine world, `Task.blocking`
reaches back to the blocking one (GEP-0029) — rather than blurred by
a vocabulary that compiles differently per side. Function coloring is
Python's design; hiding it made artifacts lie, so Gandora shows it.

**Contextual keywords, not reserved words.** `async`/`await` claimed
everywhere would break `Task.async` itself. Claiming only the two
juxtapositions costs one lookahead in the parser and breaks no
existing code.

## Backwards Compatibility

The `@async true` reinterpretation existed only in an unreleased
working tree; withdrawing it reverts `@async` to what every other
attribute is. The `async def` juxtaposition and `await` prefix were
parse errors before this GEP, so no existing module changes meaning.

## Security and Determinism

No new boundary: an async function runs on whatever event loop awaits
it, with the same trust as its caller. The compiler adds no deadline,
no scheduling, and no state; everything timing-related is explicit in
source and artifact alike.

## Tooling and AI Usage

The construct index carries `async def` and `await` cards; the MCP
corpus atom shows an async interior joined by a sync rim whose
doctest runs (R005). The formatter prints the forms as written —
`async def f(x) do` and `await expr` — via the printer's block-form
and prefix spellings. Agents are told: the coroutine world is native
syntax; deadlines and fan-out are `Task` library calls, visible in
the artifact.

## Conformance

Tests MUST cover: one-to-one emission for `async def` and `async
defp` (typed and untyped); bare `await` emission with no wrapper;
precedence — `await f(x) |> g()` pipes the awaited value, `await a +
b` awaits before adding; `await` in a comprehension body emitting a
native comprehension; the compile errors — `await` outside an async
body, `await` inside `fn`, mixed `def`/`async def` clauses, `async
def main`; a doctest through `Task.run` executing; and an end-to-end
module — an async interior fanned out and joined from a sync rim —
passing `gan test`.

## Change History

- Revision 4, 2026-08-07: native syntax. `async def`/`async defp` and
  the `await` prefix expression replace `@async true`; the contextual
  Task compilation table, the emitted `_gan_try_await` helper, the
  displayed-not-run doctest exception, and `:infinity` are withdrawn.
  Deadlines and fan-out move wholly into GEP-0029's Task.
- Revision 3, 2026-08-07: `:infinity` compiles to the bare `await`
  (and bare `gather` for `await_many`) — no deadline means no
  `wait_for` in the artifact.
- Revision 2, 2026-08-06: R002 follows the GEP-0029 rev 3 vocabulary —
  `try_await` compiles natively via an emitted async helper; the
  combinators and `try_await_many` are compile errors in `@async`
  bodies.
- Revision 1, 2026-08-06: Initial version — `@async true` annotation
  with contextual Task compilation.
