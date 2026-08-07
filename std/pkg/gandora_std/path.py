"""Pure path arithmetic over `os.path` (GEP-0010-R011): joining,
splitting, and matching path strings. Nothing here touches the
filesystem except `wildcard/1`, which only reads directory listings.
"""

import builtins
import glob as pyglob
import os
import gandora_std.enum
import gandora_std.string


def join(left: str, right: str) -> str:
    """Joins two path segments with the host separator. More than two
segments are a pipeline: `"a" |> Path.join("b") |> Path.join("c")`.

## Parameters

  - left: The leading segment.
  - right: The trailing segment.

    >>> join("src", "main.gan")
    'src/main.gan'
    >>> join(join("a", "b"), "c.gan")
    'a/b/c.gan'
"""
    return os.path.join(left, right)


def dirname(path: str) -> str:
    """The directory part of a path.

## Parameters

  - path: The path.

    >>> dirname("a/b/c.gan")
    'a/b'
"""
    return os.path.dirname(path)


def basename(path: str) -> str:
    """The final component of a path.

## Parameters

  - path: The path.

    >>> basename("a/b/c.gan")
    'c.gan'
"""
    return os.path.basename(path)


def extname(path: str) -> str:
    """The extension of the final component, dot included; empty when there is none.

## Parameters

  - path: The path.

    >>> extname("a/b/c.gan")
    '.gan'
    >>> extname("Makefile")
    ''
"""
    return os.path.splitext(path)[1]


def expand(path: str) -> str:
    """The absolute form of a path: `~` expanded, then resolved against the
current working directory (Elixir's `Path.expand/1`).

## Parameters

  - path: The path to expand.

    >>> gandora_std.string.starts_with_p(expand("x"), "/")
    True
"""
    return os.path.abspath(os.path.expanduser(path))


def absolute_p(path: str) -> bool:
    """Whether a path is absolute. Divergence from Elixir recorded per
GEP-0010-R004: Elixir spells this `Path.type(path) == :absolute`;
Gandora prefers the predicate.

## Parameters

  - path: The path to test.

    >>> absolute_p("/etc")
    True
    >>> absolute_p("src/main.gan")
    False
"""
    return os.path.isabs(path)


def wildcard(pattern: str) -> list[str]:
    """Every path matching a glob pattern, sorted for determinism (Elixir
leaves the order unspecified). `**` matches directories recursively.

## Parameters

  - pattern: The glob pattern, e.g. `src/**/*.gan`.

    >>> wildcard("/gan_no_such_dir/**/*.gan")
    []
"""
    return gandora_std.enum.sort(builtins.list(pyglob.glob(pattern, recursive=True)))
