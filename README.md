# Gandora

Gandora is the project for `gan`, an Elixir-flavored language that compiles to
readable Python. The language pairs Elixir-style modules, pattern matching,
pipelines, and hygienic `defmacro` macros with the Python ecosystem: packages
are managed by `uv`, virtual environments follow the standard `.venv` layout,
and any Python module is reachable without wrapper code through the `:module`
interop form (the same spirit as Elixir's `:erlang` atom calls).

Design decisions are recorded as Gandora Enhancement Proposals (GEPs) in
[`geps/`](geps/), following the process defined by GEP-0000. English GEPs are
normative; synchronized Chinese translations live under `geps/local/zh/`.

## Requirements

- Rust 1.85 or newer
- Python 3.11 or newer
- `uv` for Python development

## Status

Early development. The compiler and CLI are being built; see `geps/` for the
specification work that precedes each implementation step.
