---
gep: 13
title: The gan Task Runner
description: The developer entry point `gan` becomes a Gandora program — the mix/cargo role — over gandora-core, with subcommand plugins and the Rust compiler demoted to the stage-0 tool ganc.
author: MJ
status: Accepted
type: Standards Track
areas:
  - Tooling
created: 2026-08-02
updated: 2026-08-02
revision: 2
requires: [6, 12, 14]
replaces: []
superseded-by: null
resolution: null
translations:
  zh: local/zh/0013-the-gan-task-runner.md
---

# GEP-0013: The gan Task Runner

## Abstract

What developers type is not a compiler; it is a task runner — `mix`
for Elixir, `cargo` for Rust. Gandora's is called `gan`, and it is a
Gandora program: a package (`gandora-tool` on PyPI, source in
`tools/gan/`) whose entry point drives `gandora-core` in-process for
building, running, and evaluating code. The Rust binary is renamed
`ganc` — the stage-0 compiler that bootstraps the toolchain and backs
commands not yet rewritten. Unknown subcommands delegate to installed
`gan-<name>` executables, the cargo plugin convention, so the tool
ecosystem grows in Gandora without touching either core artifact.

## Motivation

GEP-0012 made the compiler a library precisely so the toolchain could
be written in the language. Doing so is the strongest completeness
proof available — the runner exercises interop, control flow
(GEP-0014), the stdlib, and the package system on every invocation —
and gives every Gandora user a daily, visible example of real Gandora
code. The `gan`/`ganc` split mirrors `cargo`/`rustc`: muscle memory
and documentation keep working while the entry point changes
implementation language.

## Scope

The runner's command set, its delegation rules, the `ganc` rename,
and distribution. The REPL's line-editing niceties, `watch`, `fmt`,
and the LSP server (a plugin, GEP-0015) are out of scope.

## Terminology

- **Runner**: the `gan` entry point, a Gandora program.
- **Stage-0 compiler**: `ganc`, the Rust binary, used to build the
  runner itself and as the delegate for legacy commands.
- **Plugin**: an executable `gan-<name>` reachable in the project
  environment.

## Specification

**GEP-0013-R001:** The Rust binary is named `ganc`; the `gandora-lang`
wheel ships it. The user-facing `gan` command is provided by the
`gandora-tool` distribution: a Gandora package (source `tools/gan/`,
compiled Python in the wheel per GEP-0006) whose console entry point
is the runner. Both release in lockstep with the toolchain version.

**GEP-0013-R002:** The runner implements natively (via
`gandora_core`): `version` (reporting runner and core versions, with
the GEP-0012-R007 mismatch warning), `build`, `check`, `run`, `exec
<code>` (compile-snippet, execute, print `_` as by `inspect/1`), and
`repl` (a GEP-0014 `loop` over stdin using the same snippet
machinery, `gan>` prompt, bindings persisting across lines). `init`
scaffolds as GEP-0001-R021 describes.

**GEP-0013-R003:** A subcommand the runner does not implement is
resolved in order: an executable `gan-<name>` on the environment's
paths (plugin, receiving the remaining arguments); otherwise
delegation to `ganc <name> ...` (covering `expand`, `doc`, `test`,
`compile`, `lsc` and future stage-0 commands); otherwise the usage
error. Plugins are ordinary distributions exposing a `gan-<name>`
entry point — installing one is `uv add`.

**GEP-0013-R006:** A project needs exactly two Gandora entries in its
`pyproject.toml`: `gandora-std` as a runtime dependency (generated
code imports it) and `gandora-tool[dev]` in the `dev` dependency
group — the `dev` extra aggregates the toolchain (the language
server, with the compiler library arriving transitively). Scaffolds
(`gan init`, `ganc init`) MUST emit exactly this shape.

**GEP-0013-R004:** Runner output and exit codes follow GEP-0001-R023.
The runner MUST NOT depend on the `gan` rust binary existing at
runtime except through R003 delegation, and MUST work in any project
where `gandora-core` and `gandora-tool` are installed.

**GEP-0013-R005:** Bootstrap: the runner's wheel is built by compiling
`tools/gan/` with `ganc` in CI (the GEP-0006 pipeline). The runner
never compiles itself at install time.

## Rationale

Delegation-with-fallback lets the rewrite proceed command by command
with no flag day: everything `ganc` does remains reachable through
`gan` from the first release of the runner. The plugin convention is
cargo's, chosen over a registry or config file because the
environment already is the registry — `uv add` installs a plugin, and
discovery is a PATH lookup.

Making `exec`/`repl` native first (rather than build/run) would have
been backwards: they showcase the language, but `build` is the
command that proves the runner can drive the compiler library
end-to-end, so both ship together.

## Backwards Compatibility

`gan` keeps its CLI surface (R003 guarantees it); scripts calling the
binary by its old identity should switch to `ganc` or keep working
through the runner. Documentation moves to `uv tool install
gandora-tool` (which depends on `gandora-core`) as the entry
installation.

## Security and Determinism

The runner executes only what its commands always executed (the
user's own project code); delegation runs executables from the
project environment, the same trust boundary as any Python
console-script.

## Tooling and AI Usage

AI agents keep using `gan <command>`; nothing changes at the surface.
Agents building Gandora tooling should ship it as a `gan-<name>`
plugin package rather than wrapping the runner.

## Rejected Alternatives

### Keeping the runner in Rust

Abandons the completeness proof and the ecosystem's ability to read
its own toolchain; the library boundary (GEP-0012) exists to enable
exactly this move.

### A new name for the entry point

Breaking `gan` muscle memory and every document for naming purity;
the cargo/rustc precedent shows the split works with the familiar
name on the runner.

## Open Questions

None for this revision.

## Conformance

Tests MUST cover: every R002 command against a real project
(including repl fed by pipe and exec printing `_`); plugin resolution
before ganc fallback and the usage error after both; the version
mismatch warning; and a wheel-installed runner driving a project with
no Rust toolchain present.

## Change History

- Revision 2, 2026-08-02: Added R006 — the `gandora-tool[dev]` extra
  as the single dev-dependency entry; scaffolds emit it.

- Revision 1, 2026-08-02: Initial version.
