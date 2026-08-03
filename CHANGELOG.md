# Changelog

## v0.9.2 — 2026-08-03

**Stack recursion now warns — with a way to say "I meant it."**

- **GEP-0019 rev 4 (R007)**: a self-recursive function that is never
  in tail position gets a compile warning pinned to its definition —
  in `gan check`, `gan build`, and as an editor Warning squiggle. It
  will grow the Python call stack (~1000 frames); the fix is the
  accumulator form, or `recur` if you want the guarantee checked.
- **`@allow :stack_recursion`**: structural recursion (tree walks,
  nested flattening) is depth-bounded and legitimate — acknowledge it
  at the definition site and the warning goes away, leaving the intent
  documented. Typos in the allow target are compile errors.
- Warnings are now spanned diagnostics everywhere (the stdlib-shadow
  notice included), so editor squiggles land on the offending line
  instead of the top of the file.

## v0.9.1 — 2026-08-03

**Closures now capture by value, and you can see how recursion compiled.**

- **GEP-0021**: closures snapshot the creation-time values of captured
  locals — `lambda x, *, n=n: x + n`, Python's own idiom, keyword-only
  so arity stays strict. Fixes a real semantic divergence: rebinding a
  variable (or a TCO loop rebinding parameters) no longer leaks into
  closures created earlier. Applies to `fn`, `&`, comprehension
  bodies, and nested closures alike.
- **GEP-0019 rev 3 (R006)**: the compiled shape of every recursive
  function is observable — `gan doc` prints it, LSP hover shows a
  badge (`while` loop vs native call stack), `gan lsc doc` returns a
  `tco` field. An optimization you can't see is one you can't trust.
- `gan doc` now shares the one documentation walk with the LSP
  (`when`-guarded first clauses no longer hide their docs); dead
  `loop` codegen removed.

## v0.9.0 — 2026-08-03

**`for` comprehensions land, `loop` retires.**

- **GEP-0020**: Elixir's `for` — generators, filters, multiple
  generators, pattern generators that *skip* non-matching elements,
  and `into: %{}` — compiles to native Python comprehensions:
  `for {k, v} <- xs, into: %{}, do: {k, v * 10}` is a dict
  comprehension, byte-for-byte what a reviewer would write.
- **GEP-0014 rev 3**: `loop` and `break` are retired. With GEP-0019
  recursion (implicit TCO + checked `recur`) and `for`, the
  construct's reason to exist is gone — and Elixir never had it. The
  compile error carries the migration recipe; the whole toolchain
  (fmt, runner repl, LSP scanners, playground) is itself migrated to
  recursive helpers, dogfooding the answer.

## v0.8.0 — 2026-08-03

**Tail-call optimization (GEP-0019).** Tail-position self-calls
compile to parameter rebinding in a loop — Elixir's natural recursive
style (`def sum_to(n, acc), do: sum_to(n - 1, acc + n)`) runs in
constant stack; a million frames pass in the test suite. Non-tail
recursion is untouched. `recur(args)` in a function body is the
explicit, compiler-checked spelling: tail position and arity are
verified at compile time. Default-parameter delegates now rebind
instead of calling.

**Local-variable type inference in the LSP (GEP-0015).** Hovering a
variable with no doc entity infers its type from the compiled Python
of the very buffer (jedi over `compile_string` output):
`log`: `list`, `result`: `str` — marked *(inferred)*.

## v0.7.1 — 2026-08-03

- **`$(a.b)` replaces `$"a.b"`** (GEP-0003 rev 4): the explicit module
  boundary now uses the sigil family's parenthesized delimiter instead
  of string quotes — `$(PIL.Image).open(f)`, `$(importlib.metadata)`,
  `$(os.path).sep`. Segments after `)` are always attributes; the
  boundary a quote used to blur is locked by construction. The old
  quoted spelling is a compile error naming the rule. Also fixed: a
  quoted/bounded boundary is no longer extended by the chain
  heuristic (`$(a.b).c` imports `a.b`, never `a.b.c`).

## v0.7.0 — 2026-08-02

**The type language, tidied (GEP-0017 rev 3, GEP-0003 rev 3).**

- **Type variables**: `@spec map(list(a), fun()) :: list(b)` — short
  lowercase names are generics, compiled to module-level
  `typing.TypeVar`s (3.11-compatible). Longer bare names error with a
  `did you mean name()?` hint.
- **Abstract containers built in**: `iterable(t)`, `sequence(t)`,
  `mapping(k, v)`, plus `keyword()` and `term()` — no more
  `$"collections.abc"` in idiomatic specs; prefer them in parameter
  positions (covariant).
- **Dotted `$` chains need no quoting**: the lowercase prefix is the
  module path, imported whole — `$importlib.metadata.version(x)`,
  `$collections.abc.Sequence`, `$os.path.join(a, b)` all just work;
  `$"..."` remains only as an explicit override.
- **std is generic**: 60+ signatures now carry type variables
  (`Enum.reduce(list(a), b, fun()) :: b`,
  `Map.get(map(k, v), k, d) :: v | d` — the default's type joins the
  return). Comparable-bounded positions (`sort`, `min`, `max`) stay
  grounded until bounded variables land. The typed surface still
  passes pyright with 0 errors.

## v0.6.2 — 2026-08-02

- **Parametrized host types in specs** (GEP-0017 rev 2):
  `$mod.Type(t, ...)` compiles to the subscripted hint
  `mod.Type[t, ...]` — `$"collections.abc".Sequence(number())`
  becomes `collections.abc.Sequence[int | float]`. Covariant
  containers for parameter positions, straight from the host.

## v0.6.1 — 2026-08-02

- **201 `@param` docs across the ecosystem**: every std parameter
  (Enum/Map/String/List/Keyword, bilingual en + zh-CN), the fmt and
  Python-intelligence tool surfaces, and the playground showcase.
  Hover, `gan doc` (localized headings), docstrings, signature help,
  and `gan lsc doc` all serve them.

## v0.6.0 — 2026-08-02

**Parameter documentation (GEP-0018).** `@param name, "text"`
documents one parameter of the next definition, validated against the
clause heads at compile time — renamed parameters break the build
instead of orphaning their docs. `@param_trans` carries translations
(GEP-0007 locale rules). Renderers generate the Elixir-style
`## Parameters` section (localized headings in `gan doc`), the
generated docstring carries it, hover shows it, and signature help
attaches each parameter's own description to its
`ParameterInformation` — the active argument's doc follows the
cursor. `@spec` heads may name arguments Elixir-style
(`name :: type`), shown in rendered specs. Hand-written
`## Parameters` sections remain valid and unparsed.

## v0.5.2 — 2026-08-02

- **The standard library is fully spec'd**: all 94 public functions
  across Enum/Map/String/List/Keyword carry `@spec` (GEP-0017), so
  every std call has typed hints, hover signatures, and signature
  help — `String.match?` takes `$re.Pattern`, the host type at the
  boundary. pyright over the generated std: 0 errors.
- Toolchain, tour, and playground modules annotated where their
  types are crisp; the typed surface (std + fully-annotated modules)
  passes pyright clean.
- Codegen: guard built-ins (`is_list`, `is_nil`, ...) are
  boolean-shaped — no more `_gan_truthy` wrapping around
  `isinstance`, which both reads better and lets type checkers
  narrow.

## v0.5.1 — 2026-08-02

- **gan-lsc mirrors the full LSP capability set** (GEP-0015 rev 6):
  `doc`, `definition`, `symbols`, plus the Python-side quartet
  `pydoc` / `pycomplete` / `pygoto` / `pysig` — every editor
  capability now has an AI-facing one-JSON-value query.

## v0.5.0 — 2026-08-02

**Typespecs (GEP-0017).** `@spec mean(list(number()), integer()) ::
float()` compiles to PEP 484 hints on the generated Python — typed
APIs for Python callers, pyright/mypy work on compiled output
unchanged, and `$mod.Type` annotates the interop boundary with the
host's own types. Zero runtime; specs surface in `gan doc`,
`gandora_core.doc`, and the LSP.

**The LSP reaches Python (GEP-0015 rev 5).** For `$module` references
and `pyimport` aliases: hover shows the Python docstring, completion
lists module members, definition jumps into the Python source, and
signature help shows full Python signatures — jedi-resolved in the
project's own environment. New `textDocument/signatureHelp` also
covers Gandora calls: clause heads with per-parameter labels, specs
as documentation, active-parameter tracking.

## v0.4.2 — 2026-08-02

**The language server grows up (GEP-0015 rev 4).** gan-lsp now serves:

- **Hover v2**: clause signatures (guards and `\\` defaults rendered),
  bilingual prose, metadata, examples; `$module` references show their
  spec origin without importing; language constructs (`def`, `loop`,
  `quote`, ...) have embedded reference cards.
- **Go to definition**: modules and functions, across project sources
  and installed packages' shipped `.gan` sources.
- **Document symbols**: module outline with rendered heads.
- **Completion**: `Module.` members (public functions and macros) with
  signatures and doc summaries, dot-triggered.
- **Formatting**: the GEP-0016 engine as `textDocument/formatting`,
  verification included.

New `gandora_core` APIs: `definition`, `symbols`, and `doc` gained
`signatures`. Fixed along the way: guard heads (`def f(x) when ...`)
no longer read as a function named `when` in doc/symbol lookup, and
the printer renders `\\` defaults infix.

## v0.4.1 — 2026-08-02

- **LSP hover (GEP-0015 rev 3)**: hovering a `Module.function`
  reference, a module name, or a local function shows its GEP-0007
  docs — default-locale prose, every translation, metadata, and
  `@example` blocks — via the new `gandora_core.doc` lookup API.
- **Two-line dependency story (GEP-0013 rev 2)**: projects need only
  `gandora-std` (runtime) plus `gandora-tool[dev]` (dev group; the
  new extra aggregates the language server, compiler library comes
  transitively). Scaffolds emit exactly this.
- VS Code extension 0.1.6: eager output channel with startup logs
  (the silent-success case is now visible), server spawned with the
  workspace as cwd so the project venv's toolchain wins, pinned
  `extensionKind: workspace`.

## v0.4.0 — 2026-08-02

- **`gan fmt` (GEP-0016)**: conservative formatting written in
  Gandora — structural indentation (do/end, brackets, clause arrows,
  pipeline alignment, hanging continuations), horizontal whitespace,
  blank-line collapse, trailing-whitespace/final-newline hygiene, and
  `&$mod.fun/1` -> `&($mod.fun/1)` capture parenthesization. Never
  joins or splits lines; heredoc/sigil bodies shift as a unit. Safety:
  a rewrite whose comments or parsed terms differ from the original
  is refused (R006). `--check` for CI; the repository formats itself.
- New `gandora_core.tokens` API (GEP-0016-R001): the full lexical
  stream with comments, raw newlines, and end spans.
- The whole repository is now `gan fmt`-clean, enforced in CI.

## v0.3.0 — 2026-08-02

**Breaking: Python interop moves from `:module` to `$module`
(GEP-0003 revision 2).** `:` is now pure data — Elixir's atom pun is
only true where modules are atoms, and the one-spelling-two-meanings
ambiguity read badly. The dividend: `$module` is a **first-class
module reference** — `m = $math; m.sqrt(4.0)`, modules in
collections, `rescue e in $builtins.ValueError`. Dotted modules keep
the quoted form (`$"os.path"`). Every revision-1 spelling gets a
targeted compile error showing the `$` rewrite; migration is
mechanical. Quoted-term encoding adds `("__pyref__", meta, [name])`
(GEP-0012). The VS Code grammar highlights `$` references as
namespaces.

## v0.2.4 — 2026-08-02

- **Parser: block constructs are expressions everywhere.** `if`,
  `unless`, `case`, `cond`, `with`, `try`, `loop`, and `quote` now
  parse inside tuples, lists, map values, and call arguments —
  `{1, if ok do 2 else 3 end}` — matching Elixir, where they were
  previously statement-position only.
- **New diagnostic: Python stdlib shadowing.** A top-level module
  whose file name collides with a Python standard-library module
  (`Collections` -> `collections.py`) now gets a warning — such a file
  shadows the stdlib for everything on the project path and breaks
  imports in surprising ways.

## v0.2.3 — 2026-08-02

- **Codegen: f-string interpolations are hoisted when they contain
  quotes/backslashes/`#`** — Python only allows those inside f-string
  expressions from 3.12, and the target default is 3.11. Applies to
  `#{}` interpolation and `<%= %>` value splices alike.
- **`gan run` uses the project's `.venv` interpreter** when present
  (GEP-0013-R002 "execute with the project Python"), not the runner's
  own; `ganc` already did.
- `ganc --version` reports its own name.

## v0.2.2 — 2026-08-02

- **Heredoc dedent (GEP-0001-R026)**: `"""` strings follow Elixir's
  semantics — the opening newline is dropped and the closing
  delimiter's indentation is stripped. Docstrings and templates are
  now flush-left. `"""` sigil bodies dedent the same way
  (GEP-0009 rev 2).
- **Scaffold defaults to Python 3.11** (`gan init`/`ganc init`,
  `targetPython`, `.python-version`); every published package lowers
  its floor to `requires-python >= 3.11`.
- `gan init` rewritten with `~gan`/`~toml`/`~jsonc` sigil templates.
- **gan-lsp: full text sync announced explicitly** — pygls defaults
  to incremental, which would hand the compiler partial text on real
  editor edits. GEP-0015 conformance session test added and run in CI.
- Runner plugin lookup checks the project `.venv/bin` before PATH
  (GEP-0013-R003: installing a plugin is `uv add`).
- `ganc` usage/error text rebranded from its old `gan` identity.
- VS Code extension 0.1.3: doc-heredoc markdown no longer swallows
  the closing `"""`; per-delimiter sigil rules; splice injection
  keeps `<%= %>` highlighted inside embedded-language strings;
  `~gan`/`~toml`/`~jsonc` embedded highlighting.

## v0.2.1 — 2026-08-02

- gan-lsp rewritten on **pygls** (protocol machinery from the LSP
  ecosystem; language logic stays Gandora, attached via module
  attributes + @decorate).
- New **gan-lsc**: the Language Server Console — language intelligence
  as one JSON value per query (`version`, `diagnostics`, `ast`,
  `expand`, `compile`, `resolve`), the AI-facing isomorphic surface,
  reachable as `gan lsc`.

## v0.2.0 — 2026-08-02

The toolchain becomes Gandora (GEP-0012..0015).

- **gandora-core** (new): the compiler as a Python extension — parse,
  expand, diagnostics, compile, snippets, resolution; quoted terms in
  the Elixir encoding. Cargo workspace: core lib / `ganc` CLI / PyO3.
- **gan task runner** (new, `gandora-tool`): the mix/cargo entry point,
  written in Gandora — build/check/run/exec/repl natively over the
  library; `gan-<name>` plugin delegation, then `ganc` fallback. The
  Rust binary is renamed `ganc` (stage-0).
- **gan-lsp** (new, `gandora-lsp`): an LSP server written in Gandora —
  lifecycle + push diagnostics; VS Code client in `editors/vscode`.
- **Language** (GEP-0014): `try/rescue/after`, `loop/recur/break`
  (constant-stack iteration), and the `in` operator.
- Codegen fixes: parenthesized binary-operand postfix bases; submodule
  interop via quoted dotted atoms.

## v0.1.0 — 2026-08-02

On PyPI: [gandora-lang](https://pypi.org/project/gandora-lang/) (the
compiler) and [gandora-std](https://pypi.org/project/gandora-std/)
(the standard library), published via Trusted Publishing from CI.

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
