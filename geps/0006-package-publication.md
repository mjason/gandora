---
gep: 6
title: Package Publication
description: Publishing Gandora packages as ordinary PyPI wheels — compiled self-contained Python plus shipped sources for compile-time macros — with no runtime dependency.
author: MJ
status: Accepted
type: Standards Track
areas:
  - Packages
  - Interop
created: 2026-08-01
updated: 2026-08-01
revision: 1
requires: [1, 2]
replaces: []
superseded-by: null
resolution: null
translations:
  zh: local/zh/0006-package-publication.md
---

# GEP-0006: Package Publication

## Abstract

A Gandora package is an ordinary Python wheel. It contains the compiled,
self-contained Python modules (usable by any Python consumer), a static
`gandora.toml` marker, and the original `.gan` sources. Runtime calls into
a published package are plain Python imports — no Gandora runtime package
exists, mirroring GEP-0001-R002. Macros are the one compile-time surface:
the consumer's compiler discovers installed packages by reading markers
from the virtual environment (never importing Python) and expands their
macros from the shipped sources during the consumer's own compilation.

## Motivation

Distribution must not fork the ecosystem: a data-science team should be
able to `uv add` a Gandora-authored package without knowing Gandora
exists, and a Gandora consumer should get the full surface — functions
*and* macros — from the same artifact. The core judgment: `pyproject.toml`/`uv` own dependency management
entirely, and the compiler reads only static files from the locked
environment.

## Scope

Package project layout, the wheel contents contract, the marker format,
consumer-side resolution, and the no-runtime guarantee. A dedicated PEP
517 build backend, cross-package `.gani`-style compiled interfaces,
version-conflict diagnostics, and transitive package macro dependencies
are deferred.

## Terminology

- **Package project**: a project whose `gandora.jsonc` sets
  `"package": true`.
- **Marker**: the `gandora.toml` file identifying a directory inside a
  wheel as Gandora-built.
- **Shipped sources**: the `.gan` files copied into the wheel under
  `_gan/`.

## Specification

### Package projects and wheel contents

**GEP-0006-R001:** `gandora.jsonc` accepts an optional boolean field
`package` (default `false`), amending the field set of GEP-0001-R019.
`gan init --package <name>` MUST scaffold a package project whose
`pyproject.toml` uses a standard Python build backend configured to
package the compile output directory, so that `uv build` produces the
wheel with no Gandora-specific build tooling.

**GEP-0006-R002:** In a package project, `gan build` MUST additionally
write, for each top-level output package directory:

- the marker `<package>/gandora.toml`;
- each module's source at `<package>/_gan/<python-path>.gan`, where
  `<python-path>` is the module's GEP-0001-R014 path.

**GEP-0006-R003:** The wheel therefore contains, and MUST NOT require
more than: compiled `.py` modules (each self-contained per
GEP-0001-R002), the marker, and the shipped sources. Macro-only modules
contribute sources and marker entries but no `.py` (GEP-0002-R009).

**GEP-0006-R004:** The marker is TOML with this schema (version 1):

```toml
schema = 1
compiler = "<gan version>"

[[modules]]
name = "AcmeText.Slug"
python = "acme_text/slug.py"
source = "acme_text/_gan/acme_text/slug.gan"
```

Paths are relative to the site-packages root. A macro-only module omits
`python`. Consumers MUST reject markers whose `schema` they do not know.

### Consumer resolution

**GEP-0006-R005:** Runtime references into a published package (`alias`,
qualified calls) compile to plain Python imports exactly as for local
modules (GEP-0001-R017); the compiler performs no discovery, and the
import resolves at runtime against the installed wheel.

**GEP-0006-R006:** When `require` or `import` names a module not found in
the project's sources, the compiler MUST search the project environment's
site-packages directories for markers, and on a name match parse the
shipped source to collect its macros (GEP-0002-R006 visibility rules
apply). The search MUST read only static files; it MUST NOT import or
execute Python.

**GEP-0006-R007:** Package macros MUST be self-contained in v0: a shipped
macro module may not itself `require` further modules. Expansion happens
in the consumer's compilation with the consumer's hygiene contexts and
limits; identical package versions MUST yield identical expansions.

**GEP-0006-R008:** A `require`d module found neither locally nor in an
installed marker MUST produce a diagnostic naming the module and the
searched environment.

### The no-runtime guarantee

**GEP-0006-R009:** Publishing and consuming packages MUST NOT introduce
any runtime dependency on Gandora: no shared support package, no import
hook, no loader. Per-module semantic helpers remain inlined in each
generated file (GEP-0001-R002). A Python-only consumer MAY use the wheel
with no knowledge of Gandora; deleting every `.gan` file and marker from
an installed wheel MUST NOT change its runtime behavior.

## Rationale

Shipping compiled Python instead of compiling on install keeps `uv add`
instant, keeps the package usable from plain Python, and removes any
version coupling between the consumer's compiler and the package author's
— the wheel behaves identically everywhere because it *is* its behavior.

Macros cannot be runtime artifacts (they do not exist at runtime,
GEP-0002-R009), so the only faithful distribution is their source.
Shipping `.gan` source trades a little parse time for zero new formats,
and leaves room for a compiled interface format in a future GEP.

Reading markers from site-packages keeps a hard rule: discovery never
executes package code — installation state is data.

## Backwards Compatibility

Additive. Non-package projects are unaffected. The marker schema is
versioned for future evolution; `_gan/` is reserved inside package
output.

## Security and Determinism

Marker scanning reads static files from the project's own environment and
never executes code. A hostile wheel can therefore contribute at most
macro *source*, which expands in the deterministic sandbox of
GEP-0002-R003 — it cannot perform I/O at compile time. Runtime behavior
of an installed wheel is ordinary Python, reviewable in the wheel itself.

## Tooling and AI Usage

Agents publishing a package: `gan init --package`, write modules,
`gan build`, `uv build`, `uv publish`. Agents consuming one: `uv add`,
then `alias`/`require` as if it were local. Agents should not vendor or
wrap package code — the wheel is already the interface.

## Rejected Alternatives

### Compile on install (ship only .gan, build via PEP 517 hook)

Would guarantee source/binary consistency but makes installs require the
compiler, breaks Python-only consumers, and couples every consumer to a
compiler version. Rejected as the default; a source-build backend can
still be added later for native-extension-style needs.

### A shared runtime wheel for helpers

Deduplicates a few dozen inlined lines at the cost of the entire
no-runtime property and a version-skew surface between packages compiled
by different compilers. Rejected; inlining is the mechanism that makes
R009 possible.

### Serialized macro IR instead of shipped sources

Faster consumer compiles and hides source, but requires a stable IR
format now. Deferred until a compiled-interface GEP; the marker's
versioned schema keeps the door open.

## Open Questions

None for this revision.

## Conformance

Tests MUST cover: package scaffold shape; marker and shipped-source
emission (including a macro-only module); consumer macro resolution from
a site-packages marker without any local copy; the R008 diagnostic; and
an end-to-end consumer run whose generated output imports the installed
package and contains no Gandora import.

## Change History

- Revision 1, 2026-08-01: Initial version.
