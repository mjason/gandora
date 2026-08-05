---
gep: 26
title: The Agent Surface
description: One-call context for AI sessions — lsc pack, batch/brief queries, and the gan agent entry point — so models stop paying tokens for query loops.
author: MJ
status: Accepted
type: Standards Track
areas:
  - Tooling
created: 2026-08-05
updated: 2026-08-05
revision: 1
requires: [15, 25]
replaces: []
superseded-by: null
resolution: null
translations:
  zh: local/zh/0026-the-agent-surface.md
---

# GEP-0026: The Agent Surface

## Abstract

`gan lsc` mirrors the language server: one question, one answer. For a
human in an editor that is right; for a model it is a token furnace —
an exploration phase of five or six queries costs thousands of tokens
and as many round trips before the first line is written. This GEP
adds the agent-shaped surface: batch and brief forms of the existing
queries, a one-call **context pack**, and `gan agent` as the session
entry point. Nothing is written into the user's project.

## Motivation

Measured in the recurring DeepSeek evaluation, a model's discovery
phase (`list_symbols` ×3, `read_doc` ×4, ...) spends 3–8k tokens and
4–7 round trips per task — almost all of it re-fetching facts that are
stable per project. The verdict already made *correction* one-call
(GEP-0025); this GEP makes *discovery* one-call.

## Specification

**GEP-0026-R001 (batch targets):** `gan lsc doc` and `gan lsc symbols`
accept multiple targets; two or more emit one JSON array (`doc`) or a
name-keyed object (`symbols`). One target keeps today's shape.

**GEP-0026-R002 (brief):** `--brief` on `doc` reduces every entry to
`{label, head, summary}` — the spec (or label) and the first doc
sentence. Detail stays one query away.

**GEP-0026-R003 (the context pack):** `gan lsc pack [Mod ...]` returns
one JSON object holding: the standard-library function lists (name per
module), every project module with its public heads (one line each),
the language-construct index plus the spec cheat sheet and a short
notes list, a verdict summary (`ok`/error/warning counts), and a
`next` pointer for going deeper. Naming modules adds their full member
docs under `deep`. The overview pack MUST stay prompt-sized (a few
thousand tokens) and deterministic for an unchanged project — cache
friendly by construction.

**GEP-0026-R004 (the entry point):** `gan agent` prints one Markdown
briefing — the working loop (build-verdict traffic light, the
apply-every-finding rule, thumb rules) followed by the rendered pack;
`--json` emits the raw pack instead. It is the recommended first
command of any AI session and writes **no files** into the project —
the project-pollution-free alternative to generated context files.

## Rationale

The pack is compiled from surfaces that already exist (`symbols`,
`doc`, construct cards, the check) — no second source of truth. A
static generated file (`llms.txt`) was rejected: it pollutes user
projects and goes stale; `gan agent` produces the same content on
demand from live facts.

## Conformance

Tests MUST cover: multi-target doc arrays and symbols objects; brief
entry shape; pack top-level keys and determinism; `pack Mod` deep
docs; `gan agent` printing loop text plus pack and degrading with a
clear message when `gan-lsc` is absent.

## Change History

- Revision 1, 2026-08-05: Initial version.
