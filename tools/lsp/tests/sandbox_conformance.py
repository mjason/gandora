"""Conformance for the AI sandbox (GEP-0023): `gan lsc try` verdicts.

Covers: a clean snippet run (stdout + value), member typo fuzzy match
(R002), cross-language migration hints (R003), undefined-variable and
keyword did-you-mean, practice hints, single main() execution, and
--no-run. Usage: sandbox_conformance.py [path-to-gan-lsc] [root]
"""

import json
import os
import subprocess
import sys

LSC = os.path.abspath(sys.argv[1]) if len(sys.argv) > 1 else "gan-lsc"
ROOT = os.path.abspath(sys.argv[2]) if len(sys.argv) > 2 else os.getcwd()

failures = []


def check(cond, label):
    print(("PASS " if cond else "FAIL ") + label)
    if not cond:
        failures.append(label)


def try_source(source, *extra):
    r = subprocess.run(
        [LSC, "try", "-", "--root", ROOT, *extra],
        input=source,
        capture_output=True,
        text=True,
        timeout=120,
        cwd=ROOT,
    )
    return json.loads(r.stdout)


def messages(d, kind=None):
    return [
        s["message"]
        for s in d["suggestions"]
        if kind is None or s["kind"] == kind
    ]


d = try_source('IO.puts("hello sandbox")\n1 + 41')
check(
    d["ok"] and d["stdout"].strip() == "hello sandbox" and d["value"] == "42",
    "clean snippet: runs, captures stdout and the value",
)

d = try_source("xs = [3, 1, 2]\nEnum.mpa(xs, fn x -> x * 2 end)")
check(
    any("Enum.map" in m for m in messages(d, "did_you_mean")),
    "Enum.mpa gets a fuzzy `Enum.map` suggestion (R002)",
)

d = try_source("def total(xs):\n    return xs")
check(
    any("no `return`" in m for m in messages(d, "migration"))
    and any("no colon" in m for m in messages(d, "migration")),
    "Python habits get migration hints (R003)",
)

d = try_source(
    "defmodule M do\n  def f(x), do: x + valeu\n  def g(value), do: value\nend"
)
check(
    any("`value`" in m for m in messages(d, "did_you_mean")),
    "an undefined variable suggests its nearest binding",
)
check(
    any("@spec" in m for m in messages(d, "practice")),
    "missing @spec on public defs is a practice hint",
)

d = try_source("defmodul M do\n  def f(x), do: x\nend")
check(
    any("`defmodule`" in m for m in messages(d, "did_you_mean")),
    "a misspelled keyword gets a did-you-mean",
)

d = try_source('defmodule M do\n  def main() do\n    IO.puts("run!")\n  end\nend')
check(d["stdout"] == "run!\n", "module main() executes exactly once")

d = try_source("1 + 1", "--no-run")
check(
    d["ok"] and d["stdout"] is None and d["python"] is not None,
    "--no-run compiles without executing",
)

print("=" * 40)
print("ALL PASS" if not failures else f"{len(failures)} FAILURES: {failures}")
sys.exit(1 if failures else 0)
