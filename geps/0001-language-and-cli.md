---
gep: 1
title: The Gandora Language and the gan CLI
description: Core identity of Gandora, the Elixir-flavored surface syntax, compilation to Python, module naming, project configuration, and the gan command line.
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
  - CLI
  - Configuration
created: 2026-08-01
updated: 2026-08-02
revision: 5
requires: [0]
replaces: []
superseded-by: null
resolution: null
translations:
  zh: local/zh/0001-language-and-cli.md
---

# GEP-0001: The Gandora Language and the gan CLI

## Abstract

Gandora is a programming language with Elixir-flavored syntax that compiles
to readable Python. This proposal defines the language identity, the source
file format, module naming and its mapping to generated Python, the project
configuration file `gandora.jsonc`, the integration with `uv`-managed Python
projects, and the `gan` command-line interface.

Gandora deliberately adopts Elixir's surface: `defmodule`/`def` with
`do ... end` blocks, atoms, pattern matching, the `|>` pipe, and
`defmacro`-based metaprogramming. Where Elixir reaches its host
platform through `:erlang` calls, Gandora reaches Python through the
`$` sigil: `$math`, `$json`, or any installed module is callable
without wrapper code (GEP-0003).

## Motivation

Python has the largest library ecosystem; Elixir has one of the most pleasant
surfaces for functional, pipeline-oriented programming and one of the best
macro systems. Gandora combines them: developers write Elixir-style modules
and macros, and deployment remains ordinary Python managed by `uv`, with the
generated code readable enough to review, debug, and profile with standard
Python tooling.

A precise founding proposal is needed so that the compiler, the package
integration, and future tooling share one contract, and so that AI agents can
implement against stable requirements rather than folklore.

## Scope

This proposal covers language identity, source files, the surface syntax
inventory, module naming and the Python name mapping, project configuration,
`uv` integration, and the `gan` CLI. The macro system is specified by
GEP-0002 and the Python interop contract by GEP-0003. The standard library,
formatter, LSP, and package publication format are out of scope and require
future GEPs.

## Terminology

- **Source file**: a UTF-8 text file with the `.gan` extension.
- **Module**: the unit of namespacing declared by `defmodule`.
- **Project**: a directory tree governed by one `gandora.jsonc` and one
  `pyproject.toml`.
- **Generated module**: the Python file produced for one Gandora module.
- **Remote atom call**: the `$module.function(args)` form that calls into
  Python (GEP-0003).
- **Entry function**: the zero-argument or one-argument `main` function used
  by `gan run`.

## Specification

### Identity

**GEP-0001-R001:** The language MUST be named **Gandora**. The compiler
executable MUST be named **`gan`**. Source files MUST use the extension
**`.gan`**.

**GEP-0001-R002:** The compiler MUST be implemented as a native executable
whose only executable build artifact is Python source code. Generated Python
MUST NOT require the Gandora compiler, a Gandora runtime package, or any
import hook at execution time beyond the project's declared Python
dependencies.

**GEP-0001-R003:** The Python distribution of the compiler MUST be named
`gandora-lang` and MUST install the native `gan` executable. Python
dependency management, virtual environments, and publication remain owned by
standard Python tooling; the default documented workflow uses `uv` and the
standard `.venv` layout.

**GEP-0001-R004:** One compiler invocation MUST target exactly one Python
version, configured as `targetPython` and defaulting to `3.12`. Generated
code MAY use any syntax available in the target version, including `match`
statements.

### Surface syntax

**GEP-0001-R005:** Gandora surface syntax MUST follow Elixir's surface
grammar for the constructs the language supports, so that Elixir syntax
highlighting and editor affordances remain usable. Divergences from Elixir
MUST be recorded in a GEP.

**GEP-0001-R006:** The v0 surface MUST include:

- `defmodule Name do ... end` with dot-separated CamelCase module names;
- `def`, `defp` (private), with `do ... end` bodies and the keyword shorthand
  `def f(x), do: expr`;
- `defmacro` and `quote`/`unquote`/`unquote_splicing` (GEP-0002);
- module attributes `@moduledoc` and `@doc` with string values;
- integers (including `_` separators), floats, booleans, `nil`, strings with
  `#{...}` interpolation, atoms (`:ok`, `:"quoted"`), lists, tuples
  (`{a, b}`), maps (`%{"k" => v, a: 1}`), keyword lists (`[a: 1, b: 2]`),
  ranges (`a..b`);
- operators `+ - * / == != < > <= >= and or not ++ <> |> = // rem div`
  with Elixir precedence, where `//` and `div`/`rem` follow Elixir's
  integer-division semantics;
- `if/else`, `unless`, `case`, `cond`, `fn args -> body end` anonymous
  functions and `&Mod.fun/1` / `&(&1 + 1)` captures, `with` for chained
  matches;
- pattern matching with `=`, in `case` clauses, and in function heads,
  including literal, variable, `_`, tuple, list (`[h | t]`), map, and pin
  (`^x`) patterns;
- multi-clause function definitions dispatched top-to-bottom by pattern and
  optional `when` guards;
- `alias`, `import`, and `require` with `as:`, `only:`, and `except:`
  options following Elixir semantics;
- remote `$` references and `expr.name`/`expr.name(args)` postfix
  access (GEP-0003).

**GEP-0001-R007:** A construct outside the v0 surface MUST produce a
diagnostic naming the unsupported construct; the compiler MUST NOT silently
mistranslate Elixir syntax it does not implement. Notable v0 exclusions:
protocols, behaviours, `receive` and every process primitive,
`try/rescue`, binaries/bitstrings (`<<>>`), and comprehensions (`for`).
Structs are specified by GEP-0004 and sigils by GEP-0005.

**GEP-0001-R008:** Comments use `#` to end of line. Source MUST be UTF-8 and
identifiers MUST follow Elixir rules: functions and variables are
`snake_case` (Unicode letters permitted) optionally ending in `?` or `!`;
modules are CamelCase segments joined by dots.

### Evaluation semantics

**GEP-0001-R009:** Gandora data types MUST map onto Python values so that no
wrapper types cross the interop boundary:

| Gandora | Python |
| --- | --- |
| integer, float, boolean, string | `int`, `float`, `bool`, `str` |
| `nil` | `None` |
| atom `:name` | interned `str` `"name"` |
| list | `list` |
| tuple | `tuple` |
| map | `dict` |
| keyword list | `list` of 2-`tuple`s |
| range `a..b` | inclusive `range(a, b + 1)` |
| anonymous function | Python callable |

**GEP-0001-R010:** Atoms compile to Python string literals. `true`, `false`,
and `nil` are the Python singletons, not atoms. Equality `==` is Python
equality. Truthiness follows Elixir: only `false` and `nil` are falsy, so
conditional and boolean forms MUST compile to explicit
`is not falsy` checks rather than Python truthiness.

**GEP-0001-R011:** Variables are immutable bindings; rebinding a name in a
sequential body creates a new binding as in Elixir. A `case`, `if`, or `cond`
expression yields the value of its taken branch. Every function returns the
value of its last expression; the compiler inserts explicit `return`
statements.

**GEP-0001-R012:** A failed pattern match MUST raise a Python exception whose
type name and message identify it as a Gandora match error.

### Modules and generated Python

**GEP-0001-R013:** Each source file MUST contain exactly one `defmodule`, and
its name MUST equal the CamelCase rendering of the file's path relative to a
configured source root: path separators become dots and each `snake_case`
segment becomes one CamelCase module segment. Example: `src/app/hello_web.gan`
under source root `src` must declare `defmodule App.HelloWeb`.

**GEP-0001-R014:** A Gandora module `App.HelloWeb` MUST compile to the Python
module `app/hello_web.py` under the output directory, with each `def` as a
top-level Python function. `defp` functions compile to functions whose names
carry a single leading underscore.

**GEP-0001-R015:** The Gandora-to-Python identifier mapping MUST be injective
and is a public compatibility contract: `?` maps to a trailing `_p`, `!`
maps to a trailing `_bang`, and any other character invalid in a Python
identifier maps to `_u<hex>_` of its code point. Unicode letters valid in
Python identifiers are preserved in NFC form. Collisions after mapping MUST
be a compile error.

**GEP-0001-R016:** `@moduledoc` and `@doc` string values MUST become Python
docstrings on the generated module and functions.

**GEP-0001-R017:** Cross-module references (`alias`, `import`, and fully
qualified `App.Mod.fun(...)` calls) MUST compile to ordinary Python imports
of the corresponding generated modules. Import cycles between Gandora
modules MUST be a compile error.

### Project configuration

**GEP-0001-R018:** A project MUST be configured by the nearest ancestor
`gandora.jsonc`. The file is JSON with comments and trailing commas
permitted; duplicate keys MUST be rejected and unknown top-level fields MUST
produce a diagnostic.

**GEP-0001-R019:** `gandora.jsonc` recognizes exactly these fields:

- `source`: ordered array of project-relative source roots, default
  `["src"]`;
- `outDir`: project-relative output directory, default `"dist"`, always
  excluded from source discovery;
- `targetPython`: Python version string, default `"3.12"`;
- `exclude`: array of project-relative glob rules removed from discovery.

`pyproject.toml` continues to own package metadata and dependencies; the two
files MUST NOT duplicate each other's concerns.

**GEP-0001-R020:** `gan run` MUST compile into an internal cache directory
`.gandora/cache/` rather than `outDir`, and that cache MUST be safe to
delete at any time and MUST NOT be published.

### The gan CLI

**GEP-0001-R021:** The v0 CLI MUST provide:

- `gan init [path]`: create a new `uv`-style project with `gandora.jsonc`,
  `pyproject.toml`, `.python-version`, `.gitignore`, and `src/main.gan`;
  `gan init --existing [path]` adds Gandora files to an existing project
  without overwriting anything that exists;
- `gan check [file...]`: parse, expand, and analyze without writing output;
- `gan build`: compile every discovered source file into `outDir`;
- `gan compile <file> [--out <dir>]`: compile explicit files;
- `gan run <file> [args...]`: compile the file's project, then execute the
  generated module for `<file>` with the project's Python interpreter,
  preferring `.venv/bin/python` and falling back to `uv run python`, passing
  `args` through as `sys.argv[1:]`;
- `gan expand <file>`: print the source after macro expansion, in surface
  syntax;
- `gan --version` / `gan -V`: print `gan <semver>`.

**GEP-0001-R022:** If the target module of `gan run` defines `main/0`, the
generated module MUST call it under `if __name__ == "__main__":`. Compiling
a module MUST NOT execute user code beyond Python's module-level definition
machinery.

**GEP-0001-R023:** Exit codes: 0 for success, 1 for compilation or runtime
failure, 2 for command-line misuse. Diagnostics MUST name the file, line,
and column of the primary span and MUST be written to stderr.

**GEP-0001-R025:** When the right-hand side of `|>` begins with `.`, the
pipe is a method pipe: `x |> .name(args)` evaluates to the postfix call
`x.name(args)` on the piped value (GEP-0003-R004), and further postfix
segments may follow. This composes Python fluent APIs (pandas, numpy)
with Elixir pipelines without intermediate bindings.

**GEP-0001-R026:** Triple-quoted strings (`"""`) are heredocs with
Elixir's semantics: the newline after the opening delimiter is not part
of the value, and the whitespace indentation of the line holding the
closing delimiter is stripped from the start of every content line.
A heredoc whose content begins on the opening line, or whose closing
delimiter does not sit alone on its line, is taken verbatim. This is
what makes indented heredocs — documentation, usage text, scaffolding
templates — read cleanly in source while producing flush-left values.

**GEP-0001-R024:** Generated Python MUST be deterministic: compiling the
same sources with the same compiler version and configuration produces
byte-identical output.

## Rationale

Compiling to readable Python rather than bytecode or an interpreter keeps
the entire Python ecosystem — debuggers, profilers, `uv`, deployment
targets — usable without Gandora-specific support, readable output plus `.venv` compatibility is the
cheapest possible integration story.

The one-module-per-file rule (unlike Elixir, which allows several) buys a
direct, predictable mapping from module names to generated Python modules,
which in turn keeps imports, tooling, and incremental compilation simple.
Path-derived module names make the mapping mechanical.

Atoms as interned strings, rather than a dedicated atom class, keep the
interop boundary wrapper-free: `:ok` compares equal to the `"ok"` a Python
library returns, and pattern matching on Python data works unchanged.

Elixir truthiness is preserved (only `false`/`nil` are falsy) because
silently adopting Python truthiness would change the meaning of ordinary
Elixir-style code such as `if list do ... end`.

## Backwards Compatibility

This is the founding language proposal; there is no earlier contract to
preserve. The identifier mapping (R015), the data-type mapping (R009), and
the module-to-path rule (R013–R014) are the compatibility surfaces future
GEPs must respect.

## Security and Determinism

The compiler MUST NOT execute user Python code, import Python packages, or
access the network during compilation. Macro expansion is a sandboxed,
deterministic compile-time evaluation defined by GEP-0002. Deterministic
output (R024) makes builds reproducible and diffable.

## Tooling and AI Usage

AI agents writing Gandora should read this GEP for the surface inventory
(R006) and its exclusions (R007) before generating code, use `gan check`
for validation, and `gan expand` to inspect macro output. Tooling may rely
on the injective name mapping (R015) to correlate Gandora names with
generated Python names in both directions.

## Rejected Alternatives

### Compile each module to a Python class namespace

Allowing several modules per file and generating one class per module keeps
Elixir's file freedom, but the generated code is less idiomatic, every call
site pays a class-attribute indirection, and module/file tooling (imports,
coverage, stack traces) degrades. One module per file was chosen instead.

### A dedicated Atom runtime class

A real atom type would make `:ok` distinguishable from `"ok"`, closer to
Elixir semantics, but every interop boundary would need conversion and
pattern matching on Python data would break. Interned strings follow the
data-first philosophy.

### Implementing the CLI in Python

A Python CLI would remove the Rust toolchain from contributors' setup, but
compile speed, single-binary distribution, and independence from the
target's virtual environment outweigh that convenience.

## Open Questions

None for v0; excluded constructs (R007) are intentionally deferred to future
GEPs rather than left ambiguous.

## Conformance

An implementation conforms when: `gan init && gan build && gan run` work on
a fresh project; every R006 construct compiles and runs with the specified
semantics; every R007 exclusion produces a named diagnostic; generated
output satisfies the mappings of R009, R013–R017; and the CLI satisfies
R021–R024. The repository's test suite maps test names to these requirement
identifiers.

## Change History

- Revision 5, 2026-08-02: Interop references follow GEP-0003 revision 2
  (`$module` instead of atom calls) throughout.

- Revision 4, 2026-08-02: Added R026, Elixir heredoc dedent semantics.

- Revision 3, 2026-08-01: Added R025, the `|> .method(args)` method pipe.

- Revision 2, 2026-08-01: R007 exclusion list updated — structs and sigils
  are now specified by GEP-0004 and GEP-0005.
- Revision 1, 2026-08-01: Initial version. Bootstrap acceptance recorded by
  the repository's initial design commit rather than an external resolution
  URL.
