"""The composer (GEP-0028-R006/R007/R008): one requirement in, one
**verified module** out. The model never recalls the language — it
composes from atoms and the context pack — and never has the last
word: the verdict does. A draft that cannot go green is returned as
a failure carrying its findings, never as confident prose.
"""

import collections.abc
import functools
import os
import pathlib
import pydantic_ai
import pydantic_ai.models.openai as oai
import pydantic_ai.providers.deepseek as dsp
import gandora_mcp.atoms
import gandora_mcp.corpus
import gandora_mcp.intel
import gandora_mcp.sandbox
import gandora_std.enum
import gandora_std.map
import gandora_std.string
import gandora_std.task


def _gan_truthy(value):
    return value is not None and value is not False

def _gan_and(value, then):
    return then() if _gan_truthy(value) else value

def _gan_or(value, then):
    return value if _gan_truthy(value) else then()

class GanMatchError(Exception):
    pass

efforts = ["none", "low", "low", "low"]

system = "You write Gandora, an Elixir-flavored language that compiles to Python.\n\nAnswer in exactly two parts, in this order and with these markers:\n\n  EXPLANATION:\n  Two to five sentences saying what the feature is and how it reads.\n  Prose only — every line of code you want to show belongs in the\n  module below, because only the module is compiled and run.\n\n  MODULE:\n  One complete module, in the shape of the verified modules shown to\n  you, and nothing else. No fences.\n\nRules that are not negotiable:\n- Every public def carries @doc, @param per parameter, @spec, @example.\n- @doc is prose only; examples live in @example, never inside @doc.\n- An @example line is `gan> Module.fun(args)` and the next line is the\n  expected value as a Python repr: 'hi' for a string, ('ok', 21) for a\n  tuple, {'k': 1} for a map, True or False for booleans.\n- A call whose value is nil prints NOTHING. Write the `gan>` line and\n  then go straight to the next line or the closing quotes — never\n  write nil or None as an expected value; it can only fail.\n- The example must be true. It is executed, and a wrong expected value\n  fails the answer.\n- No return, no while: the last expression is the value. Only false\n  and nil are falsy.\n- Data is maps: %{\"k\" => v}. Interop is $math.sqrt(x) or pyimport json.\n- @spec takes abstract parameters and returns concrete: never write\n  list()/map() as a PARAMETER type — use sequence(t), iterable(t),\n  mapping(k, v) there — and return the concrete list(t)/map().\n- A doctest expression is one simple expression: no assignments, no\n  multi-line setup. Build what you need inline.\n- There is no Integer or Float module: to_string/1, div/2, rem/2 are\n  built-ins, and parsing is $builtins.int(s) / $builtins.float(s).\n- A macro's @example is displayed, not executed. If you write a\n  defmacro, also write a plain def in the same module that calls it,\n  and put the runnable @example on that def — otherwise nothing about\n  your module can be proven.\n\nRules the compiler keeps having to teach:\n- @param must name a real parameter. A def whose clauses match patterns\n  (`def area({:rect, w, h})`) has no parameter name to document, so\n  write no @param at all for it.\n- A tuple TYPE is `tuple(t, t)`, never a tuple literal: the `{:ok, x}`\n  shape is spelled `tuple(atom(), x)` in a @spec (GEP-0017-R002).\n- A bound variable that goes unused is an ERROR. Prefix it with `_`.\n- Python exceptions are named through their module:\n  `rescue e in $builtins.ValueError -> ...` (GEP-0014).\n- Only the standard library and the Python standard library are\n  installed. Never demonstrate numpy, pandas, or requests — the\n  example is really executed and the import will fail.\n\nConcurrency (GEP-0029/0030):\n- `async def f(x) do ... end` compiles to Python's async def, and\n  `await expr` to Python's await — bare, no deadline. await is legal\n  only inside an async def body, never inside fn.\n- Task is the library over asyncio: Task.run(coro) enters the\n  coroutine world from sync code (main stays sync); Task.async(coro)\n  spawns inside an async body; `await Task.all(tasks)` joins in input\n  order; Task.try_await(task, ms) returns ('ok', v) /\n  ('error', 'timeout') / ('error', e) and cancels on timeout;\n  Task.blocking(fn -> work() end) runs a blocking function on a\n  thread. Timeouts are milliseconds.\n- The runnable @example of an async def goes through the rim:\n  `gan> Task.run(M.fun(args))`. Spawning needs a running loop —\n  never call Task.async at the rim.\n\nElixir habits that are errors here:\n- A type is a call and the module IS the type: String(), never\n  String.t(). Builtins are lowercase calls: string(), integer(),\n  float(), boolean(), map(), term(), list(t), sequence(t).\n- `~r/re/` is the regular-expression sigil; `~p(...)` is raw prompt\n  text, not a regex.\n- `fn x -> f(x) end` around a single call is written `&f/1`.\n"

triggers = {"macro": ["defmacro", "quote", "unquote"], "struct": ["defstruct", "defattr"], "test": ["test"], "type": ["type", "spec"], "json": ["json", "interop"], "python": ["pyimport", "interop"], "prompt": ["prompt"], "template": ["format"]}

model_timeout = 180000


async def example(requirement: str, root: str) -> dict:
    """A verified module answering `requirement`, or an honest failure. The
returned map carries `ok`, the `module` source, its `verdict`, the
`rounds` it took, and the `atoms` it was composed from.

## Parameters

  - requirement: What the example must demonstrate.
  - root: The project the atoms and the context pack come from.
"""
    key = _api_key(root)
    if (key is None):
        return {"ok": False, "why": "no GAN_API_KEY in the environment or .env", "explanation": "", "module": "", "verdict": {}, "rounds": 0, "atoms": []}
    else:
        ground_task = gandora_std.task.async__kw(gandora_std.task.blocking(lambda *, root=root: _grounding(root)))
        cards_task = gandora_std.task.async__kw(_constructs(requirement, root))
        atoms = gandora_mcp.atoms.search(root, requirement, 6)
        _gan_val0 = (await gandora_std.task.all([ground_task, cards_task]))
        match _gan_val0:
            case [notes, cards] as _gan_l1 if isinstance(_gan_l1, list):
                pass
            case _:
                raise GanMatchError("no match of right-hand side value: " + repr(_gan_val0))
        ground = {"atoms": atoms, "notes": notes, "cards": cards, "root": root}
        return (await _attempt(requirement, key, ground, efforts, "", 0))


def answer(text: str) -> tuple:
    """The two halves of an answer: the prose explanation and the module.
Code never rides in the prose — only the module is compiled and run,
so anything fenced off in the explanation would be an unverified
claim wearing the same clothes as a verified one.

## Parameters

  - text: The raw model output.

    >>> answer("EXPLANATION:\\nIt doubles.\\nMODULE:\\ndefmodule M do\\nend")
    ('It doubles.', 'defmodule M do\\nend')
"""
    parts = gandora_std.string.split_on(text, "MODULE:")
    if gandora_std.enum.count(parts) > 1:
        return (_prose(gandora_std.enum.at(parts, 0)), strip_fences(gandora_std.enum.at(parts, 1)))
    else:
        return ("", strip_fences(text))


def _prose(text):
    return gandora_std.string.trim(gandora_std.enum.join(gandora_std.enum.filter(gandora_std.string.split_on(gandora_std.string.trim(text), "\n"), lambda l: not (_gan_truthy(_fenced_p(l)))), "\n"))


def _fenced_p(line):
    trimmed = gandora_std.string.trim(line)
    return _gan_or(gandora_std.string.starts_with_p(trimmed, "```"), lambda: gandora_std.string.starts_with_p(trimmed, "EXPLANATION:"))


def strip_fences(text: str) -> str:
    """The module source stripped of the fences a model wraps it in.

## Parameters

  - text: The raw model output.

    >>> strip_fences("```gandora\\ndefmodule M do\\nend\\n```")
    'defmodule M do\\nend'
"""
    return gandora_std.string.trim(gandora_std.enum.join(gandora_std.enum.filter(gandora_std.string.split_on(gandora_std.string.trim(text), "\n"), lambda l: not (_gan_truthy(gandora_std.string.starts_with_p(gandora_std.string.trim(l), "```")))), "\n"))


def green_p(verdict: collections.abc.Mapping[str, object]) -> bool:
    """Whether a verdict is green: it compiles, it is idiomatic, it ran.

## Parameters

  - verdict: A Sandbox verdict.

    >>> green_p({"ok": True, "clean": True, "doctests": {"passed": True}})
    True
"""
    return _gan_and(_gan_and(gandora_std.map.get(verdict, "ok", False), lambda: gandora_std.map.get(verdict, "clean", False)), lambda: gandora_std.map.get(gandora_std.map.get(verdict, "doctests", {}), "passed", False))


async def _attempt(requirement, key, ground, efforts, findings, round):
    while True:
        atoms = gandora_std.map.get(ground, "atoms")
        if _gan_truthy(gandora_std.enum.empty_p(efforts)):
            return {"ok": False, "why": "no draft reached a clean verdict", "explanation": "", "module": "", "verdict": {}, "rounds": round, "findings": findings, "atoms": atoms}
        else:
            raw = (await _ask(_prompt(requirement, ground, findings), gandora_std.enum.at(efforts, 0), key))
            _gan_val2 = answer(raw)
            match _gan_val2:
                case (prose, draft) as _gan_t3 if isinstance(_gan_t3, tuple):
                    pass
                case _:
                    raise GanMatchError("no match of right-hand side value: " + repr(_gan_val2))
            if gandora_std.string.trim(draft) == "":
                _gan_tmp4 = _no_draft()
            else:
                _gan_tmp4 = (await gandora_std.task.blocking(lambda *, draft=draft, ground=ground: gandora_mcp.sandbox.verdict(draft, gandora_std.map.get(ground, "root"))))
            verdict = _gan_tmp4
            left = gandora_std.enum.drop(efforts, 1)
            if _gan_truthy(_gan_and(green_p(verdict), lambda: gandora_std.string.trim(prose) != "")):
                return {"ok": True, "explanation": prose, "module": draft, "verdict": verdict, "rounds": round + 1, "atoms": atoms}
            elif _gan_truthy(_gan_and(green_p(verdict), lambda: gandora_std.enum.empty_p(left))):
                return {"ok": True, "explanation": prose, "module": draft, "verdict": verdict, "rounds": round + 1, "atoms": atoms}
            elif _gan_truthy(green_p(verdict)):
                requirement, key, ground, efforts, findings, round = requirement, key, ground, left, "Your module was accepted but your answer had no EXPLANATION: section. Send both parts.", round + 1
                continue
            else:
                requirement, key, ground, efforts, findings, round = requirement, key, ground, left, _report(verdict), round + 1
                continue


def _report(verdict):
    errors = gandora_std.enum.map(gandora_std.map.get(verdict, "diagnostics", []), lambda d: "error: " + str(gandora_std.map.get(d, "message", "")))
    hints = gandora_std.enum.map(gandora_std.map.get(verdict, "suggestions", []), lambda s: "practice: " + str(gandora_std.map.get(s, "message", "")))
    doctest = gandora_std.map.get(gandora_std.map.get(verdict, "doctests", {}), "output", "")
    return gandora_std.enum.join(errors + (hints + [doctest]), "\n")


def _prompt(requirement, ground, findings):
    notes = gandora_std.map.get(ground, "notes")
    cards = gandora_std.map.get(ground, "cards")
    pieces = gandora_std.enum.join(gandora_std.enum.map(gandora_std.map.get(ground, "atoms"), _atom_block), "\n")
    modules = gandora_std.enum.join(gandora_std.enum.map(_shown(requirement), gandora_mcp.corpus.block), "\n\n")
    again = _retry(findings)
    return f"{notes}\n{cards}\nVerified examples of the pieces you may need:\n\n{pieces}\n\n{modules}\n\nRequirement: {requirement}\n{again}\n"


def _shown(requirement):
    return [gandora_mcp.corpus.baseline()] + gandora_mcp.corpus.matching(requirement)


def _retry(findings):
    if gandora_std.string.trim(findings) == "":
        return ""
    else:
        return f"\nYour previous answer was rejected by the compiler and the doctest\nrunner. Fix exactly these:\n\n{findings}\n"


async def _constructs(requirement, root):
    low = gandora_std.string.downcase(requirement)
    _gan_tmp5 = [names for _gan_for6 in triggers.items() if isinstance(_gan_for6, tuple) and len(_gan_for6) == 2 for (word, names,) in [(_gan_for6[0], _gan_for6[1],)] if _gan_truthy(gandora_std.string.contains_p(low, word))]
    wanted = _gan_tmp5
    cards = (await gandora_std.task.async_stream(gandora_std.enum.uniq(gandora_std.enum.concat(wanted)), lambda n, *, root=root: gandora_std.task.blocking(lambda *, root=root: _card(n, root)), 4))
    kept = gandora_std.enum.filter(cards, lambda c: c != "")
    if _gan_truthy(gandora_std.enum.empty_p(kept)):
        return ""
    else:
        _gan_fstr7 = gandora_std.enum.join(kept, "\n")
        return f"The constructs this requirement names:\n{_gan_fstr7}\n"


def _card(name, root):
    info = gandora_mcp.intel.doc(name, root)
    if (info is None):
        return ""
    else:
        _gan_fstr8 = str(gandora_std.map.get(info, "construct", ""))
        return f"- {name}: {_gan_fstr8}"


def _grounding(root):
    pack = gandora_mcp.intel.pack([], root)
    if (pack is None):
        return ""
    else:
        notes = gandora_std.enum.join(gandora_std.map.get(gandora_std.map.get(pack, "language", {}), "notes", []), "\n")
        _gan_tmp9 = ["- " + (mod + (": " + gandora_std.enum.join(names, " "))) for _gan_for10 in gandora_std.map.get(pack, "std", {}).items() if isinstance(_gan_for10, tuple) and len(_gan_for10) == 2 for (mod, names,) in [(_gan_for10[0], _gan_for10[1],)]]
        lines = _gan_tmp9
        _gan_fstr11 = gandora_std.enum.join(lines, "\n")
        return f"Language notes:\n{notes}\n\nThe standard library — these functions exist, no others:\n{_gan_fstr11}\n"


def _atom_block(atom):
    _gan_fstr12 = gandora_std.map.get(atom, "target", "")
    _gan_fstr13 = gandora_std.map.get(atom, "spec", "")
    _gan_fstr14 = gandora_std.map.get(atom, "doc", "")
    _gan_fstr15 = gandora_std.map.get(atom, "example", "")
    return f"## {_gan_fstr12}\n{_gan_fstr13}\n{_gan_fstr14}\n{_gan_fstr15}\n"


async def _request(text, effort, key):
    result = (await _agent_for(effort, key).run(text))
    return result.output


async def _ask(text, effort, key):
    t = gandora_std.task.async__kw(_request(text, effort, key))
    _gan_case16 = (await gandora_std.task.try_await(t, model_timeout))
    match _gan_case16:
        case ("ok", out) as _gan_t17 if isinstance(_gan_t17, tuple):
            return out
        case ("error", _why) as _gan_t18 if isinstance(_gan_t18, tuple):
            return ""
        case _:
            raise GanMatchError("no case clause matched: " + repr(_gan_case16))


@functools.cache
def _agent_for(effort, key):
    model = oai.OpenAIResponsesModel(_model_name(), provider=dsp.DeepSeekProvider(api_key=key))
    settings = oai.OpenAIResponsesModelSettings(openai_reasoning_effort=effort, max_tokens=4000)
    return pydantic_ai.Agent(model, model_settings=settings, instructions=system)


def _no_draft():
    return {"ok": False, "clean": False, "diagnostics": [], "suggestions": [], "why": "the model call failed or returned nothing", "doctests": gandora_mcp.sandbox.skipped("no draft")}


def _model_name():
    name = os.getenv("GAN_MODEL")
    if (name is None):
        return "deepseek-v4-flash"
    else:
        return name


def _api_key(root):
    env = os.getenv("GAN_API_KEY")
    if not ((env is None)):
        return env
    else:
        return _dotenv(os.path.join(root, ".env"))


def _dotenv(path):
    if not (_gan_truthy(os.path.exists(path))):
        return None
    else:
        lines = gandora_std.string.split_on(pathlib.Path(path).read_text(), "\n")
        hit = gandora_std.enum.find(lines, lambda l: gandora_std.string.starts_with_p(gandora_std.string.trim(l), "GAN_API_KEY="))
        if (hit is None):
            return None
        else:
            return gandora_std.string.trim(gandora_std.enum.at(gandora_std.string.split_on(hit, "="), 1))
