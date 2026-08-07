# Writing Gandora with AI

Gandora is designed on the assumption that much of its code will be
written by models. That shapes the toolchain in concrete ways: one
verdict, taught corrections, JSON everywhere, and prompts as
first-class text.

## Start with one call

`gan agent` prints the whole session briefing — the working loop plus
the **context pack** (GEP-0026): every std function name, every
project signature, the construct index, the spec cheat sheet, and the
current verdict, in one prompt-sized output. A model that starts from
the pack writes immediately instead of spending its first five tool
calls on discovery. `gan lsc pack --root .` returns the same as JSON;
`gan lsc pack Enum` deep-dives one module.

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

`~p` is raw prose — quotes, braces, backslashes, and inline JSON
need no escaping; `<%= expr %>` splices values:

```elixir
@task ~p(Parse the JSON string {"b": 2, "a": 1} and print its keys sorted.)

@system ~p"""
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
agent written in Gandora — litellm for the model call, `~p` for
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
gan agent               # everything above, once, as Markdown
gan lsc pack Enum       # one module's full docs in one call
gan lsc doc with for spec --brief   # many cards, one line each
gan lsc compile f.gan   # what the Python will be
```

## The MCP surface

`gan init` wires an MCP server into the project for the agents that read a
project-level file — `.mcp.json` (Claude Code), `.codex/config.toml`
(Codex), `opencode.json` (opencode). All three declare the same command,
`uv run gan mcp`, with no path and no `cwd`: commit them and they work for
everyone who clones.

Through it an agent asks the toolchain rather than its own memory:

| tool | what it answers |
| --- | --- |
| `gan_example` | a complete module for this requirement — returned only after it compiled and its doctests really ran |
| `gan_verify` | does this module compile, is it idiomatic, do its doctests pass? |
| `gan_doc` / `gan_pack` | what a name means; the one-call context pack |
| `gan_check` / `gan_briefing` | the project verdict; the session briefing |

Only `gan_example` consults a model; the rest forward facts and cannot
invent anything. The rule the surface enforces is the one above: nothing is
returned that has not been run
([GEP-0028](https://github.com/mjason/gandora/blob/main/geps/0028-the-mcp-surface.md)).
