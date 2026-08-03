---
gep: 14
title: Control-Flow Completion
description: try/rescue/after in Elixir syntax; the loop/break construct served until GEP-0019 recursion retired it, with recur surviving as the function-level jump.
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
created: 2026-08-02
updated: 2026-08-02
revision: 3
requires: [1]
replaces: []
superseded-by: null
resolution: null
translations:
  zh: local/zh/0014-control-flow-completion.md
---

# GEP-0014: Control-Flow Completion

## Abstract

Two constructs complete the control-flow story that real programs —
the task runner, REPL, and LSP server of GEP-0013+ — require.
`try/rescue/after` adopts Elixir's syntax over Python's exception
machinery. `loop` binds a state pattern and repeats its body on
`recur(new_state)` until `break(value)` (or body completion) ends it —
the honest substitute for tail recursion on a VM without TCO, in the
Clojure `loop/recur` tradition.

## Motivation

GEP-0001 deferred `try/rescue`; servers cannot survive a bad request
without it. And Gandora has no unbounded iteration: Elixir servers
recurse, but Python's recursion limit (~1000 frames) makes that a
crash, not a pattern. Writing the toolchain in Gandora (GEP-0012's
purpose) forces both gaps closed now.

## Scope

The two constructs and their compilation. `raise/2` with exception
types, `defexception`, `catch`/`throw`, and comprehension `for` remain
deferred.

## Terminology

- **Rescue clause**: `pattern -> body` inside `rescue`, where the
  pattern is a variable or `var in ExceptionRef`.
- **Loop state**: the pattern bound by `loop` and rebound by `recur`.

## Specification

### try/rescue/after

**GEP-0014-R001:** The form is Elixir's:

```elixir
try do
  body
rescue
  e in $builtins.ValueError -> handled(e)
  e -> fallback(e)
after
  cleanup()
end
```

`rescue` holds one or more `->` clauses tried top to bottom; `after`
is optional and always runs. The whole form is an expression yielding
the taken branch's value (`after` contributes no value, as in Elixir).

**GEP-0014-R002:** A rescue pattern `e in Ref` matches when the raised
Python exception is an instance of `Ref` (any expression evaluating to
an exception type — typically an interop reference like
`$builtins.ValueError` or `$json.JSONDecodeError`); bare `e` matches
every `Exception`. The bound variable is the Python exception object,
whose fields are reachable through ordinary interop
(`e.args`, `to_string(e)`).

**GEP-0014-R003:** Compilation targets Python's native machinery:
`try`/`except ... as e`/`finally`, one `except` arm per clause, in
order. `raise` (GEP-0001) composes unchanged. A `try` without
`rescue` or `after` is a compile error naming the form.

### loop/recur/break

**GEP-0014-R004:** The form is:

```elixir
loop state = initial do
  body
end
```

`state` is any pattern, bound to `initial` on entry. Inside the body,
`recur(expr)` rebinds the pattern to `expr` and restarts the body;
`break(expr)` ends the loop with `expr` as its value. A body that
completes without either ends the loop with the body's value. `loop`
is an expression.

**GEP-0014-R005:** `recur` and `break` are valid only inside a `loop`
body (a compile error otherwise, naming the construct); they take
exactly one argument and never return. Nested loops bind them to the
nearest enclosing `loop`. Rebinding uses full pattern matching: a
`recur` value that does not match the state pattern raises the
GEP-0001-R012 match error.

**GEP-0014-R007:** `loop` and `break` are retired. GEP-0019 gives
tail recursion constant stack and `recur` its compiler-checked
function-level meaning, and GEP-0020 gives iteration its `for`
comprehension — the construct's reason to exist (a VM without TCO)
is gone, and Elixir has no `loop`. The migration is mechanical:

```elixir
loop state = init do body end
# becomes
defp step(state) do body end   # recur(x) unchanged; break(v) -> v
step(init)
```

`loop` and `break` produce compile errors carrying this recipe.
R004–R006 below stand as the historical specification.

**GEP-0014-R006:** Compilation is a Python `while True:` with the
state held in a compiler-named variable, the pattern matched at the
top of each iteration, `recur` as assignment plus `continue`, and
`break(v)` as result assignment plus `break` — constant stack depth by
construction.

## Rationale

Mapping `rescue` onto Python exception types rather than inventing a
Gandora exception hierarchy keeps the interop promise: the errors a
program meets *are* Python's, and naming them through interop
references needs no new namespace. `loop/recur` is chosen over a bare
`while` because it keeps Elixir's bind-and-rebind data discipline
(state is a pattern, not mutation) and over TCO-style self-recursion
because Python cannot honor it; Clojure normalized exactly this
compromise on the JVM.

## Backwards Compatibility

Additive. `try`, `loop`, `recur`, and `break` were previously
diagnosed as unsupported or unknown calls.

## Security and Determinism

Both constructs compile to local Python control flow; no new runtime
surface. Determinism (GEP-0001-R024) is unchanged.

## Tooling and AI Usage

Agents should use `loop` for unbounded iteration (servers, REPLs) and
recursion for bounded structure; `rescue` specific exception types
before the bare-variable catch-all; and keep `after` for cleanup only.

## Rejected Alternatives

### A bare `while cond do` loop

Honest to Python but abandons the pattern-rebinding discipline that
makes Elixir state explicit; `loop` costs one line more and keeps it.

### Trampolined self-recursion

Hides the mechanism inside every function call; slower and harder to
read in generated output than one `while True`.

### A Gandora exception hierarchy

Would wrap every Python error twice; the target's exceptions are the
real ones.

## Open Questions

None for this revision.

## Conformance

Tests MUST cover: rescue clause ordering and type matching (interop
types included), the catch-all, `after` running on both paths, `try`
as an expression; `loop` with tuple-pattern state, `recur` restart,
`break` value, body-completion value, nested-loop binding, the
outside-loop diagnostics; and a million-iteration loop proving
constant stack.

## Change History

- Revision 3, 2026-08-03: R007 — `loop`/`break` retired in favor of
  GEP-0019 recursion and GEP-0020 comprehensions; `recur` lives on at
  function level.

- Revision 2, 2026-08-02: Examples updated to the GEP-0003 revision 2 `$` interop syntax.

- Revision 1, 2026-08-02: Initial version.
