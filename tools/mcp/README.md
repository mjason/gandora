# gan-mcp

The Gandora MCP surface, written in Gandora — released on PyPI in
lockstep with the rest of the toolchain, and installed as a
development dependency of the project it serves.

## Why it exists

An AI writing Gandora can already ask `gan lsc` what a name means. What
it cannot do is *know that its own code works* — so it guesses, and
ships the guess. This server closes that loop: every snippet it hands
back has been compiled in a throwaway project, judged by the same
verdict a user's `gan build` gives, and had its `@example` doctests
really executed.

The rule the surface is built on: **nothing is returned that has not
been run.**

## Tools

| tool | cost | what it answers |
| --- | --- | --- |
| `gan_example(requirement)` | a model call | Compose a complete module for this requirement — returned only after it compiled, passed the practice pass, and had its doctests run. |
| `gan_verify(source)` | none | Does this module compile, is it idiomatic, and do its doctests pass? Returns errors, practice suggestions, and the real doctest output. |
| `gan_doc(target)` | none | What is `Enum.map` / a bare name / `defmacro`? |
| `gan_pack(modules)` | none | The one-call context pack (GEP-0026). |
| `gan_check()` | none | The verdict for the whole project. |
| `gan_briefing()` | none | The `gan agent` session briefing. |

Only `gan_example` consults a model; the rest forward facts and cannot
invent anything.

## How the composer works

1. **Ground it.** The prompt carries the context pack (what the standard
   library actually contains), the atoms — verified `@example` doctests
   the test suite already runs — that best match the requirement, and
   the construct cards the requirement names (`defmacro`, `defstruct`).
   The model composes from supplied facts; it is never asked to recall
   the language.
2. **Judge it.** The draft becomes a sandbox project and faces
   `gan lsc check`, then its doctests are executed.
3. **Escalate on failure, not on suspicion.** Round one runs with
   thinking off (1–2s). A failing verdict — the actual diagnostics and
   the actual doctest output — is what raises the effort for the next
   round. Four rounds; if none goes green, the surface returns the
   findings and the closest verified atoms, never the last draft.

Measured on eleven requirements run twice (22 compositions) through the
protocol: **20 green**, 14 of them on round one at a median 2.5s. The
two failures were a macro that hit `GEP-0002-R003` (no remote calls in
macro bodies) and a deliberately impossible requirement — both returned
as failures carrying their findings, neither as code.

The bar is what it says it is, and no more: green means *the module
does what its own example claims*. It cannot mean *the requirement was
answered* — asked for something impossible, one run produced a
compound-interest function whose example used a 0% rate, correct and
irrelevant. Read what you get.

## Schemas are not written twice

`@spec` becomes the Python signature, the signature becomes the MCP
`inputSchema`, and `@doc`/`@param` become the tool description:

```
@doc "The documentation for one target."
@param target, "What to look up."
@spec gan_doc(string()) :: term()
        ↓
def gan_doc(target: str) -> object:
        ↓
{"target": {"title": "Target", "type": "string"}}
```

Two constraints follow from that chain, and both are load-bearing:

- **No default arguments on a tool function.** `def f(x \\ [])`
  compiles to `def f(*_gan_args)`, which erases the signature MCP reads.
- **The `@spec` is the schema.** A missing or sloppy spec produces a
  sloppy tool contract, so the Advisor's insistence on annotations is
  the protocol's insistence too.

## It belongs to a project, not to a machine

Install it the way every other `gan` plugin is installed — as a dev
dependency of the project it serves:

```sh
uv add --dev gandora-mcp     # `gan init` already writes it into the dev group
gan mcp                      # resolves gan-mcp from ./.venv/bin first, then PATH (GEP-0013-R003)
```

`gan init` writes the config for each agent that reads a project-level
file, in the shape and location that agent documents (GEP-0028-R012) —
so a new project is wired before it is first opened:

| agent | file |
| --- | --- |
| Claude Code | `.mcp.json` (project scope; approve it on first run) |
| Codex | `.codex/config.toml` (read for a trusted project) |
| opencode | `opencode.json` |

All three declare the same command, and it holds no paths at all —
check them into the repository and they work for everyone:

```json
{ "mcpServers": { "gandora": { "type": "stdio", "command": "uv", "args": ["run", "gan", "mcp"] } } }
```

No `cwd`, no absolute path, no environment. `uv run` resolves the
project environment from wherever it is launched, `gan` finds
`gan-mcp` in that project's `.venv/bin`, and the server discovers the
project by walking up from the working directory to the nearest
`gandora.jsonc` — so being launched in a subdirectory is fine.
`GAN_MCP_ROOT` remains the explicit override for a client that launches
somewhere else entirely.

This is not merely tidier than a global install — it is what makes the
verdicts true. The sandbox judges a snippet inside the project's own
`.venv` (falling back to the server's), so a project pinned to an older
toolchain gets that toolchain's answers rather than whichever version
happens to be installed system-wide.

Developing on it here:

```sh
cd tools/mcp
uv sync
gan build
gan test
.venv/bin/gan-mcp        # speaks MCP over stdio
```
