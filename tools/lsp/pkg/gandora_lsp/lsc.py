"""gan-lsc: the Language Server Console (GEP-0015-R001A) — language
intelligence as one JSON value per query, for AI agents and shells.
"""

import builtins
import gandora_core as core
import json
import os
import pathlib
import re
import sys
import gandora_lsp.construct_docs
import gandora_lsp.py_intel
import gandora_std.enum
import gandora_std.map
import gandora_std.string
import gandora_tool.advisor


def _gan_truthy(value):
    return value is not None and value is not False

def _gan_and(value, then):
    return then() if _gan_truthy(value) else value

class GanMatchError(Exception):
    pass

usage = "gan lsc <query> — one JSON value on stdout\n\nQueries (each accepts --root <dir>, default: cwd):\n  version                     compiler/library version\n  diagnostics <file>          full-pipeline diagnostics\n  ast <file>                  quoted term of the source\n  expand <file>               quoted term after macro expansion\n  compile <file>              generated Python source (plain text)\n  resolve <module>            how a module reference resolves\n  doc <Mod>[.<fun>]           docs: specs, signatures, prose, examples\n  definition <Mod>[.<fun>]    defining source path/line/col\n  symbols <Mod>               every definition with rendered heads\n  references <Mod>.<fun>      every project call site (+ definitions)\n  wsymbols [<query>]          project-wide symbol search\n  check                       the project verdict: {diagnostics, suggestions}\n  pydoc <mod.chain>           Python docstring for a $-style reference\n  pycomplete <mod.prefix>     Python member completions\n  pygoto <mod.chain>          Python source location\n  pysig <mod.fun>             Python call signatures\n"


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
    except Exception as e:
        _emit({"error": f"{builtins.type(e).__name__}: {str(e)}"})
        return sys.exit(2)


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
            info = core.doc(target, root)
            card = gandora_lsp.construct_docs.card(target)
            if not ((info is None)):
                return _emit(info)
            elif not ((card is None)):
                return _emit({"label": target, "construct": card})
            elif _gan_truthy(gandora_std.string.match_p(target, re.compile("^[a-z_][a-z0-9_.]*$"))):
                md = gandora_lsp.py_intel.hover_markdown(root, import_line(target), target)
                if (md is None):
                    return _emit(None)
                else:
                    return _emit({"label": target, "pydoc": md})
            else:
                return _emit(info)
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
            diags = core.check(root)
            try:
                _gan_tmp16 = gandora_std.enum.uniq(gandora_std.enum.map(core.wsymbols("", root), lambda sym: gandora_std.map.get(sym, "path")))
            except Exception as _e:
                _gan_tmp16 = gandora_std.enum.map(builtins.list(pathlib.Path(root).glob("src/**/*.gan")), str)
            files = _gan_tmp16
            test_files = gandora_std.enum.map(builtins.list(pathlib.Path(root).glob("tests/*.gan")), str)
            def _gan_fn0(path, *, diags=diags, root=root):
                try:
                    _gan_tmp17 = _read(path)
                except Exception as _e:
                    _gan_tmp17 = ""
                text = _gan_tmp17
                per_file = gandora_std.enum.filter(diags, lambda d, *, path=path: gandora_std.map.get(d, "path") == path)
                return gandora_std.enum.map(gandora_tool.advisor.analyze(text, root) + gandora_tool.advisor.lint_hints(text, per_file), lambda h, *, path=path: gandora_std.map.put(h, "path", path))
            suggestions = gandora_tool.advisor.consolidate(gandora_std.enum.flat_map(gandora_std.enum.uniq(files + test_files), _gan_fn0))
            errors = gandora_std.enum.filter(diags, lambda d: gandora_std.map.get(d, "severity") == "error")
            _emit({"ok": gandora_std.enum.empty_p(errors), "clean": _gan_and(_gan_and(gandora_std.enum.empty_p(errors), lambda: gandora_std.enum.empty_p(diags)), lambda: gandora_std.enum.empty_p(suggestions)), "diagnostics": diags, "suggestions": suggestions})
            if not (_gan_truthy(gandora_std.enum.empty_p(errors))):
                return sys.exit(1)
            else:
                return None
        case ["pydoc", chain, *_] as _gan_l18 if isinstance(_gan_l18, list):
            return _emit(gandora_lsp.py_intel.hover_markdown(root, import_line(chain), chain))
        case ["pycomplete", chain, *_] as _gan_l19 if isinstance(_gan_l19, list):
            return _emit(gandora_lsp.py_intel.complete(root, import_line(chain), chain))
        case ["pygoto", chain, *_] as _gan_l20 if isinstance(_gan_l20, list):
            return _emit(gandora_lsp.py_intel.goto(root, import_line(chain), chain))
        case ["pysig", chain, *_] as _gan_l21 if isinstance(_gan_l21, list):
            return _emit(gandora_lsp.py_intel.signatures(root, import_line(chain), chain))
        case _:
            print(usage)
            return sys.exit(2)


def import_line(chain: str) -> str:
    """The import statement that brings a dotted Python chain into scope.

## Parameters

  - chain: The dotted `$module`-style reference.

    >>> import_line("os.path.join")
    'import os'
"""
    return "import " + gandora_std.enum.at(chain.split("."), 0)


def _take_root(args):
    _gan_case22 = gandora_std.enum.find_index(args, lambda a: a == "--root")
    match _gan_case22:
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
