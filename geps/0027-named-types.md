---
gep: 27
title: Named Types
description: "@type — named and generic type aliases with a declaration site, arity checking, and compile-time structural expansion; zero runtime."
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
created: 2026-08-05
updated: 2026-08-05
revision: 1
requires: [17]
replaces: []
superseded-by: null
resolution: null
translations:
  zh: local/zh/0027-named-types.md
---

# GEP-0027: Named Types

## Abstract

The type language had builtins, type variables, and classes — but no
way to *name* a type. `@type` adds named and generic aliases with a
real declaration site: parameters are declared in the head, references
are arity-checked, and everything expands structurally at compile time
into PEP 484 annotations. Zero runtime, as always.

```elixir
@type age() :: integer()
@type result(t) :: tuple(atom(), t)
@type scores() :: map(string(), age())

@spec parse(string()) :: result(integer())   # -> tuple[str, int]
@spec load(string()) :: Mod.result(string()) # cross-module
```

## Specification

**GEP-0027-R001 (declaration):** `@type name(params) :: type` declares
a named type at module top. The name is a plain lowercase word; it MUST
NOT shadow a built-in type, MUST NOT be `t` (retired by GEP-0017
rev 5), and MUST NOT be declared twice. A preceding `@doc` documents
the type; `@spec`/`@param` do not apply to it.

**GEP-0027-R002 (generics have a declaration site):** parameters are
type variables (1–2 lowercase letters) listed in the head. A bare
variable in the body that is not declared is a compile error. (Inside
`@spec`, variables remain implicitly scoped to the spec — the
lightweight tier is unchanged.)

**GEP-0027-R003 (reference and expansion):** a named type is referenced
as a call — `age()`, `result(integer())` in its own module,
`Mod.result(string())` from any project module. References are
arity-checked. Expansion is compile-time structural substitution into
the final annotation; recursive definitions are a compile error (cycle
detection with a depth cap). Referencing a type a module does not
declare is a compile error; unknown lowercase types get a did-you-mean
over builtins and the module's own `@type` names.

**GEP-0027-R004 (advice):** a `@spec` whose entire return is a type
variable appearing in no parameter gets a practice hint — a variable
used once constrains nothing.

## Deferred

Constrained generics (`a when a: number()`), generic structs, `@opaque`,
and cross-*package* named types (requires type metadata in wheels).
Macro-generated `@type` declarations are not collected in this
revision.

## Conformance

Tests MUST cover: alias and generic expansion (same and cross module,
nested aliases); arity, undeclared-parameter, duplicate, shadowing,
`t`, and cycle errors; the did-you-mean for near-miss names; and the
lone-return-variable hint.

## Change History

- Revision 1, 2026-08-05: Initial version.
