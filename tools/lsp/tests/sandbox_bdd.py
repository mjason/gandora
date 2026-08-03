"""BDD scenarios for the Gandora sandbox (GEP-0023): `gan lsc try`.

Every scenario is Given (source) / When (try it) / Then (expectations
on the JSON verdict). Run: sandbox_bdd.py [gan-lsc] [root] [filter].
"""

import json
import os
import subprocess
import sys

LSC = os.path.abspath(sys.argv[1]) if len(sys.argv) > 1 else "gan-lsc"
ROOT = os.path.abspath(sys.argv[2]) if len(sys.argv) > 2 else os.getcwd()
FILTER = sys.argv[3] if len(sys.argv) > 3 else ""


def run_try(source, *extra):
    # a "__FILE__:<path>" arg switches to file-target mode (source unused);
    # bytes sources exercise the non-UTF8 stdin path
    target = "-"
    args = []
    for a in extra:
        if isinstance(a, str) and a.startswith("__FILE__:"):
            target = a.split(":", 1)[1]
        else:
            args.append(a)
    binary = isinstance(source, bytes)
    r = subprocess.run(
        [LSC, "try", target, "--root", ROOT, *args],
        input=source if target == "-" else None,
        capture_output=True, text=not binary, timeout=120, cwd=ROOT,
    )
    out = r.stdout.decode() if binary else r.stdout
    return json.loads(out), r.returncode


def kinds(d, kind):
    return [s["message"] for s in d["suggestions"] if s["kind"] == kind]


def all_msgs(d):
    return [s["message"] for s in d["suggestions"]]


# ---- expectation helpers ----------------------------------------------------

def ok(d, rc):            return d["ok"] is True and rc == 0
def not_ok(d, rc):        return d["ok"] is False and rc == 1
def clean(d, rc):         return ok(d, rc) and d["suggestions"] == []


def suggests(kind, needle):
    def check(d, rc):
        return any(needle in m for m in kinds(d, kind))
    check.__name__ = f"suggests {kind}: …{needle}…"
    return check


def no_suggestion_kind(kind):
    def check(d, rc):
        return kinds(d, kind) == []
    check.__name__ = f"no {kind} suggestions"
    return check


def diag_contains(needle):
    def check(d, rc):
        return any(needle in x["message"] for x in d["diagnostics"])
    check.__name__ = f"diagnostic …{needle}…"
    return check


def stdout_is(text):
    def check(d, rc):
        return d["stdout"] == text
    check.__name__ = f"stdout == {text!r}"
    return check


def value_is(text):
    def check(d, rc):
        return d["value"] == text
    check.__name__ = f"value == {text!r}"
    return check


def stage_is(stage):
    def check(d, rc):
        return d["stage"] == stage
    check.__name__ = f"stage == {stage}"
    return check


CLEAN_MODULE = '''defmodule Clean do
  @moduledoc "An idiomatic module — the sandbox must stay silent."

  @doc "Doubles every element."
  @param xs, "The numbers."
  @spec double_all(sequence(number())) :: list(number())
  @example """
      gan> Clean.double_all([1, 2])
      [2, 4]
  """
  def double_all(xs), do: for x <- xs, do: x * 2
end'''

SCENARIOS = [
    # ---- Feature: verdicts and execution -----------------------------------
    ("execution", "a clean snippet runs and reports its value",
     'IO.puts("hi")\n40 + 2', [], [ok, stdout_is("hi\n"), value_is("42"), stage_is("ok")]),
    ("execution", "a module main() runs exactly once",
     'defmodule M do\n  @moduledoc "m"\n  @doc "d"\n  @spec main() :: nil\n  @example """\n      gan> 1\n      1\n  """\n  def main(), do: IO.puts("once")\nend',
     [], [ok, stdout_is("once\n")]),
    ("execution", "a runtime crash is a run-stage failure with the error named",
     'defmodule M do\n  def main() do\n    raise "boom"\n  end\nend', [],
     [not_ok, stage_is("run"), diag_contains("boom")]),
    ("execution", "an infinite program hits the timeout, not the agent's patience",
     'defmodule M do\n  def main(), do: $time.sleep(30)\nend', [],
     [not_ok, diag_contains("timed out")]),
    ("execution", "--no-run compiles and lints without executing",
     "1 + 1", ["--no-run"], [ok, lambda d, rc: d["stdout"] is None and d["python"] is not None]),
    ("execution", "the generated Python is always inspectable",
     "x = [1, 2]\nEnum.sum(x)", [], [lambda d, rc: "sum" in d["python"]]),

    # ---- Feature: did-you-mean (AI typos are rare but costly) --------------
    ("did-you-mean", "a misspelled std function suggests the real one",
     "Enum.mpa([1], fn x -> x end)", [], [suggests("did_you_mean", "Enum.map")]),
    ("did-you-mean", "a misspelled Map function suggests candidates",
     'Map.fetc(%{"a" => 1}, "a")', [], [suggests("did_you_mean", "Map.fetch")]),
    ("did-you-mean", "a misspelled String function is caught before running",
     'String.uppcase("x")', [], [suggests("did_you_mean", "String.upcase")]),
    ("did-you-mean", "an undefined variable suggests the nearest binding",
     "defmodule M do\n  @moduledoc \"m\"\n  def f(value), do: valeu\nend", [],
     [suggests("did_you_mean", "`value`")]),
    ("did-you-mean", "a misspelled keyword suggests the spelling",
     "defmodul M do\n  def f(x), do: x\nend", [], [suggests("did_you_mean", "defmodule")]),

    # ---- Feature: migration hints (cross-language habits) ------------------
    ("migration", "return", "def f(xs):\n    return xs", [], [suggests("migration", "no `return`")]),
    ("migration", "while", "while x > 0 do\nend", [], [suggests("migration", "tail recursion")]),
    ("migration", "lambda", "f = lambda x: x", [], [suggests("migration", "`fn x ->")]),
    ("migration", "None/True/False", "x = None", [], [suggests("migration", "`nil`")]),
    ("migration", "python import", "import os", [], [suggests("migration", "pyimport")]),
    ("migration", "from-import", "from os import path", [], [suggests("migration", "no direct spelling")]),
    ("migration", "&& and ||", "a && b", [], [suggests("migration", "`and` / `or`")]),
    ("migration", "augmented assignment", "x += 1", [], [suggests("migration", "rebind")]),
    ("migration", "f-string", 'x = f"{n}"', [], [suggests("migration", "interpolation")]),
    ("migration", "switch", "switch x do\nend", [], [suggests("migration", "`case ")]),
    ("migration", "self.", "self.name", [], [suggests("migration", "no `self`")]),
    ("migration", "== nil", "x == nil", [], [suggests("migration", "is_nil")]),
    ("migration", "retired quoted pyref", '$"os.path".join("a", "b")', [],
     [suggests("migration", "$(a.b)")]),

    # ---- Feature: practice (AI laziness is the common failure) -------------
    ("practice", "missing @spec/@doc/@moduledoc is one consolidated report",
     "defmodule M do\n  def f(x), do: x\nend", [],
     [suggests("practice", "@spec on: f"), suggests("practice", "@moduledoc")]),
    ("practice", "missing @example is called out",
     "defmodule M do\n  @moduledoc \"m\"\n  @doc \"d\"\n  @spec f(term()) :: term()\n  def f(x), do: x\nend",
     [], [suggests("practice", "@example")]),
    ("practice", "concrete list() in @spec parameters wants abstract containers",
     'defmodule M do\n  @moduledoc "m"\n  @doc "d"\n  @spec f(list(integer())) :: list(integer())\n  @example """\n      gan> 1\n      1\n  """\n  def f(xs), do: xs\nend',
     [], [suggests("practice", "abstract in, concrete out")]),
    ("practice", "fn wrapping a single call wants a capture",
     "g = fn x -> to_string(x) end", [], [suggests("practice", "&f/1")]),
    ("practice", "count == 0 wants Enum.empty?",
     "Enum.count([1]) == 0", [], [suggests("practice", "Enum.empty?")]),
    ("practice", "a map+filter pipeline suggests a comprehension",
     "[1, 2] |> Enum.map(fn x -> x * 2 end) |> Enum.filter(fn x -> x > 2 end)",
     [], [suggests("practice", "comprehension")]),
    ("practice", "a bare rescue wants specific exception types",
     'try do\n  1\nrescue\n  e -> to_string(e)\nend', [], [suggests("practice", "specific types")]),
    ("practice", "repeated $mod wants a pyimport",
     "$json.dumps(1)\n$json.dumps(2)\n$json.dumps(3)", [], [suggests("practice", "pyimport json")]),

    # ---- Feature: lints flow through ---------------------------------------
    ("lints", "stack recursion warning arrives with the verdict",
     "defmodule M do\n  @moduledoc \"m\"\n  def fact(0), do: 1\n  def fact(n), do: n * fact(n - 1)\nend",
     [], [diag_contains("GEP-0019-R007")]),
    ("lints", "unused binding lint arrives",
     "defmodule M do\n  @moduledoc \"m\"\n  def f(x, y), do: x\nend", [],
     [diag_contains("GEP-0022-R002")]),
    ("lints", "the retired loop gets its migration recipe",
     "defmodule M do\n  def f() do\n    loop x = 1 do\n      break(x)\n    end\n  end\nend",
     [], [stage_is("compile"), diag_contains("GEP-0014-R007")]),

    # ---- Feature: execution edges ------------------------------------------
    ("execution-edges", "a defs-only module compiles and runs quietly",
     'defmodule M do\n  @moduledoc "m"\n  @doc "d"\n  @spec f(term()) :: term()\n  @example """\n      gan> 1\n      1\n  """\n  def f(x), do: x\nend',
     [], [ok, stdout_is("")]),
    ("execution-edges", "std functions work inside the sandbox",
     "Enum.sort([3, 1, 2]) |> Enum.sum()", [], [ok, value_is("6")]),
    ("execution-edges", "Python interop works inside the sandbox",
     "$math.sqrt(81.0)", [], [ok, value_is("9.0")]),
    ("execution-edges", "a million-frame tail recursion is fine in the sandbox",
     'defmodule M do\n  @moduledoc "m"\n  def s(n), do: s(n, 0)\n  def s(0, acc), do: acc\n  def s(n, acc), do: s(n - 1, acc + n)\n  def main(), do: IO.puts(s(1_000_000))\nend',
     [], [lambda d, rc: d["stdout"] == "500000500000\n"]),
    ("execution-edges", "a struct roundtrip runs",
     'defmodule Shopx do\n  @moduledoc "m"\n  defstruct name: nil, price: 0\n  def main() do\n    u = %Shopx{name: "kb", price: 100}\n    IO.puts("#{u.name} #{u.price}")\n  end\nend',
     [], [lambda d, rc: d["stdout"] == "kb 100\n"]),
    ("execution-edges", "a dict comprehension with into: runs",
     'for {k, v} <- [{"a", 1}, {"b", 2}], into: %{}, do: {k, v * 10}', [],
     [ok, value_is("{'a': 10, 'b': 20}")]),
    ("execution-edges", "unicode survives the round trip",
     'IO.puts("中文→émoji ✓")', [], [ok, stdout_is("中文→émoji ✓\n")]),
    ("execution-edges", "a case expression is a value",
     'case {:ok, 7} do\n  {:ok, n} -> n * 6\n  _ -> 0\nend', [], [ok, value_is("42")]),
    ("execution-edges", "IO.puts as the last statement leaves value null",
     'IO.puts("x")', [], [ok, lambda d, rc: d["value"] is None]),
    ("execution-edges", "empty input verdicts cleanly instead of crashing",
     "", [], [lambda d, rc: isinstance(d.get("ok"), bool)]),
    ("execution-edges", "closures capture by value inside the sandbox",
     "x = 1\nf = fn -> x end\nx = 2\nf.()", [], [ok, value_is("1")]),

    # ---- Feature: compile-stage precision ----------------------------------
    ("compile-errors", "recur outside tail position is a compile error",
     "defmodule M do\n  @moduledoc \"m\"\n  def f(n) do\n    recur(n - 1) + 1\n  end\nend",
     [], [not_ok, stage_is("compile"), diag_contains("GEP-0019-R005")]),
    ("compile-errors", "into: with a non-tuple body names the rule",
     'for x <- [1], into: %{}, do: x', [], [not_ok, stage_is("compile"),
     diag_contains("GEP-0020")]),
    ("compile-errors", "@spec naming a missing function is caught",
     'defmodule M do\n  @moduledoc "m"\n  @spec nope(term()) :: term()\n  def f(x), do: x\nend',
     [], [not_ok, stage_is("compile"), diag_contains("GEP-0017")]),
    ("compile-errors", "an unknown @allow target is a compile error",
     'defmodule M do\n  @moduledoc "m"\n  @allow :stack_recursoin\n  def f(x), do: x\nend',
     [], [not_ok, diag_contains("does not recognize")]),

    # ---- Feature: project-aware did-you-mean -------------------------------
    ("project", "a project module member typo is caught against real symbols",
     "Stats.maen([1, 2, 3])", [], [suggests("did_you_mean", "Stats.mean")]),
    ("project", "Keyword member typos are caught",
     "Keyword.kys([a: 1])", [], [suggests("did_you_mean", "Keyword.keys")]),
    ("project", "List member typos are caught",
     "List.flaten([1, [2]])", [], [suggests("did_you_mean", "List.flatten")]),

    # ---- Feature: negatives (no false alarms) ------------------------------
    ("negatives", "a correct member call draws no did-you-mean",
     "Enum.map([1], fn x -> x end)", [], [ok, no_suggestion_kind("did_you_mean")]),
    ("negatives", "an unknown module is skipped, not guessed at",
     "Nonexistent.thing(1)", [], [no_suggestion_kind("did_you_mean")]),
    ("negatives", "identifiers containing keywords do not trip migration",
     "returning = 1\nlambdas = 2\nreturning + lambdas", [],
     [ok, no_suggestion_kind("migration")]),
    ("negatives", "count != 0 draws no empty? hint",
     "Enum.count([1]) != 0", [], [no_suggestion_kind("practice")]),
    ("negatives", "a multi-arg fn wrapper draws no capture hint",
     "g = fn x -> to_string(x, 16) end\ng", [], [no_suggestion_kind("practice")]),
    ("negatives", "three refs across different modules draw no pyimport hint",
     "$json.dumps(1)\n$math.pi\n$os.getcwd()", [], [no_suggestion_kind("practice")]),
    ("negatives", "a typed rescue draws no bare-rescue hint",
     'try do\n  1\nrescue\n  e in $builtins.ValueError -> to_string(e)\n  e -> to_string(e)\nend',
     [], [no_suggestion_kind("practice")]),

    # ---- Feature: robustness (never a bare traceback) ----------------------
    ("robustness", "a missing input file is a JSON error, not a traceback",
     None, ["__FILE__:/nonexistent-sandbox-input.gan"],
     [lambda d, rc: "error" in d and rc == 2]),
    ("robustness", "non-UTF8 stdin is a JSON error, not a traceback",
     b"\x00\xff garbage \x01", [], [lambda d, rc: "error" in d and rc == 2]),
    ("robustness", "a 400-function module still verdicts",
     "defmodule Big do\n  @moduledoc \"m\"\n" +
     "".join(f"  def f{i}(x), do: x + {i}\n" for i in range(400)) + "end",
     ["--no-run"], [lambda d, rc: isinstance(d.get("ok"), bool)]),
    ("robustness", "a pathologically long line still verdicts",
     "x = [" + ", ".join(str(i) for i in range(2000)) + "]\nEnum.sum(x)", [],
     [ok, value_is(str(sum(range(2000))))]),
    ("robustness", "deep nesting still verdicts",
     "x = " + "(" * 60 + "1" + ")" * 60 + "\nx", [],
     [lambda d, rc: isinstance(d.get("ok"), bool)]),

    # ---- Feature: gauntlet (mass bad code never breaks the compiler) -------
    ("gauntlet", "pure python module",
     "import os\n\ndef main():\n    print(os.getcwd())\n\nif __name__ == '__main__':\n    main()",
     [], [not_ok, lambda d, rc: len(d["suggestions"]) >= 2]),
    ("gauntlet", "half-translated elixir",
     "defmodule M do\n  def run(list) do\n    list |> Enum.map(&(&1 * 2)) |> IO.inspect()\n  end\nend",
     ["--no-run"], [lambda d, rc: isinstance(d.get("ok"), bool)]),
    ("gauntlet", "javascript reflexes",
     "const x = 5;\nfunction add(a, b) { return a + b; }\nconsole.log(add(x, 2));",
     [], [not_ok, lambda d, rc: d["diagnostics"] or d["suggestions"]]),
    ("gauntlet", "unclosed do block",
     "defmodule M do\n  def f(x) do\n    x\nend", [],
     [not_ok, stage_is("compile")]),
    ("gauntlet", "stray end",
     "x = 1\nend\nend", [], [not_ok, stage_is("compile")]),
    ("gauntlet", "ruby-style def",
     "def add(a, b)\n  a + b\nend\nputs add(1, 2)", [], [not_ok]),
    ("gauntlet", "yaml pasted by mistake",
     "name: test\nsteps:\n  - run: gan build\n", [],
     [lambda d, rc: isinstance(d.get("ok"), bool)]),
    ("gauntlet", "emoji and unicode soup",
     "x = \"🚀→中文\"\nIO.puts(x)", [], [ok]),
    ("gauntlet", "deeply nested brackets",
     "x = " + "[" * 40 + "1" + "]" * 40 + "\nEnum.count(x)", [],
     [lambda d, rc: isinstance(d.get("ok"), bool)]),
    ("gauntlet", "elixir features we lack degrade with an error, not a crash",
     "defmodule M do\n  def go do\n    receive do\n      {:msg, x} -> x\n    end\n  end\nend",
     ["--no-run"], [lambda d, rc: isinstance(d.get("ok"), bool)]),
    ("gauntlet", "spawn/GenServer reflexes fail informatively",
     "pid = spawn(fn -> 1 end)\nsend(pid, :hi)", ["--no-run"],
     [lambda d, rc: isinstance(d.get("ok"), bool)]),
    ("gauntlet", "sql pasted whole",
     "SELECT * FROM users WHERE id = 1;", [], [not_ok]),
    ("gauntlet", "markdown pasted whole",
     "# Title\n\nSome *bold* claims\n\n- a list\n", [], [not_ok]),
    ("gauntlet", "null bytes in a string literal survive",
     'x = "a\\u0000b"\nString.length(x)', [],
     [lambda d, rc: isinstance(d.get("ok"), bool)]),

    # ---- Feature: silence on good code (trust) -----------------------------
    ("silence", "an idiomatic module draws zero suggestions", CLEAN_MODULE, [], [clean]),
    ("silence", "prose never triggers code patterns",
     'IO.puts("None shall return while lambda imports self.")', [],
     [ok, no_suggestion_kind("migration")]),
    ("silence", "comments never trigger code patterns",
     "# return None while lambda\n1 + 1", [], [ok, no_suggestion_kind("migration")]),
    ("silence", "@doc prose never triggers practice or migration",
     'defmodule M do\n  @moduledoc "Returns None while running"\n  @doc "d"\n  @spec f(term()) :: term()\n  @example """\n      gan> 1\n      1\n  """\n  def f(x), do: x\nend',
     [], [ok, no_suggestion_kind("migration")]),
]


def main():
    passed, failed = 0, []
    current_feature = None
    for feature, name, source, extra, checks in SCENARIOS:
        if FILTER and FILTER not in f"{feature} {name}":
            continue
        if feature != current_feature:
            print(f"\nFeature: {feature}")
            current_feature = feature
        try:
            d, rc = run_try(source, *extra)
        except Exception as e:  # noqa: BLE001
            print(f"  FAIL  Scenario: {name}\n        crashed: {e}")
            failed.append(name)
            continue
        bad = [c for c in checks if not c(d, rc)]
        if bad:
            names = ", ".join(getattr(c, "__name__", "check") for c in bad)
            print(f"  FAIL  Scenario: {name}\n        unmet: {names}")
            print(f"        verdict: ok={d['ok']} stage={d['stage']}")
            for m in all_msgs(d):
                print(f"          sugg: {m[:100]}")
            for x in d["diagnostics"]:
                print(f"          diag: {x['message'][:100]}")
            failed.append(name)
        else:
            print(f"  PASS  Scenario: {name}")
            passed += 1
    print("\n" + "=" * 40)
    print(f"{passed} passed" + (f", {len(failed)} FAILED: {failed}" if failed else " — ALL PASS"))
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
