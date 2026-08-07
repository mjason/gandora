"""The zero-cost half of the surface (GEP-0026): documentation, the
context pack, and the session briefing, forwarded from the tools that
already answer those questions. No model is consulted, so nothing
here can be invented.
"""

import collections.abc
import json
import os
import shutil
import subprocess
import sys
import gandora_std.string


def _gan_truthy(value):
    return value is not None and value is not False


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
        r = subprocess.run([bin] + argv(args, root), capture_output=True, text=True, timeout=180)
        try:
            return json.loads(r.stdout)
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
        r = subprocess.run([bin, "agent"], capture_output=True, text=True, timeout=180, cwd=root)
        return gandora_std.string.trim(r.stdout)


def _tool(bin):
    path = os.path.join(os.path.dirname(sys.executable), bin)
    if _gan_truthy(os.path.exists(path)):
        return path
    else:
        return shutil.which(bin)
