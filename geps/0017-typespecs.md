---
gep: 17
title: Typespecs and the Typed Boundary
description: Elixir-style @spec annotations that compile to Python type hints — types are declarations for tooling and the interop boundary, never runtime.
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
  - Interop
created: 2026-08-02
updated: 2026-08-02
revision: 3
requires: [1, 3, 7]
replaces: []
superseded-by: null
resolution: null
translations:
  zh: local/zh/0017-typespecs.md
---

# GEP-0017: Typespecs and the Typed Boundary

## Abstract

`@spec` brings Elixir's typespec surface to Gandora and compiles it to
PEP 484 type hints on the generated Python:

```elixir
@spec mean(list(number()), integer()) :: float()
def mean(xs, precision \\ 2), do: ...
```

becomes `def mean(xs: list[int | float], precision: int = ...) -> float:`.
The dividend is double: generated modules become *typed Python APIs*
for downstream Python callers, and the entire Python type-checking
ecosystem (pyright, mypy) applies to compiled output unchanged.
`$module.Type` references are valid types, so the interop boundary
can be annotated with the host's own types. Specs are declarations —
they add nothing at runtime and are surfaced by docs and the LSP.

## Motivation

Elixir has committed to a type system and Python already has one;
Gandora sits between them and would be poorer than both without a
typing story. The cheapest honest step is Elixir's: typed
*declarations* with tooling value, no runtime checks — realized here
as Python annotations because that is the one target every Python
tool already understands.

## Scope

The `@spec` attribute, the type expression language, its mapping to
Python hints, and doc/LSP surfacing. Gandora-side type *checking*,
`@type` aliases, and struct field types are future revisions.

## Terminology

- **Spec**: a `@spec name(arg_type, ...) :: return_type` attribute.
- **Type expression**: the term language of R002.

## Specification

**GEP-0017-R001:** `@spec fun(type, ...) :: type` immediately before a
definition (or its `@doc` block) attaches to the next `def`/`defp`/
`defmacro` group with that name. `::` and the union operator `|` are
recognized in expressions but valid only inside `@spec`; elsewhere
they are compile errors naming this GEP. A spec whose arity matches
no clause of the following definition is a compile error.

**GEP-0017-R002:** Type expressions and their Python mapping.

Scalars:

| Gandora type | Python hint |
| --- | --- |
| `integer()` | `int` |
| `float()` | `float` |
| `number()` | `int \| float` |
| `boolean()` | `bool` |
| `string()` | `str` |
| `atom()` | `str` |
| `nil` | `None` |
| `any()` / `term()` | `object` |

Concrete containers:

| Gandora type | Python hint |
| --- | --- |
| `list()` / `list(t)` | `list` / `list[t]` |
| `map()` / `map(k, v)` | `dict` / `dict[k, v]` |
| `tuple()` / `tuple(a, b, ...)` | `tuple` / `tuple[a, b, ...]` |
| `keyword()` | `list[tuple[str, object]]` |

Abstract containers — first-class, no host reference needed; prefer
them in parameter positions (they are covariant where the concrete
containers are invariant):

| Gandora type | Python hint |
| --- | --- |
| `iterable()` / `iterable(t)` | `collections.abc.Iterable[t]` |
| `sequence()` / `sequence(t)` | `collections.abc.Sequence[t]` |
| `mapping()` / `mapping(k, v)` | `collections.abc.Mapping[k, v]` |
| `fun()` | `collections.abc.Callable` |

Compositions and the boundary:

| Gandora type | Python hint |
| --- | --- |
| `a \| b` | `a \| b` |
| `$mod.Type` | `mod.Type` (module import recorded) |
| `$mod.Type(t, ...)` | `mod.Type[t, ...]` — parametrized host types |
| `Mod.t()` | the struct class generated for `Mod` (GEP-0004) |

`$mod.Type` is for host-*specific* types (`$np.ndarray`,
`$re.Pattern`, `$decimal.Decimal`); the standard abstractions above
are built in precisely so idiomatic specs never need an explicit
module boundary. Any other expression is a compile error naming this rule.

**GEP-0017-R003:** Compilation annotates the generated Python
function: a single plain-parameter clause receives parameter hints in
order plus the return hint; any group that dispatches through
`*args` (multi-clause, pattern parameters, or `\\` defaults)
receives the return annotation on the dispatcher. Emission is
deterministic and adds no imports beyond those the hints name. Specs
MUST NOT produce runtime checks, wrappers, or module attributes.

**GEP-0017-R005:** A bare lowercase name of at most two characters in
a type position is a **type variable**: `@spec map(list(a), fun()) ::
list(b)`. Every variable compiles to one module-level
`typing.TypeVar` declaration (`_T_a = typing.TypeVar("_T_a")`),
emitted deterministically, and repeated uses unify. A longer bare
name is a compile error suggesting the `name()` spelling — the guard
against a forgotten-parentheses typo silently becoming a variable.
Elixir's `when a: var` clause is accepted future syntax, not yet
required.

**GEP-0017-R004:** Specs join the documentation channel: `gan doc`
prints them above the prose, `gandora_core.doc` returns them as
`specs`, and the LSP shows them in hover and signature help
(GEP-0015).

## Rationale

Compiling to hints instead of inventing a checker means every
consumer — pyright in CI, an IDE on the generated code, a Python
caller importing a Gandora wheel — gets value on day one, and a
future Gandora-side checker can still be layered on the same
declarations. `$mod.Type` in specs makes the interop boundary the
*most* typed part of the language rather than the least. `atom()`
mapping to `str` follows the GEP-0001-R009 value mapping honestly.

## Backwards Compatibility

Additive: `::` and `|` were previously parse errors in expressions.

## Security and Determinism

Hints are text in the generated source; nothing executes.

## Tooling and AI Usage

Agents SHOULD write `@spec` on public functions and MAY run pyright
or mypy over `outDir` as a typed-lint of Gandora programs.

## Rejected Alternatives

### A Gandora-side type checker first

A checker without declarations checks nothing; declarations without a
checker are still hints, docs, and typed APIs. Declarations first.

### Runtime type assertions

Violates GEP-0001-R002 zero-runtime and Elixir's own stance: specs
document and tools verify; code does not pay at call time.

## Conformance

Tests MUST cover: every R002 mapping including unions, parametrics,
`$mod.Type`, and `Mod.t()`; the arity-mismatch and misplaced-`::`
errors; multi-clause and default-argument emission shapes; doc and
LSP surfacing; and a pyright run over annotated output accepting a
well-typed module.

## Change History

- Revision 3, 2026-08-02: The abstract containers (`iterable`,
  `sequence`, `mapping`) and `keyword()`/`term()` became built-in
  types; type variables added (R005, compiled to `typing.TypeVar`);
  R002 reorganized with the guidance that `$mod.Type` is for
  host-specific types only, and `$mod.sub.Type` chains resolve by the
  GEP-0003-R010 rule with no quoting needed.

- Revision 2, 2026-08-02: R002 — parametrized host types
  `$mod.Type(t, ...)` map to subscripted hints, e.g.
  `$"collections.abc".Sequence(number())` becomes
  `collections.abc.Sequence[int | float]`.

- Revision 1, 2026-08-02: Initial version.
