# Changelog

## v0.18.0 — 2026-08-05

**Named types (GEP-0027): generics get a declaration site.**

- **`@type name(params) :: type`** — named and generic aliases:
  `@type result(t) :: tuple(atom(), t)` declares, `result(integer())`
  / `Mod.result(string())` reference (project-wide), everything
  expands structurally at compile time into PEP 484 annotations —
  zero runtime, nested aliases welcome.
- **Every misuse teaches**: undeclared variable in a body, wrong arity
  at a reference, duplicate declarations, shadowing a builtin, naming
  a type `t` (retired), and recursion are compile errors with the fix;
  unknown lowercase types now get a did-you-mean over builtins *and*
  the module's own @type names.
- **Advice**: a @spec whose entire return is a type variable used
  nowhere else gets a practice hint — it constrains nothing.
- Tour demonstrates (`vec()`, `outcome(t)` in App.Mathy); `type`
  construct card; spec card and manual updated; deferred explicitly:
  constrained generics, generic structs, @opaque, cross-package types.
- Conformance: +2 cargo (+8 assertions), +2 BDD; lsp 57, cargo 183,
  five repos `clean: true`.

## v0.17.1 — 2026-08-05

**The module IS the type (GEP-0017 rev 5): `t` leaves the language.**

- A struct type is spelled by calling its module — `App.Shop()` — the
  exact mirror of `$pandas.DataFrame()` for host classes: **uppercase
  call = class**, one shape for both worlds. The former `Mod.t()`
  spelling is retired and errors with the `Mod()` recipe; bare `Mod`
  and parameterized `Mod(...)` teach their fixes too. No `t` appears
  anywhere in the type language — one less convention to memorize,
  for humans and models alike.
- Swept everywhere: tour, playground, docs site, manual, construct
  cards, BDD sources; all five repos hold `clean: true`.

## v0.17.0 — 2026-08-05

**The agent surface (GEP-0026): discovery in one call — plus two
spelling disciplines the audit demanded.**

- **`gan lsc pack [Mod ...]`**: the one-call context — std function
  lists, every project signature, the construct index, the spec cheat
  sheet, and a verdict summary, prompt-sized (~1.5k tokens) and
  deterministic (cache-friendly). Named modules ride along with full
  docs. **`gan agent [--json]`** is the session entry point: the
  working loop + the rendered pack in one output, writing no files
  into the project. Measured on the 24-task DeepSeek gauntlet with the
  pack injected: 24/24 green, mean steps 8.0 → 6.75, discovery
  `list_symbols` calls 14 → 4.
- **Batch + brief queries**: `gan lsc doc A B C [--brief]` returns one
  array (brief = `{label, head, summary}`, ~40 tokens each);
  `gan lsc symbols Mod1 Mod2` returns a name-keyed object.
- **Types are calls (GEP-0017)**: one spelling rule — `integer()`,
  `list(t)`, `Mod.t()`, `$mod.Type()` all take parentheses; only type
  variables (1–2 lowercase letters) and `nil` are bare. Bare `Mod.t` /
  `$mod.Type` now error with the parenthesized fix; `Mod.t(...)` with
  parameters errors. The rule caught std's own bare `$re.Pattern` on
  its first run.
- **Sigil name discipline (GEP-0005-R010)**: one-or-two-character
  sigil names are the functional whitelist `~w ~s ~r ~p` — anything
  else short is a compile error naming the list; 3+ characters are
  language-tagged text sigils. **`~p` is the prompt sigil now**
  (GEP-0009 rev 7), joining the functional family; `~prompt` remains
  an equivalent text tag.
- Advisor: Elixir-reflex hints stand down when the project really
  defines a module by that name (an `Agent` module of your own is
  yours).
- Conformance: +8 BDD (pack shape/determinism/deep, batch, brief,
  both spelling enforcements); lsp 55, gan 16, std 151, cargo 181.

## v0.16.1 — 2026-08-04

**One data spelling: `%json` withdrawn (GEP-0009 rev 6).**

- `%json` shipped in v0.16.0 and is repealed the same day: Gandora
  maps (`%{}`) are the one way to write data — richer than JSON (atom
  keys, arbitrary expressions) and already fluent for AI writers. A
  pasted JSON document becomes a map by swapping `:` for `=>`; the
  Advisor now teaches exactly that on sight of a JSON object line, and
  runtime JSON text is `$json.loads(s)`. `~json` stays what every `~`
  name is — a text template tag.
- The embedded family is two tiers now: `~` text, `$python(expr)`
  code. Mechanism exists only where the compiler truly understands
  the body; everything else is convention.

## v0.16.0 — 2026-08-04

**Three embedded tiers by symbol, prompts without escaping, JSON as a
literal — shaped by ten DeepSeek-in-the-loop rounds that ended 20/20.**

- **The tier/symbol split (GEP-0009 rev 5)**: `~` is uniformly *text*
  now — any sigil name (including `~python`/`~json`) is just a
  language tag on a raw string template; **`$python(expr)`** is the
  code splice (the `$` world — was `~python`, all 51 sites migrated,
  the Advisor carries the recipe); **`%json`** is the data tier.
- **`%json` compile-time data literals (GEP-0009-R007)**: the body is
  parsed by the JSONC reader at compile time (comments and trailing
  commas fine) and emitted as plain Python data — an OpenAI tool
  schema pastes verbatim, a typo is a compile error, zero runtime.
- **`~prompt` blessed (GEP-0009-R006)**: raw prose for AI prompts —
  quotes, braces, backslashes, inline JSON, no escaping ever; heredoc
  or one-liner; `<%= %>` splices values. The `\\\"` era is over.
- **The eval agent is now litellm-powered and pure Gandora**: a
  tool-calling DeepSeek loop (openai-compatible via api_base) with
  `~prompt` tasks and a `%json` tool schema — the language writing
  its own agent tooling is the proof of the AI-era thesis.
- std: `String.first/2`,`String.last/1` (Elixir parity);
  number-formatting taught where agents look (String moduledoc,
  `format` construct card, `defaults`/`with` cards enriched);
  `then/2` and doctest-repr teaching in the Advisor;
  did-you-mean identifiers no longer clip capitalized words.
- Ten recursive rounds, DeepSeek writing code against `gan build`
  every round: 14/16 baseline → 20 tasks → **20/20 twice in a row**,
  then four boss tasks (expression evaluator, struct receipt,
  retry pipeline, doctest-perfect module) **4/4** — convergence
  driven by verdict teaching, not model size.

## v0.15.1 — 2026-08-04

**Soft keywords stop being mangled where Python allows them.**

- Codegen fix (GEP-0001-R015): identifier keyword-mangling is now
  position-aware. `match` is a Python *soft* keyword — legal as a
  method name and a binding — so `pattern.match(line)` compiles as
  written (previously `match__kw`, an AttributeError at runtime), and
  `match = ...` stays bare. Hard-keyword bindings still rename
  (`class` → `class__kw`); a hard keyword in attribute position is a
  compile error teaching the `$builtins.getattr(x, "name")` spelling.
  With three regression tests; the Verifier's own ty-output parser now
  uses `.match()` as living proof.

## v0.15.0 — 2026-08-04

**Build is the verdict (GEP-0025 rev 3): artifact verification with ty,
one command, and ten more DeepSeek-in-the-loop rounds.**

- **`gan check` merged into `gan build`** (breaking): one command runs
  diagnostics, Advisor suggestions, and artifact verification, then
  compiles — errors stop artifacts. The old spelling prints a
  migration note and delegates. `gan lsc check` stays as the JSON
  surface (now including verification findings); `gan run` gates
  unchanged.
- **Artifact verification (R008)**: the generated Python is checked
  with [ty](https://docs.astral.sh/ty/) under resolution rules — an
  undefined function (`totl` → did you mean `total`?), a dead import
  (`Integer.to_string` → `import integer`), a missing module member
  (`Enum.mpa`), or a wrong-arity call is now a **build error** mapped
  back to the `.gan` source line, before anything runs. A dead import
  named like a Python builtin teaches `$builtins.round(...)`. ty ships
  as a gandora-tool dependency; the layer degrades to silence without
  it. gandora-core ships a `.pyi` stub so its own API verifies.
- **`gan build --strict`**: lifts the rule blocklist — full ty
  type-flow findings join as `[type]` warnings (opinions inform,
  facts gate). Same flag on `gan lsc check`.
- **Native default parameters**: `def f(x, y \\ 1)` with immutable
  literal defaults now compiles to a real Python signature
  (`def f(x, y=1):`) instead of a `*args` dispatcher — typed, honest,
  and arity-checkable; mutable defaults keep the dispatcher (call-time
  semantics). `Keyword.get/3` and friends gained real signatures.
- **std**: `Enum.chunk_every/3` (sliding step, Elixir parity).
- **Teaching**: richer `with` construct card (full chain shape),
  new `defaults` card, unresolvable-module guidance.
- DeepSeek-in-the-loop rounds (write → build → self-repair): baseline
  8/8 → hard set grown to 16 tasks, 14–15/16 green steady-state; the
  missing-dep scenario converges *because* verification errors teach
  the adaptation. Eval harness: submit refused while yellow, real venv
  scratch projects, chat retries.
- Conformance: +6 BDD (R008 errors, arity, source mapping, clean
  std-using program); lsp 47, std 148, cargo 175; all five repos hold
  `clean: true` under their own build.

## v0.14.1 — 2026-08-04

**The zero-noise verdict (GEP-0025 rev 2) — check recursively sharpened
over 20+ rounds against the language's own codebases and a weak-model
agent gauntlet (DeepSeek eval: 2/8 → 7/8+ green).**

- **The trust line (R005)**: an idiomatic project now verdicts **zero
  suggestions** — std, the toolchain, the tour, and the playground all
  hold `clean: true` under their own check. Rules earned exemptions by
  surviving that bar: `$builtins` repetition, `rescue _e ->` swallows,
  `Mod.t()` / `$mod.Type()` spellings on `@spec` lines, dot-call
  `fn x -> g.(x) end` wraps (not capturable), and test modules
  (exempt from library annotation coverage).
- **Verdict ergonomics (R004/R007)**: `ok`/`clean` traffic light
  specified; identical findings consolidate across files with the
  spread annotated; every suggestion carries the line of its first
  evidence; `gan check` prints `path:line`.
- **The project surface (R006)**: check now covers top-level
  `tests/*.gan` (Advisor pass) — exactly what `gan test` runs; deeper
  fixture directories are not advised.
- **Masking hardened**: paren-sigil bodies are masked with a
  balanced-parenthesis scan — `~python(next((...), None))` no longer
  leaks `None` into migration hints.
- **Compiler fix (found by dogfooding the Advisor's own advice)**:
  `&f/1` captures of a `defp` now resolve to the private compiled
  name, and `&to_string/1` / `&inspect/1` compile to `str` / `repr`
  (kernel forms have no function object). With regression test.
- **Dogfood sweep**: every remaining true finding applied across
  std/toolchain/tour/playground — abstract `sequence()`/`mapping()`
  spec params in std, pyimport graduations, capture conversions,
  comprehension rewrites, doctests on every module (tour 16/16),
  precise `$lsprotocol.types.*Params()` specs on all LSP handlers
  (also fixes a pygls injection hazard with `term()` annotations),
  playground verify grew to 164 cases.
- **Elixir-reflex teaching**: `Integer.` / `Stream.` / `File.` /
  `GenServer.` and 12 other Elixir-stdlib reflexes that would compile
  to a dead bare import now get a targeted migration hint; bare Python
  exception names (`ZeroDivisionError`) teach the `$builtins.` spelling;
  `%{x | field: v}` on a struct value teaches `%Mod{x | field: v}`;
  `{:ok, t}` tuple literals in specs teach `tuple(atom(), x)`.
- **std fix (found by the check's own dogfood)**: `Enum.take/2` and
  `Enum.drop/2` now honor Elixir's negative-count semantics (take -2 =
  last two; drop -2 = drop the last two) — they previously sliced the
  wrong end. With doctests and unit tests.
- **Agent eval hardened** (gan-playground): 8 tasks; submit is refused
  while the verdict is yellow (the gate teaches); scratch projects get
  a real venv so std resolves; runtime errors show the traceback tail.
  DeepSeek weak-model run: 2/8 → 7/8 green before the struct hint.
- Conformance: +17 BDD scenarios (R004–R007, each R005 exemption,
  every new reflex/exception/struct hint, tuple-spec teaching);
  lsp suite 42, fmt 15, std 147, cargo 174.

## v0.14.0 — 2026-08-04

**One verdict, one gate — and a toolchain that tests itself in Gandora.**

- **GEP-0025 (The Check, supersedes GEP-0023)**: `gan try` is gone.
  `gan check` is the whole verdict — compiler diagnostics plus Advisor
  suggestions (practice, migration, did-you-mean) per file;
  `gan lsc check` returns `{ok, clean, diagnostics, suggestions}`.
  **`gan build` and `gan run` gate on check**: errors stop before
  anything happens, the heavy-compiler way.
- **Advisor moves into gandora-tool** — one teaching engine below the
  runner, lsc, and future surfaces; gains a gandora-std introspection
  fallback (venv-less projects still get `Enum.mpa → Enum.map`), a
  `Python.`-guess migration hint, and `lsc doc` falls back to Python
  module docs (`gan lsc doc math`).
- **The conformance suites are now Gandora**: the check BDD
  (`test_check_verdict.gan`), the fmt CLI contract
  (`test_fmt_cli.gan`), and a JSON-RPC client driving gan-lsp
  (`test_lsp_protocol.gan`) — all ExUnit-style, all run by `gan test`.
  The Python harnesses are deleted; CI runs `ganc test`.
- Breaking: `gan try` / `gan lsc try` / `gan lsc review` removed;
  `gan lsc check` changes shape.

## v0.13.0 — 2026-08-04

**ExUnit parity, macros that talk back, and the verdict at project scope.**

- **GEP-0024 rev 2 — the ExUnit surface**: `use Test` brings
  `test "name" do`, `describe "prefix" do` (name prefixing),
  `assert a == b` (failures report left and right), `refute`, `in`
  membership, plus typed `assert_raise/2`, `assert_in_delta/3`,
  `flunk/1`. Macros compile to plain defs — pytest sees ordinary
  tests. Works from the installed wheel (macros ship as .gan sources).
- **GEP-0002 rev 3 — the macro kit**: string builtins
  (`slug/downcase/replace/to_atom`), pattern matching over quoted code
  (`{:"==", _m, [l, r]}`, do-block pairs), macro-as-function calls
  with recursion — and **`compile_warn/1`**: library macros raise
  spanned compiler warnings through the same channel as kernel lints.
  Extensions now teach with the kernel's voice.
- **GEP-0023 rev 4 — `gan lsc review`**: the sandbox's teaching pass
  over every project source (check + per-file practice/migration/
  member suggestions).
- **Gauntlet**: 14 hostile-input scenarios (pure Python, JS, Ruby,
  YAML, SQL, Markdown, truncated blocks, BEAM-isms) — every one
  degrades to a JSON verdict, never a crash; BDD now 83 scenarios.
- std tests grew to 145 (ExUnit-style suite included).

## v0.12.0 — 2026-08-03

**The official test tool — and the first recursive-improvement round.**

- **GEP-0024**: `gan test` now runs `tests/*.gan` after the doctests —
  test modules compile with the project's full module resolution and
  execute under pytest; `Test` (std) ships the assertion family
  (`assert_eq/true/false/nil/raises/contains`). std carries 134 such
  tests, the fmt engine 10, the playground a demo suite.
- **`Keyword.get/3`** (default value) joins std — found by the new
  test suite.
- **`gan lsc doc` answers language constructs** (`for`, `recur`,
  `with`, ...) — the dead end agents hit when asking about concepts is
  gone; construct cards are shared with hover.
- **Fixed**: `alias`/`import` of modules in a `pyPackage` project
  emitted unprefixed imports (`from test import *` instead of
  `from gandora_std.test import *`); snippet/REPL top level now
  applies GEP-0021 closure snapshots; expression-position `recur`
  errors cite GEP-0019, not the retired loop; spec-type errors teach
  the correction (`:ok` → atom(), `Int` → integer(), 20+ mappings);
  `gan lsc` never prints a bare traceback.
- **Sandbox**: practice hints now embed copyable templates; BDD grew
  to 69 scenarios; the DeepSeek agent evaluation (written in Gandora,
  `gan-playground/src/agent_eval.gan`) converges 6/6 naive tasks.

## v0.11.2 — 2026-08-03

- **`gan try`** — the sandbox gets its first-class spelling
  (`gan lsc try` remains identical): `gan try --help`, `gan try -`,
  `gan try snippet.gan [--no-run]`.

## v0.11.1 — 2026-08-03

**The sandbox grows teeth — and learns when to stay silent.**

- **Practice engine rebuilt for how AI actually fails** (rarely typos,
  often shortcuts): consolidated annotation-coverage report
  (@spec/@doc/@moduledoc/@example), "abstract in, concrete out" on
  @spec parameters, map+filter chains → `for`, `fn x -> f(x) end` →
  `&f/1`, `count == 0` → `Enum.empty?`, bare rescue → specific types.
- **Silence guarantee**: strings, heredocs, sigils, and comments are
  masked before any check — prose can never trip a pattern, and an
  idiomatic module returns zero suggestions.
- **Skill-style help**: `gan lsc try` (no args / --help) prints the
  full guide — JSON contract, suggestion kinds, the agent loop.
  Exit code 0/1 makes verdicts scriptable.
- **BDD conformance**: 39 Given/When/Then scenarios across six
  features (execution, did-you-mean, migration, practice, lints,
  silence) — `tools/lsp/tests/sandbox_bdd.py`.

## v0.11.0 — 2026-08-03

**The sandbox: AI writes it, `gan lsc try` verdicts it — and teaches.**

- **GEP-0023**: `gan lsc try <file|-> [--no-run]` — one JSON verdict
  for generated code: compile + lints, then execution in a temp dir
  with the project interpreter under a hard timeout (stdout + the
  snippet's last value captured), plus the generated Python itself.
- **Did-you-mean** (edit-distance over real candidates): `Enum.mpa` →
  `Enum.map` (checked against actual module symbols), `valeu` →
  `value` (the snippet's own identifiers), `defmodul` → `defmodule`.
- **Migration hints** for cross-language habits: `return`, `while`,
  `lambda`, `None/True/False`, Python `def ...():`, `import`,
  `&&`/`||`, `+=`, f-strings, `self.`, `switch`, `== nil` — each
  answered with the Gandora spelling.
- **Practice hints**: public defs missing `@spec`; repeated `$mod`
  where a `pyimport` is the idiom.
- The manual's AI toolbox documents the loop: generate → try → apply
  suggestions → try → write.

## v0.10.4 — 2026-08-03

**Interop housekeeping: pyimport over repeated boundary spellings.**

- **GEP-0003 rev 6** tooling guidance: a module used repeatedly SHOULD
  be a `pyimport` (bare binds the first segment; `as:` renames) —
  bare-name attribute chains carry no import ambiguity, so repeated
  `$(...)` spellings are a smell. The toolchain practices it: fmt's
  six `$(sys)` sites and the LSP's `$(os)` became one `pyimport` each.
- **LSP**: hover/jedi intelligence now resolves bare `pyimport sys`
  names too (the alias map only knew `as:` forms).

## v0.10.3 — 2026-08-03

**Hover polish: your language by choice, and method pipes explained.**

- **GEP-0015 rev 9**: with no locale set anywhere, hovers show the
  default language only — translations stay out of the way until a
  `gandora.local.jsonc` / `GAN_DOC_LOCALE` / editor setting asks for
  them ("all" is now the explicit opt-in for every-language sections).
- **`|> .method()` hover**: postfix method-pipe tokens get a proper
  construct card (a Python method on the piped value) instead of
  falling into local-variable inference.
- Local-variable hover no longer echoes junk when jedi has nothing to
  say (`upper : upper (inferred)` is gone).

## v0.10.2 — 2026-08-03

**Docs in your language, examples on every std function.**

- **GEP-0015 rev 8 (R015)**: documentation language is a developer
  preference — `gandora.local.jsonc` (`docLocale`, gitignored,
  per-project-per-person) > `GAN_DOC_LOCALE` env / VS Code setting >
  bilingual sections. Hover and signature help render one language
  when asked, **localized parameter tables included** (they were
  English-only before), with per-item fallback.
- **std: 94/94 functions now carry runnable `@example` doctests**
  (69 added), every one verified by `gan test`; `Map.new/1` (pairs →
  map) joins the API; `Enum.each` documents its `:ok` return.
- **Fixed**: docstrings containing backslashes halved on load
  (`re.compile("\\d+")` degraded to a SyntaxWarning); docstring
  emission now escapes them.

## v0.10.1 — 2026-08-03

**The tour, tools, and std now practice everything they preach.**

- **New tour chapter** `recursion + comprehensions`: tail recursion at a
  million frames, checked `recur`, structural recursion with
  `@allow :stack_recursion`, and `for` comprehensions with pattern-skip
  and `into:` — every head fully `@doc`/`@spec`/`@param` annotated
  (bilingual), so hovering it shows the compiled shape.
- **Tour-wide annotation sweep**: every public function in every chapter
  carries `@doc` + `@spec` (host types like `$pandas.DataFrame()` and
  struct types like `App.Shop.t()` included) — the examples now
  demonstrate the documentation standards, not just the syntax.
- **Toolchain surface**: the runner, lsc, PyIntel, and the LSP's pure
  helpers gained specs and docs; std was already at 100% (94/94).

## v0.10.0 — 2026-08-03

**The tooling milestone: find-references, rename, quick fixes — for editors and AI alike.**

- **LSP (GEP-0015 rev 7)**: `references` (call sites across the project,
  alias- and `&`-capture-aware, exact name-token ranges — string
  interpolations included), cross-file `rename` (refuses targets defined
  outside the project), `workspace/symbol` search, and quick-fix code
  actions that insert `@allow :...` or the `_` prefix right from a lint
  squiggle.
- **lsc**: `references`, `wsymbols`, and `check` — whole-project
  diagnostics with lints as one JSON value, the AI-agent workhorse.
- **fmt (GEP-0016 rev 2)**: `gan fmt -` (stdin→stdout, verified, exit
  0/2) and `--diff` (unified diff, rewrites nothing, exit 1).
- **Fixed (GEP-0003 rev 5)**: `$(sys)` — the single-segment explicit
  boundary — was silently falling back to the dotted-chain heuristic
  (`$(sys).stderr.write` imported `sys.stderr`); bounded-ness now
  travels through the AST. Also fixed: spans inside `#{...}`
  interpolations were fragment-relative; diagnostics, hover, and
  references inside interpolations now land on real source lines.

## v0.9.3 — 2026-08-03

**Rust-style lints: the compiler now proves your code unsafe before Python does.**

- **GEP-0022**: five statically provable warnings, each pinned to its
  definition in `gan check`/`gan build`/editor squiggles:
  - *undefined variable* — a read nothing binds is a guaranteed
    `NameError`; pyimport names known, `import Mod` stands the lint down
  - *unused binding* — suggest the `_` prefix (sigil `<%= %>` splices
    count as reads)
  - *unreachable clause* — a guard-less all-variable head shadows the
    same-arity clauses after it; ditto `case` wildcards
  - *discarded comprehension* — `for` in statement position; use
    Enum.each for side effects
  - *unused defp* — dead code; acknowledge deliberate keep-alive with
    `@allow :unused_function`
- The sweep over our own five codebases found and removed one real
  dead statement in the LSP's jedi bridge — the lint paid for itself
  before shipping.

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
