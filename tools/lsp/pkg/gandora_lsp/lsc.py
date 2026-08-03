"""gan-lsc: the Language Server Console (GEP-0015-R001A) — language
intelligence as one JSON value per query, for AI agents and shells.
"""

import builtins
import gandora_core as core
import json
import os
import pathlib
import sys
import gandora_lsp.py_intel
import gandora_lsp.sandbox
import gandora_std.enum
import gandora_std.map


def _gan_truthy(value):
    return value is not None and value is not False

class GanMatchError(Exception):
    pass

usage = "gan lsc <query> — one JSON value on stdout\n\nQueries (each accepts --root <dir>, default: cwd):\n  version                     compiler/library version\n  diagnostics <file>          full-pipeline diagnostics\n  ast <file>                  quoted term of the source\n  expand <file>               quoted term after macro expansion\n  compile <file>              generated Python source (plain text)\n  resolve <module>            how a module reference resolves\n  doc <Mod>[.<fun>]           docs: specs, signatures, prose, examples\n  definition <Mod>[.<fun>]    defining source path/line/col\n  symbols <Mod>               every definition with rendered heads\n  references <Mod>.<fun>      every project call site (+ definitions)\n  wsymbols [<query>]          project-wide symbol search\n  check                       whole-project diagnostics, lints included\n  try <file|-> [--no-run]     sandbox: compile, lint, suggest, execute\n  pydoc <mod.chain>           Python docstring for a $-style reference\n  pycomplete <mod.prefix>     Python member completions\n  pygoto <mod.chain>          Python source location\n  pysig <mod.fun>             Python call signatures\n"


def main() -> None:
    """Entry: parses --root, dispatches one query, emits one JSON value."""
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
        case ["doc", target, *_] as _gan_l9 if isinstance(_gan_l9, list):
            return _emit(core.doc(target, root))
        case ["definition", target, *_] as _gan_l10 if isinstance(_gan_l10, list):
            return _emit(core.definition(target, root))
        case ["symbols", module, *_] as _gan_l11 if isinstance(_gan_l11, list):
            return _emit(core.symbols(module, root))
        case ["references", target, *_] as _gan_l12 if isinstance(_gan_l12, list):
            return _emit(core.references(target, root))
        case ["wsymbols", query, *_] as _gan_l13 if isinstance(_gan_l13, list):
            return _emit(core.wsymbols(query, root))
        case ["wsymbols"] as _gan_l14 if isinstance(_gan_l14, list):
            return _emit(core.wsymbols("", root))
        case ["check", *_] as _gan_l15 if isinstance(_gan_l15, list):
            return _emit(core.check(root))
        case ["try"] as _gan_l16 if isinstance(_gan_l16, list):
            return print(gandora_lsp.sandbox.help())
        case ["try", "--help", *_] as _gan_l17 if isinstance(_gan_l17, list):
            return print(gandora_lsp.sandbox.help())
        case ["try", target, *rest] as _gan_l18 if isinstance(_gan_l18, list):
            if target == "-":
                _gan_tmp19 = sys.stdin.read()
            else:
                _gan_tmp19 = _read(target)
            source = _gan_tmp19
            verdict = gandora_lsp.sandbox.try_source(source, root, not (_gan_truthy(gandora_std.enum.member_p(rest, "--no-run"))))
            _emit(verdict)
            if not (_gan_truthy(gandora_std.map.get(verdict, "ok"))):
                return sys.exit(1)
            else:
                return None
        case ["pydoc", chain, *_] as _gan_l20 if isinstance(_gan_l20, list):
            return _emit(gandora_lsp.py_intel.hover_markdown(root, _import_line(chain), chain))
        case ["pycomplete", chain, *_] as _gan_l21 if isinstance(_gan_l21, list):
            return _emit(gandora_lsp.py_intel.complete(root, _import_line(chain), chain))
        case ["pygoto", chain, *_] as _gan_l22 if isinstance(_gan_l22, list):
            return _emit(gandora_lsp.py_intel.goto(root, _import_line(chain), chain))
        case ["pysig", chain, *_] as _gan_l23 if isinstance(_gan_l23, list):
            return _emit(gandora_lsp.py_intel.signatures(root, _import_line(chain), chain))
        case _:
            print(usage)
            return sys.exit(2)


def _import_line(chain):
    return "import " + gandora_std.enum.at(chain.split("."), 0)


def _take_root(args):
    _gan_case24 = gandora_std.enum.find_index(args, lambda a: a == "--root")
    match _gan_case24:
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
