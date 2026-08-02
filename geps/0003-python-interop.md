---
gep: 3
title: Python Interop
description: Wrapper-free access to Python through remote atom calls, pyimport, postfix access, and decorator declarations.
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
  - Interop
created: 2026-08-01
updated: 2026-08-01
revision: 1
requires: [1]
replaces: []
superseded-by: null
resolution: null
translations:
  zh: local/zh/0003-python-interop.md
---

# GEP-0003: Python Interop

## Abstract

Gandora reaches Python the way Elixir reaches Erlang: an atom names the
foreign module, and a call on that atom is a direct call into it.
`:math.sqrt(2.0)` compiles to `import math` plus `math.sqrt(2.0)` with no
wrapper, registry, or runtime shim. `pyimport` declares aliased imports,
postfix `.name` access works on any value, and `@decorate` attaches Python
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

- **Remote atom call**: `:module.function(args)` or attribute read
  `:module.attribute`.
- **Foreign module**: the Python module named by an atom or `pyimport`.

## Specification

**GEP-0003-R001:** An atom in call position names a foreign Python module:
`:math.sqrt(x)` compiles to a module-level `import math` and the expression
`math.sqrt(x)`. A dotted module uses a quoted atom: `:"os.path".join(a, b)`
compiles to `import os.path` and `os.path.join(a, b)`.

**GEP-0003-R002:** `:module.name` without a call is an attribute read
(`sys.argv` for `:sys.argv`). `:module` alone evaluates to the module
object.

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
keyword arguments: `:json.dumps(data, indent: 2)` compiles to
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

The `:erlang`-style atom call is the smallest possible interop surface: it
needs no declaration for one-off calls, reads unambiguously (a leading `:`
marks the foreign boundary), and compiles to exactly the Python a reviewer
expects. `pyimport` exists for the aliased, repeated-use case (`np`, `pd`)
where atoms would be noisy.

Not checking foreign modules at compile time preserves the guarantee that
compilation never imports Python — the property that keeps builds
deterministic and safe (the compiler reads only static metadata).

## Backwards Compatibility

Founding interop proposal. R001–R005 syntax and the compiled-output shapes
are the compatibility contract.

## Security and Determinism

The compiler records imports as text; it never executes or imports foreign
code, so a hostile package cannot run during compilation. Everything a
program can do at runtime is exactly what the equivalent hand-written
Python could do.

## Tooling and AI Usage

Agents should prefer remote atom calls for one-off standard-library use and
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

Tests MUST cover: atom calls with plain and quoted (dotted) modules,
attribute reads, module-object references, `pyimport` with and without
`as:`, postfix chains on expressions, keyword arguments on every call kind,
decorator stacking order, and the absence of compile-time imports (compiling
a file referencing a nonexistent module succeeds).

## Change History

- Revision 1, 2026-08-01: Initial version.
