"""The zero-cost half of the surface (GEP-0026): documentation, the
context pack, and the session briefing, forwarded from the tools that
already answer those questions. No model is consulted, so nothing
here can be invented.
"""

import collections.abc
import json
import sys
import gandora_std.file
import gandora_std.path
import gandora_std.string
import gandora_std.system


def _gan_truthy(value):
    return value is not None and value is not False

class GanMatchError(Exception):
    pass

tool_timeout = 180000


def argv(args: collections.abc.Sequence[str], root: str) -> list[str]:
    """The lsc arguments for one query, rooted at `root`.

## Parameters

  - args: The query and its arguments.
  - root: The project root the query runs against.

    >>> argv(["doc", "Enum.map"], "/tmp/p")
    ['doc', 'Enum.map', '--root', '/tmp/p']
"""
    return args + ["--root", root]


def lsc(args: collections.abc.Sequence[str], root: str) -> object:
    """One `gan lsc` query against `root`, decoded; nil when it fails.

## Parameters

  - args: The lsc arguments, e.g. `["doc", "Enum.map"]`.
  - root: The project root the query runs against.
"""
    bin = _tool("gan-lsc")
    if (bin is None):
        return None
    else:
        _gan_val0 = gandora_std.system.cmd(bin, argv(args, root), [("timeout", tool_timeout)])
        match _gan_val0:
            case (out, _status) as _gan_t1 if isinstance(_gan_t1, tuple):
                pass
            case _:
                raise GanMatchError("no match of right-hand side value: " + repr(_gan_val0))
        try:
            return json.loads(out)
        except Exception as _e:
            return None


def doc(target: str, root: str) -> object:
    """The docs for one target: `Mod.fun`, a bare name, or a construct.

## Parameters

  - target: What to look up.
  - root: The project root.
"""
    return lsc(["doc", target], root)


def pack(deep: collections.abc.Sequence[str], root: str) -> object:
    """The one-call context pack, with `deep` modules fully expanded.

## Parameters

  - deep: Module names whose full docs ride along.
  - root: The project root.
"""
    return lsc(["pack"] + deep, root)


def check(root: str) -> object:
    """The verdict for a whole project — the traffic light, as JSON.

## Parameters

  - root: The project root.
"""
    return lsc(["check"], root)


def briefing(root: str) -> str:
    """The `gan agent` session briefing: the working loop plus the pack.

## Parameters

  - root: The project root.
"""
    bin = _tool("gan")
    if (bin is None):
        return "gan not found — install gandora-tool"
    else:
        _gan_val2 = gandora_std.system.cmd(bin, ["agent"], [("cd", root), ("timeout", tool_timeout)])
        match _gan_val2:
            case (out, _status) as _gan_t3 if isinstance(_gan_t3, tuple):
                pass
            case _:
                raise GanMatchError("no match of right-hand side value: " + repr(_gan_val2))
        return gandora_std.string.trim(out)


def _tool(bin):
    local = gandora_std.path.join(gandora_std.path.dirname(sys.executable), bin)
    if _gan_truthy(gandora_std.file.exists_p(local)):
        return local
    else:
        return gandora_std.system.find_executable(bin)
