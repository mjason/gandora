# Gandora Practices — the house style

Gandora is a language for AI authors and human reviewers: the same
line count that saves a model output tokens is what lets a person
audit the code at a glance. These rules exist for that trade. They are
not aspirational prose — the practice engine in `gan build` /
`gan lsc check` enforces every one it can prove, `gan lsc doc
practices` answers with the digest, and `gan agent` carries the digest
into every session briefing.

## 0. Declare, don't narrate

The philosophy every rule below instantiates: **declarative beats
procedural**. State what things *are*, once, where they belong — and
let machinery derive what happens. Procedural code narrates its
decisions: a `case` with twelve arms, a usage string hand-maintained
beside it, a registration block repeating every name a third time.
Each narration is a copy, and copies disagree. Declarative code states
the fact on the definition itself:

```gandora
@command {"build", "[--strict]", "the verdict + compile"}
defp build_cmd(rest), do: build(Enum.member?(rest, "--strict"))
```

A `defattr` + `@on_definition` hook (GEP-0008) accumulates these into
one table; usage text and dispatch both read it. The help cannot
describe a command that does not run, and no command runs
undescribed. `gan`'s CLI, `gan-lsc`'s query surface, and `gan-mcp`'s
tool table are all this one shape — and so is `@doc` itself, which is
the same mechanism, built in.

The ladder of preference: a **data table** walked by `Enum` beats
branching logic; an **annotation on the definition** beats a table
maintained elsewhere; a **macro** beats both when the repetition
happens at compile time. Climb to the next rung only when the current
one cannot state the fact.

## 1. Data flows left to right

Pipe the subject through its transformations. Nested calls read inside
out and force the reviewer to hold a stack in their head; a pipeline
reads once, in order.

```gandora
# no — three levels deep before the subject appears
Enum.sort($builtins.list(pyglob.glob(pattern, recursive: true)))

# yes — the story in reading order
pyglob.glob(pattern, recursive: true)
|> $builtins.list()
|> Enum.sort()
```

Every standard-library function takes its subject first, so every call
is pipe-ready. Two calls may nest (`inspect(x)` inside a `IO.puts` is
fine); three is where the engine draws the line. Assertion lines in
tests are exempt — `assert_eq(f(g(x)), want)` is the universal test
idiom.

## 2. `for` is a comprehension, not a loop

A bare `for x <- xs, do: f(x)` is `Enum.map` wearing a costume — write
the pipeline. `for` earns its place when it does what `Enum.map`
cannot say in one pass:

```gandora
for x <- xs, rem(x, 2) == 0, do: x * x          # fuse filter + map
for {k, v} <- pairs, into: %{}, do: {k, v * 10} # build a map
for {:ok, v} <- results, do: v                  # pattern skip
```

The reverse also holds: an adjacent `Enum.filter |> Enum.map` fuses
into one comprehension. (`Enum.map |> Enum.filter` does not — the
filter tests the mapped value, and there is no `for` spelling for
that; keep piping.) One genuine exception: `await` in the body must
stay a comprehension, because `fn` cannot hold an `await` (GEP-0030).

## 3. Name the function, don't wrap it

`&f/1` over `fn x -> f(x) end`. The capture says "this function,
arity 1" in five characters; the wrapper says the same thing in
twenty and adds a binding to read past.

## 4. Outcomes are verdicts

A boundary that can fail returns a tuple the caller can match:
`{:ok, value}` / `{:error, why}` — the shape `File.read/1` and
`Task.try_await/2` model. For query code where any failure means
"no answer", guard with the `Safe.safe/2` macro from `gandora-tool`:

```gandora
import Safe
syms = safe(core.symbols(mod, root), [])
```

Reach for an explicit `try`/`rescue e in $mod.Type` only when the
failure itself carries meaning. A bare `rescue e ->` swallows every
exception and is flagged.

## 5. Host work goes through the stdlib

`Path`, `File`, and `System` (GEP-0010-R011) are the reviewable
spelling of the host's filesystem and process surface:

| instead of | write |
|---|---|
| `os.path.join(a, b)` | `Path.join(a, b)` — chain with `|>` |
| `pathlib.Path(p).read_text()` | `File.read!(p)` / `File.read(p)` |
| `os.path.exists(p)` / `isdir` | `File.exists?(p)` / `File.dir?(p)` |
| `p.rglob("*.gan")` | `Path.wildcard("src/**/*.gan")` |
| `subprocess.run([...])` | `System.cmd(bin, args, opts)` → `{out, status}` |
| `os.getenv(n)` / `sys.exit(c)` | `System.get_env(n)` / `System.halt(c)` |

Interop stays for what the stdlib does not wrap — that is what `$mod`
and `pyimport` are for (three or more `$mod` uses graduate to one
`pyimport`). The wrapper modules themselves are the one place the raw
interop belongs.

## 6. Repetition is a shape, and shapes are data

The same map literal built in four places is a constructor function.
The same registration written six times is a table the code iterates:

```gandora
Enum.each(tools(), fn {name, f, desc} ->
  server.add_tool(f, name: name, description: desc)
end)
```

When the repetition happens at compile time — the same `def` skeleton,
the same guard around every query — it is a `defmacro` with a
`quote`/`unquote` body, and the expansion is exactly what a reviewer
would have written by hand (`Safe.safe/2` is the working example).
And when the table describes the definitions themselves — commands,
tools, routes — it should not be maintained *near* them but declared
*on* them: `defattr` + `@on_definition` (GEP-0008) turns an annotation
into a table entry carrying the function's own name and capture, so
the data cannot disagree with the execution (chapter 0; `Cli` and
`Toolkit` are the working examples). Declare structure once; let the
language mean it everywhere.

## 7. Annotations are the contract

Every public `def` carries `@doc` (prose: what and why), `@param` per
parameter, `@spec` (abstract containers in parameter position —
`sequence(t)`, `mapping(k, v)` — concrete on return), and an
`@example` whose `gan>` lines actually run under `gan test`. Code in
an answer that has not run is a claim, not evidence.

## 8. The verdict is the reviewer's floor

`gan build` green means: compiles, zero practice findings, artifact
verified. The toolchain's own sources hold themselves to
`suggestions == []` (GEP-0025-R005) — the rules above survived being
applied to the code that enforces them.
