"""Whole-module atoms for the syntax the standard library cannot
demonstrate (GEP-0028-R009). A `defmacro` has no `Enum.map` to
imitate — so the corpus carries one working module per construct, and
the test suite puts every one of them through the same verdict a
composed answer faces. An atom that stopped compiling stops being an
atom: that is what makes it safe to show a model.
"""

import collections.abc
import gandora_std.enum
import gandora_std.map
import gandora_std.string

shape_module = "defmodule Doubler do\n  @moduledoc \"One function, annotated the way every module must be.\"\n\n  @doc \"Doubles a number.\"\n  @param x, \"The number to double.\"\n  @spec double(integer()) :: integer()\n  @example \"\"\"\n      gan> Doubler.double(21)\n      42\n  \"\"\"\n  def double(x), do: x * 2\nend\n"

macro_module = "defmodule Tagger do\n  @moduledoc \"A macro that tags a value, and a function that uses it.\"\n\n  @doc \"Wraps `expr` in a tuple with `label`, at compile time.\"\n  @param label, \"The tag.\"\n  @param expr, \"The expression to tag.\"\n  @example \"\"\"\n      Tagger.tagged(\"sum\", 1 + 2)\n  \"\"\"\n  defmacro tagged(label, expr) do\n    quote do\n      {unquote(label), unquote(expr)}\n    end\n  end\n\n  @doc \"Tags a small computation, proving the macro expands.\"\n  @spec demo() :: tuple()\n  @example \"\"\"\n      gan> Tagger.demo()\n      ('sum', 3)\n  \"\"\"\n  def demo(), do: tagged(\"sum\", 1 + 2)\nend\n"

struct_module = "defmodule Point do\n  @moduledoc \"A point, as a struct.\"\n\n  defstruct x: 0, y: 0\n\n  @doc \"Moves a point along the x axis.\"\n  @param point, \"The point to move.\"\n  @param dx, \"How far to move.\"\n  @spec move(Point(), integer()) :: Point()\n  @example \"\"\"\n      gan> Point.move(%Point{x: 1, y: 2}, 3).x\n      4\n  \"\"\"\n  def move(point, dx), do: %Point{point | x: point.x + dx}\nend\n"

match_module = "defmodule Shape do\n  @moduledoc \"Pattern matching: clauses with patterns, guards, and case.\"\n\n  @doc \"The area of a shape given as a tagged tuple, matched with case.\"\n  @param shape, \"`{:rect, w, h}` or `{:circle, r}`.\"\n  @spec area(tuple()) :: number()\n  @example \"\"\"\n      gan> Shape.area({:rect, 3, 4})\n      12\n      gan> Shape.area({:circle, 1})\n      3.14\n  \"\"\"\n  def area(shape) do\n    case shape do\n      {:rect, w, h} -> w * h\n      {:circle, r} when r > 0 -> 3.14 * r * r\n      _other -> 0\n    end\n  end\n\n  @doc \"The same dispatch as clauses in the function head, with a guard.\"\n  @spec describe(tuple()) :: string()\n  @example \"\"\"\n      gan> Shape.describe({:rect, 3, 4})\n      'a rectangle'\n      gan> Shape.describe({:circle, 0})\n      'nothing'\n  \"\"\"\n  def describe({:rect, _w, _h}), do: \"a rectangle\"\n  def describe({:circle, r}) when r > 0, do: \"a circle\"\n  def describe(_other), do: \"nothing\"\nend\n"

for_module = "defmodule Squares do\n  @moduledoc \"A comprehension with a filter.\"\n\n  @doc \"The squares of the even numbers in `xs`.\"\n  @param xs, \"The numbers.\"\n  @spec evens_squared(sequence(integer())) :: list(integer())\n  @example \"\"\"\n      gan> Squares.evens_squared([1, 2, 3, 4])\n      [4, 16]\n  \"\"\"\n  def evens_squared(xs), do: for x <- xs, rem(x, 2) == 0, do: x * x\nend\n"

sigil_module = "defmodule Sigils do\n  @moduledoc \"The text sigils: word lists and regexes.\"\n\n  @doc \"The words of a ~w list joined by `sep`.\"\n  @param sep, \"The separator.\"\n  @spec joined(string()) :: string()\n  @example \"\"\"\n      gan> Sigils.joined(\"-\")\n      'red-green-blue'\n  \"\"\"\n  def joined(sep), do: Enum.join(~w(red green blue), sep)\n\n  @doc \"Whether `s` looks like a year, matched with a ~r regex.\"\n  @param s, \"The text to test.\"\n  @spec year?(string()) :: boolean()\n  @example \"\"\"\n      gan> Sigils.year?(\"2026\")\n      True\n  \"\"\"\n  def year?(s), do: not is_nil(~r/^\\d{4}$/.match(s))\nend\n"

interop_module = "defmodule Interop do\n  @moduledoc \"Reaching into Python: one-off calls and imported modules.\"\n\n  pyimport json\n\n  @doc \"The square root, via a one-off Python call.\"\n  @param x, \"A non-negative number.\"\n  @spec root(number()) :: float()\n  @example \"\"\"\n      gan> Interop.root(9)\n      3.0\n  \"\"\"\n  def root(x), do: $math.sqrt(x)\n\n  @doc \"A JSON document as a map, via an imported module.\"\n  @param text, \"The JSON text.\"\n  @spec decode(string()) :: map()\n  @example \"\"\"\n      gan> Interop.decode(~s({\"a\": 1}))\n      {'a': 1}\n  \"\"\"\n  def decode(text), do: json.loads(text)\nend\n"

task_module = "defmodule FanOut do\n  @moduledoc \"Concurrent fan-out: async def interiors, one Task.run rim.\"\n\n  @doc \"Doubles `x` after yielding to the loop — a real coroutine.\"\n  @param x, \"The number.\"\n  @spec double_soon(integer()) :: integer()\n  @example \"\"\"\n      gan> Task.run(FanOut.double_soon(21))\n      42\n  \"\"\"\n  async def double_soon(x) do\n    _pause = await Task.sleep(0)\n    x * 2\n  end\n\n  @doc \"Spawns three coroutines and joins them in input order.\"\n  @spec spread() :: list(integer())\n  @example \"\"\"\n      gan> Task.run(FanOut.spread())\n      [2, 4, 6]\n  \"\"\"\n  async def spread() do\n    tasks = for x <- [1, 2, 3], do: Task.async(double_soon(x))\n    await Task.all(tasks)\n  end\nend\n"

async_module = "defmodule Fetcher do\n  @moduledoc \"Deadlines and blocking work in the coroutine world.\"\n\n  @doc \"Joins an awaitable under `ms` milliseconds: a verdict, not an exception.\"\n  @param coro, \"The awaitable to join.\"\n  @param ms, \"The deadline in milliseconds; the task is cancelled on timeout.\"\n  @spec within(term(), integer()) :: tuple()\n  @example \"\"\"\n      gan> Task.run(Fetcher.within(Task.completed(7), 1000))\n      ('ok', 7)\n      gan> Task.run(Fetcher.within(Task.sleep(60000), 20))\n      ('error', 'timeout')\n  \"\"\"\n  async def within(coro, ms) do\n    await Task.try_await(Task.async(coro), ms)\n  end\n\n  @doc \"Blocking work overlapped with a coroutine, joined in order.\"\n  @spec mixed() :: list(integer())\n  @example \"\"\"\n      gan> Task.run(Fetcher.mixed())\n      [41, 42]\n  \"\"\"\n  async def mixed() do\n    a = Task.async(Task.blocking(fn -> 40 + 1 end))\n    b = Task.async(Task.completed(42))\n    await Task.all([a, b])\n  end\nend\n"

rescue_module = "defmodule Guarded do\n  @moduledoc \"try/rescue: absorb a Python exception, name it only if used.\"\n\n  @doc \"Parses an integer, falling back to `default` on bad input.\"\n  @param s, \"The text to parse.\"\n  @param default, \"Returned when parsing fails.\"\n  @spec parse_or(string(), integer()) :: integer()\n  @example \"\"\"\n      gan> Guarded.parse_or(\"42\", 0)\n      42\n      gan> Guarded.parse_or(\"nope\", 7)\n      7\n  \"\"\"\n  def parse_or(s, default) do\n    try do\n      $builtins.int(s)\n    rescue\n      _e in $builtins.ValueError -> default\n    end\n  end\n\n  @doc \"The error message of a crash, observed and kept.\"\n  @param f, \"A zero-argument function that may raise.\"\n  @spec message_of(fun()) :: string()\n  @example \"\"\"\n      gan> Guarded.message_of(fn -> 1 / 0 end)\n      'division by zero'\n  \"\"\"\n  def message_of(f) do\n    try do\n      to_string(f.())\n    rescue\n      e -> to_string(e)\n    end\n  end\nend\n"


def all() -> list[dict]:
    """Every syntax atom: the words that summon it, what it shows, and the
module that proves it still works.

    >>> gandora_std.enum.count(all())
    10
"""
    return [baseline()] + [{"triggers": ["macro", "defmacro", "quote", "unquote"], "why": "a macro plus a def that calls it — the def carries the runnable example", "module": macro_module}, {"triggers": ["struct", "defstruct"], "why": "a struct, its update syntax, and the module as its own type", "module": struct_module}, {"triggers": ["pattern", "match", "case", "guard", "when", "clause"], "why": "pattern matching by case and by function clauses, with a guard", "module": match_module}, {"triggers": ["comprehension", "for ", "filter", "loop"], "why": "a comprehension with a filter", "module": for_module}, {"triggers": ["sigil", "~w", "~r", "~s", "regex", "regular expression"], "why": "the ~w word list and the ~r regex sigil", "module": sigil_module}, {"triggers": ["python", "interop", "pyimport", "json", "import"], "why": "Python interop, both one-off ($math.sqrt) and imported (pyimport json)", "module": interop_module}, {"triggers": ["task", "concurrent", "parallel", "thread"], "why": "concurrent fan-out: async def authors coroutines, Task.async spawns, await Task.all joins, Task.run is the rim (GEP-0029/0030)", "module": task_module}, {"triggers": ["async", "await", "coroutine", "asyncio", "stream", "event loop"], "why": "deadlines return verdicts and cancel (Task.try_await), blocking work crosses on Task.blocking; doctests enter through Task.run (GEP-0029)", "module": async_module}, {"triggers": ["rescue", "try", "exception", "raise"], "why": "try/rescue: an unused binder is _e, a used one is e; the message is the repr", "module": rescue_module}]


def baseline() -> dict:
    """The shape every answer must take, shown rather than described: no
requirement summons it by name, and every prompt carries it.

    >>> gandora_std.map.get(baseline(), "triggers")
    []
"""
    return {"triggers": [], "why": "the annotation shape every module must have", "module": shape_module}


def matching(requirement: str) -> list[dict]:
    """The atoms a requirement summons, by the words it uses.

## Parameters

  - requirement: The requirement text.

    >>> gandora_std.enum.count(matching("a macro that logs a value"))
    1
"""
    low = gandora_std.string.downcase(requirement)
    return gandora_std.enum.filter(all(), lambda a, *, low=low: _summoned_p(a, low))


def _summoned_p(atom, low):
    return gandora_std.enum.count(gandora_std.enum.filter(gandora_std.map.get(atom, "triggers", []), lambda t, *, low=low: gandora_std.string.contains_p(low, t))) > 0


def block(atom: collections.abc.Mapping[str, object]) -> str:
    """The prompt block for one syntax atom.

## Parameters

  - atom: One corpus entry.

    >>> block({"why": "w", "module": "m"})
    '## A verified module: w\\n\\nm'
"""
    return "## A verified module: " + (gandora_std.map.get(atom, "why", "") + ("\n\n" + gandora_std.string.trim(gandora_std.map.get(atom, "module", ""))))
