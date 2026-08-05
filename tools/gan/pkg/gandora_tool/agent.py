"""`gan agent` — the entry point for an AI session (GEP-0026-R004):
prints one Markdown briefing holding the working loop and the
project's context pack, so a harness starts with everything and the
model queries almost nothing. Nothing is written into the project.
"""

import builtins
import collections.abc
import json
import os
import subprocess
import sys
import gandora_std.enum
import gandora_std.map
import gandora_std.string


def _gan_truthy(value):
    return value is not None and value is not False

class GanMatchError(Exception):
    pass

loop_text = "# Working in Gandora (agent briefing)\n\nGandora is an Elixir-flavored language compiling to readable Python.\nThe loop: **write -> `gan build` (fix EVERY finding) -> `gan test` -> done.**\n\n- `gan lsc check --root .` returns the verdict as JSON — a traffic light:\n  `ok: false` = red (fix errors), `clean: false` = yellow (apply every\n  suggestion), `clean: true` = green (ship).\n- Every diagnostic contains the correct spelling; apply it verbatim.\n- Details on demand: `gan lsc doc <Mod.fun|construct> [--brief]`,\n  `gan lsc pack <Mod>` for one module's full docs.\n- Rules of thumb: no return/while (last expression is the value; use\n  for/Enum or tail recursion); only false/nil are falsy; prompts are\n  `~p(raw text)`; data is maps `%{\"k\" => v}`; interop is `$math.sqrt(x)`\n  or `pyimport json`; doctest expected output is the Python repr.\n"


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
    lsc = os.path.join(os.path.dirname(sys.executable), "gan-lsc")
    if not (_gan_truthy(os.path.exists(lsc))):
        return None
    else:
        deep = gandora_std.enum.filter(args, lambda a: not (_gan_truthy(gandora_std.string.starts_with_p(a, "--"))))
        r = subprocess.run([lsc, "pack"] + (deep + ["--root", os.getcwd()]), capture_output=True, text=True, timeout=120)
        try:
            return json.loads(r.stdout)
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
    std = gandora_std.map.get(pack, "std", {})
    _gan_tmp0 = [std_line(mod, names) for _gan_for1 in std.items() if isinstance(_gan_for1, tuple) and len(_gan_for1) == 2 for (mod, names,) in [(_gan_for1[0], _gan_for1[1],)]]
    std_lines = _gan_tmp0
    project = gandora_std.map.get(pack, "project", {})
    def _gan_fn0(*_gan_args):
        match _gan_args:
            case ((mod, heads) as _gan_t2,) if isinstance(_gan_t2, tuple):
                return [f"- **{mod}**"] + gandora_std.enum.map(heads, lambda h: f"    - `{h}`")
        raise GanMatchError("no clause of _gan_fn0/1 matched " + repr(_gan_args))
    proj_lines = gandora_std.enum.flat_map(builtins.sorted(project.items()), _gan_fn0)
    lang = gandora_std.map.get(pack, "language", {})
    verdict = gandora_std.map.get(pack, "verdict", {})
    return gandora_std.enum.join(["## Standard library (every public function)", gandora_std.enum.join(std_lines, "\n"), "\n## This project", gandora_std.enum.join(proj_lines, "\n"), "\n## Language constructs (`gan lsc doc <name>`)", gandora_std.enum.join(gandora_std.map.get(lang, "constructs", []), " "), "\n## The spec type language", gandora_std.map.get(lang, "spec", ""), "\n## Current verdict", json.dumps(verdict), "\n" + gandora_std.map.get(pack, "next", "")], "\n")
