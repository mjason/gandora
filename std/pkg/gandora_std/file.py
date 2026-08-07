"""The filesystem, thin (GEP-0010-R011). Bang functions let the host
exception fly verbatim — a wrapper that translated exceptions would
be a runtime. `read/1` is the one verdict-shaped reader, for the
everyday "use the file if it is there" pattern.
"""

import builtins
import os
import pathlib
import shutil
import gandora_std.enum
import gandora_std.string


def _gan_truthy(value):
    return value is not None and value is not False


def cwd_bang() -> str:
    """The current working directory.

    >>> gandora_std.string.starts_with_p(cwd_bang(), "/")
    True
"""
    return os.getcwd()


def read(path: str) -> tuple[str, str]:
    """The verdict-shaped read: `{:ok, text}`, or `{:error, message}` with
the host's message as a string. Divergence from Elixir recorded per
GEP-0010-R004: Elixir returns a posix atom, this returns the message.

## Parameters

  - path: The file to read.

    >>> read("/dev/null")
    ('ok', '')
    >>> read("/gan_no_such_file")[0]
    'error'
"""
    try:
        return ("ok", read_bang(path))
    except builtins.OSError as e:
        return ("error", str(e))


def read_bang(path: str) -> str:
    """The text of a file; raises the host exception when it cannot be read.

## Parameters

  - path: The file to read.

    >>> read_bang("/dev/null")
    ''
"""
    return pathlib.Path(path).read_text(encoding="utf-8")


def write_bang(path: str, content: str) -> str:
    """Writes text to a file, replacing what was there; returns `:ok`.

## Parameters

  - path: The file to write.
  - content: The text to write.

    >>> write_bang("/tmp/gan_std_file_example.txt", "hi")
    'ok'
    >>> read_bang("/tmp/gan_std_file_example.txt")
    'hi'
"""
    _chars = pathlib.Path(path).write_text(content, encoding="utf-8")
    return "ok"


def exists_p(path: str) -> bool:
    """Whether a path exists at all — file, directory, or anything else.

## Parameters

  - path: The path to test.

    >>> exists_p("/")
    True
    >>> exists_p("/gan_no_such_file")
    False
"""
    return os.path.exists(path)


def dir_p(path: str) -> bool:
    """Whether a path is a directory.

## Parameters

  - path: The path to test.

    >>> dir_p("/")
    True
    >>> dir_p("/dev/null")
    False
"""
    return os.path.isdir(path)


def ls_bang(path: str) -> list[str]:
    """The entries of a directory, sorted for determinism (Elixir leaves
the order unspecified); raises when the directory cannot be listed.

## Parameters

  - path: The directory to list.

    >>> gandora_std.enum.member_p(ls_bang("/"), "tmp")
    True
"""
    return gandora_std.enum.sort(os.listdir(path))


def mkdir_p_bang(path: str) -> str:
    """Creates a directory and any missing parents; an existing directory is fine. Returns `:ok`.

## Parameters

  - path: The directory to create.

    >>> mkdir_p_bang("/tmp/gan_std_file_example_dir/deep")
    'ok'
    >>> rm_rf_bang("/tmp/gan_std_file_example_dir")
    'ok'
"""
    os.makedirs(path, exist_ok=True)
    return "ok"


def rm_rf_bang(path: str) -> str:
    """Removes a file or a directory tree; a missing target is fine.
Returns `:ok`. Divergence from Elixir recorded per GEP-0010-R004:
Elixir's `rm_rf` returns the removed paths, this does not track them.

## Parameters

  - path: The file or directory to remove.

    >>> rm_rf_bang("/tmp/gan_std_never_created")
    'ok'
"""
    if _gan_truthy(os.path.isdir(path)):
        shutil.rmtree(path, ignore_errors=True)
    else:
        pathlib.Path(path).unlink(missing_ok=True)
    return "ok"
