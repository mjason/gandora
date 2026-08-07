"""`gan agent` — the entry point for an AI session (GEP-0026-R004):
prints one Markdown briefing holding the working loop and the
project's context pack, so a harness starts with everything and the
model queries almost nothing. Nothing is written into the project.
"""

import collections.abc
import json
import sys
import gandora_std.enum
import gandora_std.file
import gandora_std.map
import gandora_std.path
import gandora_std.string
import gandora_std.system


def _gan_truthy(value):
    return value is not None and value is not False

class GanMatchError(Exception):
    pass

loop_text = "# Working in Gandora (agent briefing)\n\nGandora is an Elixir-flavored language compiling to readable Python.\nThe loop: **write -> `gan build` (fix EVERY finding) -> `gan test` -> done.**\n\n- `gan lsc check --root .` returns the verdict as JSON — a traffic light:\n  `ok: false` = red (fix errors), `clean: false` = yellow (apply every\n  suggestion), `clean: true` = green (ship).\n- Every diagnostic contains the correct spelling; apply it verbatim.\n- Details on demand: `gan lsc doc <Mod.fun|construct> [--brief]`,\n  `gan lsc pack <Mod>` for one module's full docs.\n- Rules of thumb: no return/while (last expression is the value; use\n  for/Enum or tail recursion); only false/nil are falsy; prompts are\n  `~p(raw text)`; data is maps `%{\"k\" => v}`; interop is `$math.sqrt(x)`\n  or `pyimport json`; doctest expected output is the Python repr.\n\nHouse style (the practice pass enforces these; long form:\ndocs/practices.md, digest: `gan lsc doc practices`):\n- Data flows left to right: `xs |> Enum.filter(p) |> Enum.sort()` —\n  never `f(g(h(x)))`; every std call takes its subject first.\n- A bare `for x <- xs, do: f(x)` is `Enum.map` — `for` earns its\n  place with a filter, a pattern skip, or `into:` (the one exception\n  is `await` in the body, which `fn` cannot hold).\n- `&f/1` over `fn x -> f(x) end`.\n- Outcomes are verdict tuples `{:ok, v}` / `{:error, why}`; guard\n  failure-means-no-answer queries with `safe(expr, fallback)`\n  (`import Safe`, from gandora-tool).\n- Host work goes through std `File`/`Path`/`System`\n  (`File.read!`, `Path.wildcard`, `System.cmd`), not\n  `$os`/`$pathlib`/`$subprocess`.\n- Repeated shape becomes a data table + Enum; compile-time\n  repetition becomes a `defmacro` whose expansion is what you would\n  have written by hand.\n"


def run(args: collections.abc.Sequence[str]) -> None:
    """Prints the briefing; `--json` emits the raw context pack instead.

## Parameters

  - args: CLI arguments after `agent`.
"""
    pack = _fetch_pack(args)
    if (pack is None):
        print(loop_text)
        return print("(context pack unavailable — install gandora-lsp for `gan lsc pack`)")
    elif _gan_truthy(gandora_std.enum.member_p(args, "--json")):
        return print(json.dumps(pack))
    else:
        print(loop_text)
        return print(_render(pack))


def _fetch_pack(args):
    lsc = gandora_std.path.join(gandora_std.path.dirname(sys.executable), "gan-lsc")
    if not (_gan_truthy(gandora_std.file.exists_p(lsc))):
        return None
    else:
        deep = gandora_std.enum.filter(args, lambda a: not (_gan_truthy(gandora_std.string.starts_with_p(a, "--"))))
        _gan_val0 = gandora_std.system.cmd(lsc, ["pack"] + (deep + ["--root", gandora_std.file.cwd_bang()]), [("timeout", 120000)])
        match _gan_val0:
            case (out, _status) as _gan_t1 if isinstance(_gan_t1, tuple):
                pass
            case _:
                raise GanMatchError("no match of right-hand side value: " + repr(_gan_val0))
        try:
            return json.loads(out)
        except Exception as _e:
            return None


def std_line(mod: str, names: collections.abc.Sequence[str]) -> str:
    """One std-cheat Markdown line: the module and its function names.

## Parameters

  - mod: The module name.
  - names: Its public function names.

    >>> std_line("Enum", ["map", "filter"])
    '- **Enum**: map filter'
"""
    return "- **" + (mod + ("**: " + gandora_std.enum.join(names, " ")))


def _render(pack):
    def _gan_fn0(*_gan_args):
        match _gan_args:
            case ((mod, names) as _gan_t2,) if isinstance(_gan_t2, tuple):
                return std_line(mod, names)
        raise GanMatchError("no clause of _gan_fn0/1 matched " + repr(_gan_args))
    std_lines = gandora_std.enum.map(gandora_std.map.to_list(gandora_std.map.get(pack, "std", {})), _gan_fn0)
    def _gan_fn1(*_gan_args):
        match _gan_args:
            case ((mod, heads) as _gan_t3,) if isinstance(_gan_t3, tuple):
                return [f"- **{mod}**"] + gandora_std.enum.map(heads, lambda h: f"    - `{h}`")
        raise GanMatchError("no clause of _gan_fn1/1 matched " + repr(_gan_args))
    proj_lines = gandora_std.enum.flat_map(gandora_std.enum.sort(gandora_std.map.to_list(gandora_std.map.get(pack, "project", {}))), _gan_fn1)
    lang = gandora_std.map.get(pack, "language", {})
    verdict = gandora_std.map.get(pack, "verdict", {})
    return gandora_std.enum.join(["## Standard library (every public function)", gandora_std.enum.join(std_lines, "\n"), "\n## This project", gandora_std.enum.join(proj_lines, "\n"), "\n## Language constructs (`gan lsc doc <name>`)", gandora_std.enum.join(gandora_std.map.get(lang, "constructs", []), " "), "\n## The spec type language", gandora_std.map.get(lang, "spec", ""), "\n## Current verdict", json.dumps(verdict), "\n" + gandora_std.map.get(pack, "next", "")], "\n")
