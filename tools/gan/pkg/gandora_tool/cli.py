"""The declarative command surface (GEP-0008): a CLI declares each
command as an annotation on the handler that implements it —

    @command {"build", "[--strict]", "the verdict + compile"}
    defp build_cmd(rest), do: ...

— and the `on_command` hook registers `{entry, &handler/1}` into the
consumer's accumulated `@command_table`. Usage text and dispatch both
read that one table: what the help prints is what the dispatcher
runs, and neither can drift from the other. A `nil` help hides an
entry from usage (aliases) without unhooking it from dispatch.
"""

import collections.abc
import gandora_std.enum
import gandora_std.string


class GanMatchError(Exception):
    pass

column = 26


def usage(prefix: str, table: collections.abc.Sequence[tuple]) -> str:
    """The usage body generated from a command table: one aligned line per
visible entry, continuation help lines indented to the same column.

## Parameters

  - prefix: The program name printed before each token.
  - table: The accumulated `@command_table`.

    >>> usage("gan", [(("go", "<file>", "runs it"), None)])
    '  gan go <file>           runs it'
"""
    _gan_tmp0 = [_entry_lines(prefix, entry) for _gan_for1 in table if isinstance(_gan_for1, tuple) and len(_gan_for1) == 2 for (entry,) in [(_gan_for1[0],)] if not ((entry[2] is None))]
    lines = _gan_tmp0
    return gandora_std.enum.join(lines, "\n")


def _entry_lines(*_gan_args):
    match _gan_args:
        case (prefix, (token, argspec, help) as _gan_t2,) if isinstance(_gan_t2, tuple):
            left = gandora_std.enum.join(gandora_std.enum.filter([prefix, token, argspec], lambda part: part != ""), " ")
            lead = "  " + left
            if gandora_std.string.length(lead) >= (column - 1):
                _gan_tmp3 = lead + "  "
            else:
                _gan_tmp3 = gandora_std.string.pad_trailing(lead, column)
            padded = _gan_tmp3
            _gan_val4 = gandora_std.string.split_on(help, "\n")
            match _gan_val4:
                case [first, *more] as _gan_l5 if isinstance(_gan_l5, list):
                    pass
                case _:
                    raise GanMatchError("no match of right-hand side value: " + repr(_gan_val4))
            rest = gandora_std.enum.map(more, lambda l: gandora_std.string.duplicate(" ", column) + l)
            return gandora_std.enum.join([padded + first] + rest, "\n")
    raise GanMatchError("no clause of entry_lines/2 matched " + repr(_gan_args))


def dispatch(table: collections.abc.Sequence[tuple], cmd: str, rest: collections.abc.Sequence[str], fallback: collections.abc.Callable) -> object:
    """Runs the table entry for `cmd` with `rest`; an unknown command goes
to `fallback.(cmd, rest)` — the plugin/delegation seam.

## Parameters

  - table: The accumulated `@command_table`.
  - cmd: The subcommand token.
  - rest: The arguments after the token.
  - fallback: Called with (cmd, rest) when no entry matches.

    >>> dispatch([(("go", "", "g"), lambda r: ("ran", r))], "go", ["x"], None)
    ('ran', ['x'])
"""
    def _gan_fn0(*_gan_args, cmd=cmd):
        match _gan_args:
            case (((token, _argspec, _help) as _gan_t6, _f) as _gan_t7,) if isinstance(_gan_t6, tuple) and isinstance(_gan_t7, tuple):
                return token == cmd
        raise GanMatchError("no clause of _gan_fn0/1 matched " + repr(_gan_args))
    hit = gandora_std.enum.find(table, _gan_fn0)
    _gan_case8 = hit
    match _gan_case8:
        case None:
            return fallback(cmd, rest)
        case (_entry, f) as _gan_t9 if isinstance(_gan_t9, tuple):
            return f(rest)
        case _:
            raise GanMatchError("no case clause matched: " + repr(_gan_case8))
