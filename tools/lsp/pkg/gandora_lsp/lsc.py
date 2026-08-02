"""gan-lsc: the Language Server Console (GEP-0015-R001A) — language
intelligence as one JSON value per query, for AI agents and shells.
"""

import builtins
import gandora_core as core
import json
import os
import pathlib
import sys
import gandora_std.enum


class GanMatchError(Exception):
    pass

usage = "gan lsc <query> — one JSON value on stdout\n\nQueries (each accepts --root <dir>, default: cwd):\n  version                     compiler/library version\n  diagnostics <file>          full-pipeline diagnostics\n  ast <file>                  quoted term of the source\n  expand <file>               quoted term after macro expansion\n  compile <file>              generated Python source (plain text)\n  resolve <module>            how a module reference resolves\n"


def main():
    args = gandora_std.enum.drop(builtins.list(sys.argv), 1)
    _gan_val0 = _take_root(args)
    match _gan_val0:
        case (root, args) as _gan_t1 if isinstance(_gan_t1, tuple):
            pass
        case _:
            raise GanMatchError("no match of right-hand side value: " + repr(_gan_val0))
    try:
        return _dispatch(args, root)
    except core.CompileError as e:
        parts = builtins.list(e.args)
        _emit({"error": gandora_std.enum.at(parts, 0), "path": gandora_std.enum.at(parts, 1), "line": gandora_std.enum.at(parts, 2), "col": gandora_std.enum.at(parts, 3)})
        return sys.exit(1)


def _dispatch(args, root):
    _gan_case2 = args
    match _gan_case2:
        case ["version", *_] as _gan_l3 if isinstance(_gan_l3, list):
            return _emit({"version": core.version()})
        case ["diagnostics", file, *_] as _gan_l4 if isinstance(_gan_l4, list):
            return _emit(core.diagnostics(_read(file), file, root))
        case ["ast", file, *_] as _gan_l5 if isinstance(_gan_l5, list):
            return _emit(core.parse(_read(file), file))
        case ["expand", file, *_] as _gan_l6 if isinstance(_gan_l6, list):
            return _emit(core.expand(_read(file), file, root))
        case ["compile", file, *_] as _gan_l7 if isinstance(_gan_l7, list):
            return print(core.compile_string(_read(file), file, root))
        case ["resolve", module, *_] as _gan_l8 if isinstance(_gan_l8, list):
            return _emit(core.resolve(root, module))
        case _:
            print(usage)
            return sys.exit(2)


def _take_root(args):
    _gan_case9 = gandora_std.enum.find_index(args, lambda a: a == "--root")
    match _gan_case9:
        case None:
            return (os.getcwd(), args)
        case i:
            root = gandora_std.enum.at(args, i + 1)
            return (root, gandora_std.enum.take(args, i) + gandora_std.enum.drop(args, i + 2))


def _read(file):
    return pathlib.Path(file).read_text()


def _emit(value):
    return print(json.dumps(value))


if __name__ == "__main__":
    main()
