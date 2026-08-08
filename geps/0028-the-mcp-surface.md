---
gep: 28
title: The MCP Surface
description: An MCP server that hands models verified Gandora — every example compiled, judged, and really run before it is returned.
author: MJ
status: Accepted
type: Standards Track
areas:
  - Tooling
created: 2026-08-06
updated: 2026-08-08
revision: 5
requires: [13, 15, 25, 26]
replaces: []
superseded-by: null
resolution: null
translations:
  zh: local/zh/0028-the-mcp-surface.md
---

# GEP-0028: The MCP Surface

## Abstract

`gan-mcp` is a Model Context Protocol server, written in Gandora, that
exposes the toolchain to any MCP client. It carries the answers a model
needs — documentation, the context pack, the verdict — and one answer
no previous surface could give: **a complete module that has been
compiled, judged, and really run**. This GEP fixes the rule the whole
surface exists to enforce: nothing is returned that has not been run.

## Motivation

GEP-0025 made correction one call and GEP-0026 made discovery one call.
Both improved what a model *knows*; neither changed what a model
*ships*. A model still writes a snippet, reads it back, finds it
plausible, and hands it over — and plausibility is not a verdict. The
observed failure is not ignorance of syntax but confidence without
evidence: code that reads correctly, names a function that does not
exist, and reaches the user anyway.

Gandora is unusually well placed to close this. `@example` is not
decoration: it compiles to a native Python doctest that `gan test`
executes (GEP-0007). An example is therefore its own assertion, and a
module carrying one is a self-verifying artifact. Generating an example
and testing it are the same act; a surface that skips the second half
is choosing to.

A second reason the server belongs here rather than in a client: MCP
derives a tool's `inputSchema` from the function signature, and
Gandora derives the signature from `@spec` with the description from
`@doc`/`@param`. The annotation discipline the Advisor already enforces
is exactly the protocol's metadata. The contract is written once.

## Scope

Covered: the stdio server, its tool surface, the verification pipeline,
the composer that turns a requirement into a verified module, and the
project-level configuration that wires it into the agents that read
one. Out of scope: HTTP transports, MCP sampling, and prompts and
resources beyond the briefing.

## Terminology

**Sandbox project** — a throwaway project holding one module, built and
discarded per request.

**Atom** — a verified example of one language capability, drawn from a
source the test suite already runs.

**Composer** — the model-backed path from a requirement to a module.

## Specification

**GEP-0028-R001 (the server):** `gan-mcp` speaks MCP over stdio and is
itself a Gandora program. `GAN_MCP_ROOT` selects the project its
queries answer for, defaulting to the working directory.

**GEP-0028-R002 (nothing unrun):** Every code-bearing answer MUST carry
the verdict that produced it: the diagnostics, the practice
suggestions, and the result of executing the module's doctests. An
answer whose module does not reach `ok: true` MUST be returned as a
failure carrying those findings. Returning unverified code as prose —
however plausible — is a defect of this surface, not a degraded mode of
it.

**GEP-0028-R003 (the pipeline):** Verification writes the module into a
sandbox project at the path its own `defmodule` demands
(GEP-0001-R013), takes its verdict from `gan lsc check`
(GEP-0025-R009) rather than any private re-implementation, executes
the compiled artifact's doctests under a hard timeout, and removes the
sandbox afterwards. The user's project is never written to.

**GEP-0028-R004 (schemas are not written twice):** Tool inputs are
derived from `@spec`, descriptions from `@doc` and `@param`. A tool
function MUST NOT take default arguments, which erase the generated
signature, and MUST carry a spec precise enough to stand as a public
contract.

**GEP-0028-R005 (the zero-cost tools):** `gan_doc`, `gan_pack`,
`gan_check`, and `gan_briefing` forward surfaces that already exist
(GEP-0015, GEP-0025, GEP-0026) and MUST NOT consult a model. What
cannot be invented MUST NOT be.

**GEP-0028-R006 (the composer explains and demonstrates):**
`gan_example` takes one feature, syntax, or capability to demonstrate
and returns two things: a prose **explanation**, and a **complete
module** — `defmodule` with `@moduledoc`, and for each public function
`@doc`, `@param`, `@spec`, and at least one `@example` doctest — that
passed R002. It MUST NOT return a bare fragment: a fragment cannot be
compiled, cannot be doctested, and cannot become an atom, so it cannot
be verified at all. **Every line of code in the answer MUST live in the
verified module.** Code in the explanation would be an unverified claim
dressed as a verified one, so the surface strips it.

**GEP-0028-R007 (grounding before generation):** The composer's prompt
MUST carry the context pack and the atoms relevant to the requirement.
The model is asked to compose from supplied facts, never to recall the
language.

**GEP-0028-R008 (the verdict drives the effort):** The composer runs
with reasoning disabled on its first attempt. A failing verdict — not a
guess about difficulty — is what raises the effort on the next attempt.
Rounds are bounded; when the last one fails, the surface returns the
findings and the closest verified atoms, never the last draft.

**GEP-0028-R009 (atoms are earned):** An atom MUST come from a source
the test suite runs — a std or tour `@example`, or a composed module
that passed R002. A green composition MAY be recorded as an atom;
recording it does not exempt it from re-verification when served.

**GEP-0028-R010 (executing model-written code):** The sandbox runs code
the server did not write. It MUST execute in a temporary directory
under a timeout, MUST NOT be reachable from the user's project, and its
failure MUST surface as a verdict rather than as an exception crossing
the protocol boundary.

**GEP-0028-R011 (credentials):** Model configuration comes from the
environment (`GAN_API_KEY`, `GAN_MODEL`, and an optional
`GAN_BASE_URL` defaulting to the vendor endpoint). Credentials MUST NOT
be logged, echoed into a verdict, or written into a sandbox.

**GEP-0028-R012 (the project owns the wiring):** `gan init` MUST write
the project-level MCP configuration each agent reads on its own, in
that agent's documented shape and location — `.mcp.json` for Claude
Code, `.codex/config.toml` for Codex, `opencode.json` for opencode —
and MUST NOT overwrite one that exists. Every such file declares the
same pathless command, `uv run gan mcp`: no `cwd`, no absolute path,
no environment. `uv run` resolves the project environment from
wherever the client launches, `gan` finds `gan-mcp` in that project's
`.venv/bin` (GEP-0013-R003), and the server discovers the project by
walking up to the nearest `gandora.jsonc`. `gandora-mcp` is therefore
a development dependency of every scaffolded project. These files are
meant to be committed: checked in, the wiring works for everyone who
clones, and the verdicts they produce are the project's own, judged
inside its own environment rather than by whatever toolchain happens
to be installed system-wide.

**GEP-0028-R012A (agent instructions belong to the agents, rev 5):**
`gan init` writes nothing into `AGENTS.md` or `CLAUDE.md`, ever.
Those files carry each agent's own conventions, structure, and voice;
a tool appending a universal section into them is pollution, not
wiring — revision 4 specified exactly that and was withdrawn the same
day, against a real project. R012's config files are different in
kind: they are machine-read, single-shape, and ours to define. The
self-setup path is the server's `instructions` instead: they tell a
connected agent that lacks Gandora coverage in its own instruction
file to add a short section itself, in that file's conventions — the
loop, retrieval before editing, `gan_verify` as the only proof. Each
AI configures itself through its normal edit flow, with its user
watching; the MCP server keeps its property of never writing into the
project.

## Rationale

The surface refuses the split most codegen tools accept — a fast path
that answers from the model and a slow path that checks. Here the check
*is* the answer: a verdict of `clean: true` with passing doctests is
the only thing that distinguishes this server from a model that has
read the manual. Making that non-negotiable (R002) is what lets an
agent hand the result straight to a user.

R006 chooses the module over the fragment because the module is the
smallest unit the toolchain can judge. A fragment is shorter to read
and impossible to verify; the extra lines are the evidence. Splitting
the answer in two is the same discipline the language already applies
to its own documentation: `@doc` is prose, `@example` is executed, and
nothing is both.

The prompt that drives the composer is itself written in the language's
prompt syntax — `~p"""` with `<%= %>` splices (GEP-0009-R006) — and it
carries no hand-written module template. The shape of an answer is
shown by corpus modules that the test suite compiles and runs, so the
one piece of code in the pipeline that nobody verified does not exist.

R008 inverts the usual reasoning-budget question. With a ground-truth
verifier in the loop, an iteration carrying a real diagnostic teaches
more than deeper deliberation over the same guess — so effort follows
failure rather than anticipating it.

## Security and Determinism

The sandbox executes generated code, which is the same trust boundary
`gan run` already crosses on the user's behalf, narrowed: temporary
directory, hard timeout, no network requirement, removed afterwards. A
verified answer is reproducible for an unchanged toolchain — the
verdict and doctest output are facts about the artifact, not about the
model that drafted it.

## Tooling and AI Usage

An MCP client SHOULD call `gan_briefing` once per session, `gan_doc` or
`gan_pack` for meaning, and `gan_verify` on every module it intends to
hand over — including modules it wrote itself. `gan_example` is for
when the requirement is clear and the shape is not.

## Conformance

Tests MUST cover: a correct module reaching `ok`/`clean` with doctests
run and passed; a module calling a std function that does not exist
failing with the artifact diagnostic; a module whose expected doctest
output is wrong compiling clean yet failing its doctests; module-name
to sandbox-path derivation including a dotted name; the tool schemas
derived from `@spec`; every corpus module surviving the same verdict it
teaches; and the composer returning either an explanation plus a module
that passed R002, or a failure carrying findings — never an unverified
draft.

## Change History

- Revision 5, 2026-08-08: R012A inverted — `gan init` does NOT write
  agent-instruction files; revision 4's append-a-section design
  polluted real projects' `AGENTS.md`/`CLAUDE.md` with a voice not
  their own and was withdrawn the same day. The server's
  `instructions` now carry the self-setup nudge: each agent adds its
  own short Gandora section, in its own file's conventions.

- Revision 4, 2026-08-08: R012A — `gan init` wires the
  agent-instruction files: creates `AGENTS.md` when absent, appends
  the marker-delimited Gandora section when present, and adds one
  `@AGENTS.md` import to an existing `CLAUDE.md`. The section is a
  pointer, not a rulebook copy; the MCP server continues to write
  nothing into the project. Withdrawn by revision 5.

- Revision 3, 2026-08-08: the write-once contract is now fully wired.
  A tool is a bare `@tool true` marker on the function that implements
  it (`defattr`/`@on_definition`, GEP-0008 rev 2) — the hook derives
  the table entry from the definition's own name and arity — and
  registration carries the whole annotation surface into the protocol:
  the `@doc` prose is the tool description (previously a hand-kept
  blurb overrode it), and each `@param` text is injected into its
  `inputSchema` property, parsed from the `## Parameters` docstring
  section our own codegen emits (GEP-0018) — a contract rather than a
  guess. The marker carries no prose of its own: a second description
  channel on the same definition would be exactly the duplication this
  GEP exists to forbid. Nothing about a tool is written twice.

- Revision 2, 2026-08-07: Accepted. `gandora-mcp` joins the release
  chain as the sixth published package, and R012 makes `gan init`
  write the project-level MCP configuration for Claude Code, Codex,
  and opencode — one pathless command, checked in, working for
  everyone who clones.
- Revision 1, 2026-08-06: Initial version.
