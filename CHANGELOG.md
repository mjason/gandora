# Changelog

## v0.1.0 — 2026-08-02

The founding release: a complete, tested v0 of the Gandora language.

- **Language** (GEP-0001): Elixir surface — modules, multi-clause
  `def` with guards, pattern matching (`=`, `case`, `cond`, `with`),
  pipes incl. the `|> .method()` form, `fn`/captures, string
  interpolation and heredocs, Elixir truthiness and truncated
  division; compiles to readable, deterministic, self-contained
  Python (zero runtime).
- **Macros & metaprogramming** (GEP-0002/0008): hygienic `defmacro`
  with `quote`/`unquote`/`unquote_splicing`/`var!`, deterministic
  sandboxed expansion, definition-generating macros,
  `def unquote(name)`, `use`/`__using__`, and the
  `defattr`/`@on_definition` annotation system.
- **Interop** (GEP-0003): `:module` atom calls, `pyimport`, postfix
  chains, kwargs, `@decorate` — wrapper-free Python access.
- **Data** (GEP-0004): `defstruct` → frozen dataclasses with literals,
  patterns, and functional update; module attributes as import-time
  bindings.
- **Sigils & templates** (GEP-0005/0009): `~w`/`~s`/`~r`, the
  `~<lang>` embedded-language family, and EEx-style `<%= %>` value and
  code splices.
- **Packages** (GEP-0006): ordinary PyPI/git wheels carrying compiled
  Python, a `gandora.toml` marker, and shipped sources for macros;
  marker-based runtime resolution.
- **Documentation** (GEP-0007): Markdown `@doc` with metadata,
  prose-only `@doc_trans` translations, `@example` doctests compiled
  to native Python doctests, `gan doc`/`gan test`, embedded bilingual
  builtin docs.
- **Standard library** (GEP-0010, [std/](std/)): `Enum`, `String`,
  `Map`, `List`, `Keyword` — 77 data-first functions written in
  Gandora, bilingual and doctested.
- **Multi-arity** (GEP-0011): one name across arities with `\\`
  default parameters.
- **Tooling**: `gan init`/`check`/`build`/`run`/`compile`/`expand`/
  `doc`/`test`, `gandora.jsonc`, uv/.venv integration.

All specifications ship with synchronized Chinese translations
(`geps/local/zh/`, `docs/local/zh/`).
