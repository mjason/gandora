# Writing Gandora with AI

Gandora is designed on the assumption that much of its code will be
written by models. That shapes the toolchain in concrete ways: one
verdict, taught corrections, JSON everywhere, and prompts as
first-class text.

## The loop

**write → `gan build` (fix every finding) → `gan test` → ship.**

For an agent, the JSON surface is `gan lsc check`:

```json
{"ok": true, "clean": false, "diagnostics": [], "suggestions": [
  {"kind": "practice", "line": 3,
   "message": "Annotation coverage: missing @spec on: total — e.g. ..."}]}
```

Treat it as a traffic light: `ok: false` → fix errors; `clean: false`
→ apply every suggestion; `clean: true` → submit. Every message
contains the correct spelling — a model can apply it without opening
the manual.

## What the verdict catches for you

- **Cross-language reflexes**: `return`, `while`, `None`, f-strings,
  `Integer.to_string`, `Stream.map`, `|> then(...)`, bare exception
  names — each gets the Gandora spelling in one line.
- **Necessarily-fatal facts before running**: undefined functions
  (with did-you-mean against real symbol tables), dead imports,
  wrong-arity calls — from ty over the *generated* Python, mapped back
  to your source line.
- **Laziness, not typos**: models rarely misspell; they skip
  annotations, write map+filter chains where a comprehension reads
  better, and reach for `$math` five times instead of one `pyimport`.
  The practice pass names each shortcut and its fix.

## Prompts and tool schemas in the language

`~prompt` is raw prose — quotes, braces, backslashes, and inline JSON
need no escaping; `<%= expr %>` splices values:

```elixir
@task ~prompt(Parse the JSON string {"b": 2, "a": 1} and print its keys sorted.)

@system ~prompt"""
You are a coding agent. The user's name is <%= name %>.
Reply with {"status": "ok"} when done.
"""
```

Tool schemas are ordinary maps — a pasted JSON document becomes one by
swapping `:` for `=>` (the build teaches this on sight):

```elixir
@tools [
  %{"type" => "function", "function" => %{
    "name" => "build_code",
    "description" => "The build verdict for a Gandora module.",
    "parameters" => %{"type" => "object", "properties" => %{
      "code" => %{"type" => "string"}}, "required" => ["code"]}}}
]
```

## An agent, in Gandora

The repository's own evaluation harness is a tool-calling DeepSeek
agent written in Gandora — litellm for the model call, `~prompt` for
the tasks, tail recursion for the loop:

```elixir
resp = litellm.completion(
  model: "openai/" <> model,
  api_base: base, api_key: key,
  messages: messages, tools: tools(), timeout: 120
)
msg = Enum.at(resp.choices, 0).message
```

It matters because it is also the measurement: a small model that has
never seen the manual, holding only `build_code` / `run_code` /
`read_doc` / `list_symbols` / `submit`, converges to green on a
24-task gauntlet — fizzbuzz through a precedence-respecting expression
evaluator — purely by following the verdict's teaching.

## Discovery tools worth wiring in

```console
gan lsc doc spec        # the type-language cheat sheet
gan lsc doc with        # construct cards: shapes, not prose
gan lsc symbols Enum    # what actually exists
gan lsc compile f.gan   # what the Python will be
```
