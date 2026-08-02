---
gep: 4
title: Structs and Module Attributes
description: defstruct compiled to frozen dataclasses, struct literals, patterns and updates, and module attributes as import-time bindings that enable stateful Python decorators.
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
  - Interop
created: 2026-08-01
updated: 2026-08-02
revision: 2
requires: [1, 3]
replaces: []
superseded-by: null
resolution: null
translations:
  zh: local/zh/0004-structs-and-module-attributes.md
---

# GEP-0004: Structs and Module Attributes

## Abstract

This proposal adds two module-level data declarations to the v0 surface.
`defstruct` declares the struct of its module and compiles to a frozen
Python `dataclass`, giving Gandora typed-record ergonomics — construction
`%User{name: "MJ"}`, pattern matching `%User{name: n}`, and functional
update `%User{u | age: 2}` — on top of a class any Python library
understands. Module attributes `@name expr` compile to module-level
assignments evaluated at import time, which makes stateful Python
decorators (`@decorate @app.route("/")` after `@app $flask.Flask(...)`)
expressible without wrapper modules.

## Motivation

GEP-0001 deliberately excluded `defstruct`, and GEP-0003's `@decorate`
covers only decorators reachable as pure expressions. Practical Python
reuse hits both limits quickly: structured data wants named, defaulted
fields rather than raw maps, and the most common decorator idiom in the
Python ecosystem — Flask, FastAPI, Celery, atexit-style registries —
decorates functions with methods of a module-level object. Both gaps close
with declarations that map one-to-one onto idiomatic Python: dataclasses
and module-level constants.

## Scope

`defstruct` declarations; struct literal, pattern, and update syntax; the
plain-map update form `%{m | k: v}`; module attributes and attribute
reads. Compile-time field validation across modules, struct typing,
protocols, and derivation are out of scope.

## Terminology

- **Struct**: the single named record type a module may declare with
  `defstruct`.
- **Struct class**: the generated Python dataclass representing a struct.
- **Module attribute**: a named module-level binding declared with
  `@name expr`, distinct from the reserved attributes `@doc`,
  `@moduledoc`, and `@decorate`.

## Specification

### Struct declaration

**GEP-0004-R001:** `defstruct` MUST appear only at module top level, at
most once per module. Its argument is either a keyword list of
`field: default` pairs, a list of atoms (fields defaulting to `nil`), or a
list mixing both. Field order is declaration order.

**GEP-0004-R002:** The struct of module `App.User` MUST compile to a
Python `dataclasses.dataclass` with `frozen=True`, named after the final
module segment (`User`), defined in that module's generated file before
module attributes and functions. Field defaults compile in declaration
order; a default that is a list, tuple, or map literal MUST compile to a
`default_factory` so mutable defaults are per-instance.

**GEP-0004-R003:** Struct instances are immutable: assigning a field
raises the standard `dataclasses.FrozenInstanceError`. Functional update
(R006) is the supported way to derive changed values.

### Struct expressions and patterns

**GEP-0004-R004:** `%Mod{field: value, ...}` constructs a struct.
`Mod` resolves like any module reference (aliases apply); when it resolves
to the enclosing module the generated code uses the local class name,
otherwise it imports the target module. Construction compiles to a
keyword-argument constructor call; omitted fields take their defaults.

**GEP-0004-R005:** In v0 the compiler performs no cross-module field
validation; naming a nonexistent field fails at runtime with Python's
standard `TypeError`. A future revision MAY add static validation.

**GEP-0004-R006:** `%Mod{expr | field: value, ...}` is functional update:
it evaluates to a new struct equal to `expr` with the named fields
replaced, compiling to `dataclasses.replace`. Field access remains the
GEP-0003-R004 postfix form (`user.name`).

**GEP-0004-R007:** `%Mod{field: pattern, ...}` in pattern position MUST
match exactly instances of the struct class (including the type check)
and then match each named field's value against its subpattern. Omitted
fields are unconstrained.

**GEP-0004-R008:** `%{expr | key: value, ...}` (no module) is plain-map
update compiling to a dict merge (`{**expr, "key": value}`). Unlike
Elixir, v0 does not raise when a key is absent; this divergence MUST be
documented and MAY be tightened by a future GEP.

### Module attributes

**GEP-0004-R009:** At module top level, `@name expr` where `name` is not
one of the reserved attributes (`doc`, `moduledoc`, `decorate`) declares
the module attribute `name`. It MUST compile to a module-level Python
assignment evaluated at import time, emitted in source order after
imports and the struct class and before function definitions.

**GEP-0004-R010:** An attribute initializer MAY reference imports,
`pyimport` aliases, the module's struct, and attributes declared earlier
in the module. Each attribute MUST be declared exactly once; redeclaring
one is a compile error.

**GEP-0004-R011:** `@name` in expression position — including inside
`@decorate` expressions and postfix chains such as
`@decorate @app.route("/")` — reads the attribute and compiles to the
module-level name. Reading an undeclared attribute is a compile error.

**GEP-0004-R012:** Unlike Elixir, a Gandora module attribute is an
import-time runtime binding, not a compile-time constant that is inlined.
This divergence is deliberate (Rationale) and MUST be documented in
user-facing material.

## Rationale

A frozen dataclass is the closest Python object to an Elixir struct:
named defaulted fields, structural equality, `match` support with keyword
patterns, and a canonical repr — all generated by the standard library,
readable in the output, and consumable by any Python code. `frozen=True`
preserves Elixir's immutability contract and makes `dataclasses.replace`
the natural update form.

Module attributes diverge from Elixir's compile-time semantics because
the dominant use case — decorator state like a Flask app — requires one
shared runtime object, not a value inlined at each use site. Inlining
`@app $flask.Flask(...)` would create one application per reference,
which is never what the author means. Import-time assignment matches what
a Python author would write by hand.

The struct update spelling `%Mod{expr | ...}` (rather than extending
`%{expr | ...}`) is required because structs compile to classes, not
dicts, so the two updates need different generated code and the compiler
cannot distinguish them at runtime cost zero; Elixir accepts the same
qualified spelling.

## Backwards Compatibility

Purely additive to GEP-0001/0003. Existing programs compile unchanged.
The reserved-attribute set is unchanged; only previously-rejected syntax
becomes meaningful.

## Security and Determinism

Attribute initializers and struct defaults are ordinary compiled
expressions executed at import time by Python, exactly like hand-written
module-level code; the compiler still never imports or executes anything
at compile time. Generated output remains deterministic.

## Tooling and AI Usage

Agents should prefer structs over ad-hoc maps once a shape has a name,
use `%Mod{expr | ...}` instead of rebuilding structs field by field, and
declare decorator state (`Flask`, registries, sessions) as module
attributes rather than generating wrapper Python modules.

## Rejected Alternatives

### Compile structs to plain dicts with a type tag

Closer to Elixir's map-based structs and open to `%{... | ...}` updates,
but every Python consumer would see dicts instead of typed objects,
losing attribute access, `isinstance`, and library interop — the main
reason to compile to Python at all.

### Elixir-faithful compile-time attributes with inlining

Would keep `@name` semantics identical to Elixir but makes the flagship
decorator use case (one shared app object) impossible to express, and
constant inlining is an optimization Python does not need from us.

### Mutable dataclasses (`frozen=False`)

Friendlier to mutating Python libraries but silently abandons Elixir's
data model; a library needing mutation can receive a dict or a dedicated
Python-side class via interop instead.

## Open Questions

None for this revision.

## Conformance

Tests MUST cover: declaration forms (keyword list, atom list, mixed);
default factories for mutable defaults; construction with and without
omitted fields; same-module and cross-module construction and patterns;
frozen-ness; functional update; map update; attribute declaration order,
duplicate and undeclared diagnostics; attribute reads in function bodies
and in `@decorate` chains; and an end-to-end program exercising a
decorator held in a module attribute.

## Change History

- Revision 2, 2026-08-02: Examples updated to the GEP-0003 revision 2 `$` interop syntax.

- Revision 1, 2026-08-01: Initial version.
