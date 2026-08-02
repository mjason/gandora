---
gep: 3
title: Python Interop
description: Wrapper-free access to Python through first-class $module references, pyimport, postfix access, and decorator declarations.
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
  - Interop
created: 2026-08-01
updated: 2026-08-02
revision: 3
requires: [1]
replaces: []
superseded-by: null
resolution: null
translations:
  zh: local/zh/0003-python-interop.md
---

# GEP-0003: Python Interop

## Abstract

Gandora reaches into its host with one sigil: `$` names a foreign
Python module, and everything after it is direct access.
`$math.sqrt(2.0)` compiles to `import math` plus `math.sqrt(2.0)` with
no wrapper, registry, or runtime shim, and `$math` alone is the module
object itself — a first-class value. Atoms (`:ok`) are pure data and
never name modules. `pyimport` declares aliased imports, postfix
`.name` access works on any value, and `@decorate` attaches Python
decorators to generated functions.

## Motivation

The value of compiling to Python is the ecosystem. Interop must therefore
be the cheapest construct in the language: any installed module callable
immediately, with the compiler doing nothing at compile time but recording
the import — never importing or executing Python itself (GEP-0001-R002,
Security section of GEP-0001).

## Scope

Remote atom calls, `pyimport`, postfix attribute and method access,
decorator declarations, and calling conventions (positional and keyword
arguments). Typed FFI declarations and static verification of foreign
signatures are deferred to a future GEP.

## Terminology

- **Remote reference**: `$module`, `$module.function(args)`, or
  attribute read `$module.attribute`.
- **Foreign module**: the Python module named by a `$` reference or
  `pyimport`.

## Specification

**GEP-0003-R001:** `$name` is a remote reference to the Python module
`name` — any identifier, including uppercase (`$PIL`). A call through
it compiles to a module-level import plus the direct expression:
`$math.sqrt(x)` becomes `import math` and `math.sqrt(x)`. A dotted
module uses the quoted form: `$"os.path".join(a, b)` compiles to
`import os.path` and `os.path.join(a, b)`.

**GEP-0003-R002:** `$module.name` without a call is an attribute read
(`sys.argv` for `$sys.argv`). `$module` alone evaluates to the module
object and is a first-class value: it can be bound, passed, stored in
collections, and used wherever Python accepts a module —
`m = $math; m.sqrt(4.0)`, `rescue e in $builtins.ValueError`.

**GEP-0003-R010:** A multi-segment reference `$a.b.c...` resolves
without quoting: the leading run of lowercase segments — stopping
before the final segment — is the module path, imported whole
(`import a.b`), and the remaining segments are attribute access.
`$collections.abc.Sequence` imports `collections.abc`;
`$importlib.metadata.version(x)` imports `importlib.metadata`;
`$os.path.join(a, b)` imports `os.path`; `$math.pi` imports `math`.
Dotted imports are always safe, so the rule is deterministic and
independent of whether a parent package re-exports its submodules.
The quoted form `$"a.b"` remains as the explicit override for the one
ambiguous residue: a bare reference whose final lowercase segment is
itself a submodule needing an explicit import.

**GEP-0003-R009:** Atoms are pure data (GEP-0001-R010) and never name
modules. An atom followed by `.` and an identifier is a compile error
whose message names this rule and shows the `$` spelling — the
migration diagnostic for revision-1 sources.

**GEP-0003-R003:** `pyimport module` and `pyimport module, as: alias` are
top-level declarations compiling to `import module` /
`import module as alias`. The alias is then usable as a plain identifier:
`np.array([1, 2])`. `pyimport` MUST appear before any definition in the
module body. Compiling a `pyimport` MUST NOT import the module at compile
time.

**GEP-0003-R004:** Postfix access applies to any expression: `expr.name`
compiles to attribute access and `expr.name(args)` to a method call. Chains
compose left to right: `df.rolling(5).mean()`. When `expr` is an
uppercase-leading Gandora module path, the reference is a Gandora
cross-module call (GEP-0001-R017), not interop; otherwise it is plain
Python attribute access.

**GEP-0003-R005:** Calls (remote, method, local, and captured) MUST support
Elixir keyword-argument syntax in the final position, compiling to Python
keyword arguments: `$json.dumps(data, indent: 2)` compiles to
`json.dumps(data, indent=2)`. Keyword keys MUST be valid Python identifiers
after the GEP-0001-R015 mapping.

**GEP-0003-R006:** `@decorate expr` immediately before a `def` attaches the
expression as a Python decorator on the generated function, in source
order when repeated (nearest decorator innermost, matching Python's `@`
stacking). The expression is compiled but not evaluated at compile time.

**GEP-0003-R007:** Values cross the boundary by the GEP-0001-R009 mapping
with no conversion layer; foreign values are dynamically typed and the
compiler performs no static checking on foreign calls in v0.

**GEP-0003-R008:** Foreign dependencies are ordinary entries in
`pyproject.toml` resolved by `uv`; the compiler MUST NOT maintain its own
package metadata for foreign modules and MUST NOT verify at compile time
that a foreign module is installed. A missing module fails at runtime with
Python's standard `ModuleNotFoundError`.

## Rationale

Revision 1 borrowed Elixir's `:erlang` convention, but the pun is
semantically true only in Elixir, where modules literally are atoms.
Here `:math` the value was a string while `$math.sqrt` was a module
reference — one spelling, two meanings, distinguishable only by the
trailing dot, and module references could never be first-class. `$`
splits the roles: `:` is always data, `$` is always the host
environment (the shell-variable intuition), the two highlight
differently, and `$module` gains honest module-object semantics.
It needs no declaration for one-off calls and compiles to exactly the
Python a reviewer expects. `pyimport` remains for the aliased,
repeated-use case (`np`, `pd`) where references would be noisy.

Not checking foreign modules at compile time preserves the guarantee that
compilation never imports Python — the property that keeps builds
deterministic and safe (the compiler reads only static metadata).

## Backwards Compatibility

Revision 2 is a breaking syntax change: `$module.name` references from
revision 1 no longer compile. The R009 diagnostic points every such
site at the `$` spelling, and the migration is a mechanical rewrite;
compiled-output shapes are unchanged.

## Security and Determinism

The compiler records imports as text; it never executes or imports foreign
code, so a hostile package cannot run during compilation. Everything a
program can do at runtime is exactly what the equivalent hand-written
Python could do.

## Tooling and AI Usage

Agents should prefer remote `$` references for one-off standard-library use and
`pyimport ... as:` for libraries used repeatedly, and should not generate
wrapper modules around Python APIs — the absence of wrappers is the design.

## Rejected Alternatives

### A declared FFI with typed signatures (extern blocks)

Per-function `extern` declarations would give static checking but cost a
declaration per function, which contradicts the goal that Python use should
need near-zero ceremony. Typed declarations remain open as an additive
future GEP.

### Compile-time import verification

Importing modules during compilation would catch typos earlier but breaks
determinism, slows builds, executes arbitrary code, and couples compilation
to the state of a virtual environment.

## Open Questions

None for v0.

## Conformance

Tests MUST cover: `$` calls with plain and quoted (dotted) modules,
attribute reads, first-class module-object references (bound and
passed), the R009 atom-dot diagnostic, `pyimport` with and without
`as:`, postfix chains on expressions, keyword arguments on every call kind,
decorator stacking order, and the absence of compile-time imports (compiling
a file referencing a nonexistent module succeeds).

## Change History

- Revision 3, 2026-08-02: Added R010 — dotted `$` chains resolve by
  the lowercase-prefix import rule; quoting becomes a rare explicit
  override.

- Revision 2, 2026-08-02: Interop moved from atom calls to the `$`
  sigil (R001/R002 rewritten, R009 added): `:` is now pure data, and
  `$module` is a first-class module reference.

- Revision 1, 2026-08-01: Initial version.
