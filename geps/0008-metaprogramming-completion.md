---
gep: 8
title: Metaprogramming Completion
description: Definition-generating macros, use/__using__, unquote in definition heads, and a user-extensible attribute system with definition hooks.
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
  - Macros
created: 2026-08-01
updated: 2026-08-08
revision: 2
requires: [2, 7]
replaces: []
superseded-by: null
resolution: null
translations:
  zh: local/zh/0008-metaprogramming-completion.md
---

# GEP-0008: Metaprogramming Completion

## Abstract

This proposal completes the Elixir metaprogramming story
(quote-and-unquote, macros, and domain-specific-languages guides).
Macros may now generate definitions; `def unquote(name)(...)` works in
templates; `use Mod` invokes `Mod.__using__`. Built-in attributes stop
being special: `defattr` registers user attributes (accumulating or
not) and `@on_definition` names a hook macro that receives each
definition together with its collected attributes and may rewrite it —
the mechanism `@doc`/`@example`-class features are built from, now
available to users.

## Motivation

The v0 macro system (GEP-0002) deliberately forbade generating
definitions, which blocks the defining Elixir patterns: `deftest`-style
DSLs, `use`-injected APIs, and attribute-driven code generation
(routers, schemas). Meanwhile Gandora's own `@doc`/`@example` are
compiler-hardcoded; Elixir shows the better shape —
`Module.register_attribute/3` plus `@on_definition` — where the
language grows its own annotation systems in userland.

## Scope

Definition generation, `use`, unquote in heads, attribute
registration, and definition hooks. `@before_compile`/`@after_compile`
hooks are deferred. Cross-package hook distribution, deferred in
revision 1, turned out to need nothing beyond GEP-0006 macro shipping
— a hook is a macro — and is exercised since revision 2 (`gan-lsc`
consumes `Cli.on_command` from gandora-tool).

## Terminology

- **Declaration macro**: a macro invoked at module top level whose
  expansion produces definitions.
- **Registered attribute**: an attribute declared by `defattr`.
- **Definition hook**: the macro named by `@on_definition`, run for
  each subsequent definition.

## Specification

**GEP-0008-R001:** Macro expansion at module top level MAY produce
`def`, `defp`, `defstruct`, documentation attributes, `@decorate`, and
`__block__` sequences of these, which the compiler flattens into the
module body. Expansion MAY additionally produce writes of registered
attributes (rev 2): they are absorbed as registrations — joining the
R004 value table in expansion order, subject to the same accumulate
rules — and never reach codegen as plain module attributes. This is
the second half of R005's "definition plus registrations": without it
a hook could rewrite definitions but never build the table that makes
rewriting worth doing. Header forms (`defmodule`, `alias`, `import`,
`require`, `pyimport`, `use`) MUST NOT be macro-generated. This amends
GEP-0002-R007.

**GEP-0008-R002:** In a quote template, `def unquote(expr)(params)`
(and `defp`) defines a function whose name is the atom or string value
of `expr` at expansion time, enabling name-computed definitions. The
same coercion applies in capture position (rev 2): `&unquote(expr)/n`
captures the named local function, private-name mangling included — a
hook that received a definition's head can register a callable to it.

**GEP-0008-R003:** `use Mod` and `use Mod, opts` at module top level
are equivalent to `require Mod` followed by expanding
`Mod.__using__(opts)` (arity 0 or 1) in place. A `use` target without
`__using__` is a compile error naming the module.

**GEP-0008-R004:** `defattr :name` registers a module attribute for
the current module; `defattr :name, accumulate: true` makes repeated
`@name value` occurrences collect in source order instead of erroring.
Values are quoted terms. `@name` reads follow GEP-0004-R011; an
accumulated attribute reads as a list. Unregistered, non-built-in
attributes remain the GEP-0004 module-attribute bindings.

**GEP-0008-R005:** `@on_definition Mod.hook` (after `require Mod`)
registers a definition hook. For each subsequent `def`/`defp`, the
compiler expands `Mod.hook(kind, head, attrs, body)` where `kind` is
`:def`/`:defp`, `head` and `body` are the definition's quoted terms,
and `attrs` is a keyword list of the registered-attribute values
collected since the previous definition (which are then reset, like
`@doc`). The hook returns replacement top-level syntax (commonly the
reconstructed definition plus registrations), subject to R001. Hooks
run in the GEP-0002-R003 sandbox with its determinism and limits.

**GEP-0008-R006:** Built-in attributes (`@doc` family, `@example`,
`@decorate`, `@moduledoc` family) keep their GEP-0007/0003 semantics
and are not visible to hooks; a `defattr` name colliding with a
built-in is a compile error.

## Rationale

Definition hooks receiving (kind, head, attrs, body) are exactly how
Elixir libraries build annotation systems; passing collected attributes
into a rewriting macro subsumes decorator registries, route tables, and
doc-like channels without adding compiler features per use case. `use`
plus declaration macros then covers the DSL guide's `deftest` pattern
end to end. Header forms stay non-generatable so module identity and
the dependency graph remain statically known (GEP-0002-R006's
guarantee).

## Backwards Compatibility

Amends GEP-0002-R007 (relaxation only). Existing attribute semantics
(GEP-0004) are unchanged for unregistered names.

## Security and Determinism

Everything runs in the existing expansion sandbox; hooks add no new
capabilities, only new inputs. Expansion remains deterministic and
bounded.

## Tooling and AI Usage

Agents should reach for `use` + declaration macros for DSLs, `defattr`
+ `@on_definition` for annotation systems, and verify generated
definitions with `gan expand`.

## Rejected Alternatives

### Compiler-special decorators per feature

Each new annotation (routes, caching, tracing) would need compiler
work; the hook mechanism moves that to libraries, as Elixir does.

### Allowing macro-generated imports/defmodule

Would make the module graph depend on expansion results, breaking
static macro resolution and package discovery.

## Open Questions

None for this revision.

## Conformance

Tests MUST cover: a declaration macro generating multiple defs
(flattening included); `def unquote(name)(...)`; `use` with and
without opts and its missing-`__using__` diagnostic; accumulating and
non-accumulating `defattr` with reset-per-definition semantics; an
`@on_definition` hook that decorates and registers definitions; the
R006 collision diagnostic; and `gan expand` output for all of the
above.

## Change History

- Revision 2, 2026-08-08: R001 — expansion may write registered
  attributes, absorbed as registrations (the R005 "definition plus
  registrations" promise, previously rejected by the implementation as
  duplicate plain attributes); R002 — the atom/string coercion extends
  to capture position (`&unquote(name)/n`). Motivated by the toolchain
  going declarative: `gan`'s command table and `gan-mcp`'s tool table
  are built by `@on_definition` hooks from annotations on the
  definitions themselves, so the data and the execution cannot drift.

- Revision 1, 2026-08-01: Initial version.
