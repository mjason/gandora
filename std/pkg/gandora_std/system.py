"""The host process surface, thin (GEP-0010-R011): running commands,
reading the environment, finding executables, and leaving. `cmd`
follows Elixir's contract — captured text output plus exit status —
because a subprocess whose output you did not capture is a
subprocess you cannot put in a verdict.
"""

import builtins
import collections.abc
import os
import shutil
import subprocess
import sys
import typing
import gandora_std.enum
import gandora_std.keyword


def _gan_truthy(value):
    return value is not None and value is not False

class GanMatchError(Exception):
    pass

_T_a = typing.TypeVar("_T_a")


def cmd(*_gan_args) -> tuple[str, int]:
    """Runs `bin` with `args`, capturing text output: `{stdout, exit_status}`.
Options: `cd:` runs in a directory, `stderr_to_stdout: true` merges
the streams (both Elixir's), and `timeout:` milliseconds raises the
host's timeout error on expiry (a Gandora extension, recorded per
GEP-0010-R004).

## Parameters

  - bin: The executable.
  - args: Its arguments.
  - opts: `cd:`, `stderr_to_stdout:`, `timeout:` (ms).

    >>> cmd("echo", ["hi"])
    ('hi\\n', 0)
    >>> cmd("pwd", [], [("cd", "/")])
    ('/\\n', 0)
"""
    while True:
        match _gan_args:
            case (bin, args, opts,):
                ms = gandora_std.keyword.get(opts, "timeout")
                if _gan_truthy(gandora_std.keyword.get(opts, "stderr_to_stdout", False)):
                    _gan_tmp0 = subprocess.STDOUT
                else:
                    _gan_tmp0 = None
                err = _gan_tmp0
                if (ms is None):
                    _gan_tmp1 = None
                else:
                    _gan_tmp1 = ms / 1000
                r = subprocess.run([bin] + args, stdout=subprocess.PIPE, stderr=err, text=True, cwd=gandora_std.keyword.get(opts, "cd"), timeout=_gan_tmp1)
                return (r.stdout, r.returncode)
            case (bin, args,):
                _gan_args = (bin, args, [])
                continue
        raise GanMatchError("no clause of cmd/2,3 matched " + repr(_gan_args))


def get_env(name: str, default: _T_a = None) -> str | _T_a:
    """The value of an environment variable, or `default` (nil unless given) when unset.

## Parameters

  - name: The variable name.
  - default: Returned when the variable is unset.

    >>> (get_env("GAN_STD_SURELY_UNSET") is None)
    True
    >>> get_env("GAN_STD_SURELY_UNSET", "fallback")
    'fallback'
"""
    while True:
        return os.environ.get(name, default)


def find_executable(name: str) -> str | None:
    """The absolute path of an executable on `PATH`, or nil when absent.

## Parameters

  - name: The executable name.

    >>> (find_executable("sh") is None)
    False
    >>> (find_executable("gan-no-such-tool") is None)
    True
"""
    return shutil.which(name)


def argv() -> list[str]:
    """The command-line arguments, program name excluded (as in Elixir)."""
    return gandora_std.enum.drop(builtins.list(sys.argv), 1)


def halt(status: int = 0) -> None:
    """Exits the VM with `status` (0 unless given).

## Parameters

  - status: The exit status.
"""
    while True:
        return sys.exit(status)
