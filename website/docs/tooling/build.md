# The Build Verdict

`gan build` is Gandora's single quality gate (GEP-0025): one command
runs the whole judgment and then compiles. There is no separate lint
step, no separate checker to remember — the build *is* the verdict.

## What the verdict contains

1. **Errors** — the program cannot compile (or cannot run: see
   artifact verification below). The build stops before writing
   artifacts; exit code 1.
2. **Warnings** — statically provable facts that don't block:
   stack-growing recursion (GEP-0019), unused bindings, unreachable
   clauses, discarded comprehensions (GEP-0022). Facts, never
   opinions.
3. **Suggestions** — the Advisor's teaching pass: best-practice gaps
   (`practice`), cross-language habits (`migration`), and misspelled
   names checked against real symbol tables (`did_you_mean`). Each one
   carries the line of its first evidence and the correct spelling.

```console
$ gan build
warning: src/prog.gan:8: fact/1 is self-recursive outside tail position ...
practice: src/prog.gan:3: Annotation coverage: missing @spec on: total ...
did_you_mean: src/prog.gan:12: `Enum.mpa` is not a function of Enum — did you mean `Enum.map`?
compiled 3 module(s)
```

## Artifact verification

After codegen, the generated Python is checked with
[ty](https://docs.astral.sh/ty/) under *resolution rules only*: an
undefined name, an unresolvable import, a missing module member, or a
wrong-arity call is **runtime-fatal fact**, so it reports as a build
error mapped back to your `.gan` source line:

```console
error: src/prog.gan:13: Name `totl` used when not defined — did you mean `total`?
error: src/prog.gan:5: Cannot resolve imported module `requests` — `$x`/`pyimport x` needs an importable module
```

Type-flow *opinions* never gate a build. Opt into them with
`gan build --strict`, which reports the full ty analysis as `[type]`
warnings.

## The traffic light

`gan lsc check` returns the same verdict as one JSON object — the
AI-facing surface:

```json
{"ok": true, "clean": false,
 "diagnostics": [...], "suggestions": [{"kind": "did_you_mean", "line": 3, ...}]}
```

- `ok: false` — **red**: fix errors.
- `ok: true, clean: false` — **yellow**: apply every suggestion.
- `clean: true` — **green**: ship it.

## The zero-noise trust line

A rule only earns its place in the Advisor by staying silent on
idiomatic code: Gandora's own standard library, toolchain, example
tour, and playground all verdict `clean: true` under their own build.
When the verdict speaks, it is worth reading — that is the contract.

Identical findings across many files collapse into one annotated
entry; test modules are exempt from library annotation coverage; the
verdict covers `src/` plus top-level `tests/*.gan` — exactly what
`gan test` runs.
