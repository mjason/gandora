"""gan-lsc: the Language Server Console (GEP-0015-R001A) — language
intelligence as one JSON value per query, for AI agents and shells.
Each query is declared as a `@command` annotation on its handler
(GEP-0008, via the gandora-tool `Cli` hook): the usage text and the
dispatch read one accumulated table and cannot drift.
"""

import builtins
import gandora_core as core
import json
import re
import gandora_lsp.construct_docs
import gandora_lsp.context_pack
import gandora_lsp.py_intel
import gandora_std.enum
import gandora_std.file
import gandora_std.map
import gandora_std.path
import gandora_std.string
import gandora_std.system
import gandora_tool.advisor
import gandora_tool.cli
import gandora_tool.verifier
from gandora_tool.safe import *


def _gan_truthy(value):
    return value is not None and value is not False

def _gan_and(value, then):
    return then() if _gan_truthy(value) else value

class GanMatchError(Exception):
    pass


def main() -> None:
    """Entry: parses --root, dispatches one query, emits one JSON value."""
    _gan_val0 = _take_root(gandora_std.system.argv())
    match _gan_val0:
        case (root, args) as _gan_t1 if isinstance(_gan_t1, tuple):
            pass
        case _:
            raise GanMatchError("no match of right-hand side value: " + repr(_gan_val0))
    try:
        return _run(args, root)
    except core.CompileError as e:
        parts = builtins.list(e.args)
        _emit({"error": gandora_std.enum.at(parts, 0), "path": gandora_std.enum.at(parts, 1), "line": gandora_std.enum.at(parts, 2), "col": gandora_std.enum.at(parts, 3)})
        return gandora_std.system.halt(1)
    except Exception as e:
        _emit({"error": f"{builtins.type(e).__name__}: {str(e)}"})
        return gandora_std.system.halt(2)


def _run(*_gan_args):
    match _gan_args:
        case ([cmd, *rest] as _gan_l2, root,) if isinstance(_gan_l2, list):
            def _gan_fn0(*_gan_args, cmd=cmd):
                match _gan_args:
                    case (((token, _argspec, _help) as _gan_t3, _f) as _gan_t4,) if isinstance(_gan_t3, tuple) and isinstance(_gan_t4, tuple):
                        return token == cmd
                raise GanMatchError("no clause of _gan_fn0/1 matched " + repr(_gan_args))
            hit = gandora_std.enum.find(_queries(), _gan_fn0)
            _gan_case5 = hit
            match _gan_case5:
                case None:
                    return _usage_halt()
                case (_entry, f) as _gan_t6 if isinstance(_gan_t6, tuple):
                    return f(rest, root)
                case _:
                    raise GanMatchError("no case clause matched: " + repr(_gan_case5))
        case ([] as _gan_l7, _root,) if isinstance(_gan_l7, list):
            return _usage_halt()
    raise GanMatchError("no clause of run/2 matched " + repr(_gan_args))


def _queries():
    return [(("version", "", "compiler/library version"), _version_q), (("diagnostics", "<file>", "full-pipeline diagnostics"), _diagnostics_q), (("ast", "<file>", "quoted term of the source"), _ast_q), (("expand", "<file>", "quoted term after macro expansion"), _expand_q), (("compile", "<file>", "generated Python source (plain text)"), _compile_q), (("resolve", "<module>", "how a module reference resolves"), _resolve_q), (("pack", "[<Mod> ...]", "one-call agent context: std lists, project\nsignatures, construct index, verdict summary;\nnamed modules ride along with full docs (GEP-0026-R003)"), _pack_q), (("doc", "<target> [...] [--brief]", "docs for one or many targets; --brief = one line each"), _doc_q), (("definition", "<Mod>[.<fun>]", "defining source path/line/col"), _definition_q), (("symbols", "<Mod> [...]", "every definition with rendered heads"), _symbols_q), (("references", "<Mod>.<fun>", "every project call site (+ definitions)"), _references_q), (("wsymbols", "[<query>]", "project-wide symbol search"), _wsymbols_q), (("check", "[--strict]", "the project verdict: {diagnostics, suggestions}"), _check_q), (("pydoc", "<mod.chain>", "Python docstring for a $-style reference"), _pydoc_q), (("pycomplete", "<mod.prefix>", "Python member completions"), _pycomplete_q), (("pygoto", "<mod.chain>", "Python source location"), _pygoto_q), (("pysig", "<mod.fun>", "Python call signatures"), _pysig_q)]


def _usage_halt():
    print("gan lsc <query> — one JSON value on stdout")
    print("")
    print("Queries (each accepts --root <dir>, default: cwd):")
    print(gandora_tool.cli.usage("", _queries()))
    return gandora_std.system.halt(2)


def _positional(args):
    return gandora_std.enum.filter(args, lambda a: not (_gan_truthy(gandora_std.string.starts_with_p(a, "--"))))


def _version_q(_rest, _root):
    return _emit({"version": core.version()})


def _diagnostics_q(*_gan_args):
    match _gan_args:
        case ([file, *_] as _gan_l8, root,) if isinstance(_gan_l8, list):
            return _emit(core.diagnostics(_read(file), file, root))
        case ([] as _gan_l9, _root,) if isinstance(_gan_l9, list):
            return _usage_halt()
    raise GanMatchError("no clause of diagnostics_q/2 matched " + repr(_gan_args))


def _ast_q(*_gan_args):
    match _gan_args:
        case ([file, *_] as _gan_l10, _root,) if isinstance(_gan_l10, list):
            return _emit(core.parse(_read(file), file))
        case ([] as _gan_l11, _root,) if isinstance(_gan_l11, list):
            return _usage_halt()
    raise GanMatchError("no clause of ast_q/2 matched " + repr(_gan_args))


def _expand_q(*_gan_args):
    match _gan_args:
        case ([file, *_] as _gan_l12, root,) if isinstance(_gan_l12, list):
            return _emit(core.expand(_read(file), file, root))
        case ([] as _gan_l13, _root,) if isinstance(_gan_l13, list):
            return _usage_halt()
    raise GanMatchError("no clause of expand_q/2 matched " + repr(_gan_args))


def _compile_q(*_gan_args):
    match _gan_args:
        case ([file, *_] as _gan_l14, root,) if isinstance(_gan_l14, list):
            return print(core.compile_string(_read(file), file, root))
        case ([] as _gan_l15, _root,) if isinstance(_gan_l15, list):
            return _usage_halt()
    raise GanMatchError("no clause of compile_q/2 matched " + repr(_gan_args))


def _resolve_q(*_gan_args):
    match _gan_args:
        case ([module, *_] as _gan_l16, root,) if isinstance(_gan_l16, list):
            return _emit(core.resolve(root, module))
        case ([] as _gan_l17, _root,) if isinstance(_gan_l17, list):
            return _usage_halt()
    raise GanMatchError("no clause of resolve_q/2 matched " + repr(_gan_args))


def _pack_q(rest, root):
    return _emit(gandora_lsp.context_pack.build(root, _positional(rest)))


def _doc_q(*_gan_args):
    match _gan_args:
        case ([target, *rest] as _gan_l18, root,) if isinstance(_gan_l18, list):
            brief = gandora_std.enum.member_p(rest, "--brief")
            targets = [target] + _positional(rest)
            results = gandora_std.enum.map(targets, lambda t, *, brief=brief, root=root: _doc_one(t, root, brief))
            if gandora_std.enum.count(targets) == 1:
                return _emit(gandora_std.enum.at(results, 0))
            else:
                return _emit(results)
        case ([] as _gan_l19, _root,) if isinstance(_gan_l19, list):
            return _usage_halt()
    raise GanMatchError("no clause of doc_q/2 matched " + repr(_gan_args))


def _definition_q(*_gan_args):
    match _gan_args:
        case ([target, *_] as _gan_l20, root,) if isinstance(_gan_l20, list):
            return _emit(core.definition(target, root))
        case ([] as _gan_l21, _root,) if isinstance(_gan_l21, list):
            return _usage_halt()
    raise GanMatchError("no clause of definition_q/2 matched " + repr(_gan_args))


def _symbols_q(*_gan_args):
    match _gan_args:
        case ([module, *rest] as _gan_l22, root,) if isinstance(_gan_l22, list):
            mods = [module] + _positional(rest)
            if gandora_std.enum.count(mods) == 1:
                return _emit(core.symbols(module, root))
            else:
                return _emit(gandora_std.map.new(gandora_std.enum.map(mods, lambda m, *, root=root: (m, core.symbols(m, root)))))
        case ([] as _gan_l23, _root,) if isinstance(_gan_l23, list):
            return _usage_halt()
    raise GanMatchError("no clause of symbols_q/2 matched " + repr(_gan_args))


def _references_q(*_gan_args):
    match _gan_args:
        case ([target, *_] as _gan_l24, root,) if isinstance(_gan_l24, list):
            return _emit(core.references(target, root))
        case ([] as _gan_l25, _root,) if isinstance(_gan_l25, list):
            return _usage_halt()
    raise GanMatchError("no clause of references_q/2 matched " + repr(_gan_args))


def _wsymbols_q(*_gan_args):
    match _gan_args:
        case ([query, *_] as _gan_l26, root,) if isinstance(_gan_l26, list):
            return _emit(core.wsymbols(query, root))
        case ([] as _gan_l27, root,) if isinstance(_gan_l27, list):
            return _emit(core.wsymbols("", root))
    raise GanMatchError("no clause of wsymbols_q/2 matched " + repr(_gan_args))


def _check_q(rest, root):
    diags = core.check(root)
    errors0 = gandora_std.enum.filter(diags, lambda d: gandora_std.map.get(d, "severity") == "error")
    if _gan_truthy(gandora_std.enum.empty_p(errors0)):
        cache = gandora_std.path.join(root, ".gandora/cache")
        try:
            _gan_tmp29 = gandora_tool.verifier.verify(root, cache, core.build(root, cache), gandora_std.enum.member_p(rest, "--strict"))
        except Exception as _e__gan59:
            _gan_tmp29 = []
        verification = _gan_tmp29
        _gan_tmp28 = diags + verification
    else:
        _gan_tmp28 = diags
    diags = _gan_tmp28
    try:
        _gan_tmp30 = gandora_std.enum.uniq(gandora_std.enum.map(core.wsymbols("", root), lambda sym: gandora_std.map.get(sym, "path")))
    except Exception as _e__gan60:
        _gan_tmp30 = gandora_std.path.wildcard(gandora_std.path.join(root, "src/**/*.gan"))
    files = _gan_tmp30
    test_files = gandora_std.path.wildcard(gandora_std.path.join(root, "tests/*.gan"))
    def _gan_fn1(path, *, diags=diags, root=root):
        try:
            _gan_tmp31 = _read(path)
        except Exception as _e__gan61:
            _gan_tmp31 = ""
        text = _gan_tmp31
        per_file = gandora_std.enum.filter(diags, lambda d, *, path=path: gandora_std.map.get(d, "path") == path)
        return gandora_std.enum.map(gandora_tool.advisor.analyze(text, root) + gandora_tool.advisor.lint_hints(text, per_file), lambda h, *, path=path: gandora_std.map.put(h, "path", path))
    suggestions = gandora_tool.advisor.consolidate(gandora_std.enum.flat_map(gandora_std.enum.uniq(files + test_files), _gan_fn1))
    errors = gandora_std.enum.filter(diags, lambda d: gandora_std.map.get(d, "severity") == "error")
    _emit({"ok": gandora_std.enum.empty_p(errors), "clean": _gan_and(_gan_and(gandora_std.enum.empty_p(errors), lambda: gandora_std.enum.empty_p(diags)), lambda: gandora_std.enum.empty_p(suggestions)), "diagnostics": diags, "suggestions": suggestions})
    if not (_gan_truthy(gandora_std.enum.empty_p(errors))):
        return gandora_std.system.halt(1)
    else:
        return None


def _pydoc_q(*_gan_args):
    match _gan_args:
        case ([chain, *_] as _gan_l32, root,) if isinstance(_gan_l32, list):
            return _emit(gandora_lsp.py_intel.hover_markdown(root, import_line(chain), chain))
        case ([] as _gan_l33, _root,) if isinstance(_gan_l33, list):
            return _usage_halt()
    raise GanMatchError("no clause of pydoc_q/2 matched " + repr(_gan_args))


def _pycomplete_q(*_gan_args):
    match _gan_args:
        case ([chain, *_] as _gan_l34, root,) if isinstance(_gan_l34, list):
            return _emit(gandora_lsp.py_intel.complete(root, import_line(chain), chain))
        case ([] as _gan_l35, _root,) if isinstance(_gan_l35, list):
            return _usage_halt()
    raise GanMatchError("no clause of pycomplete_q/2 matched " + repr(_gan_args))


def _pygoto_q(*_gan_args):
    match _gan_args:
        case ([chain, *_] as _gan_l36, root,) if isinstance(_gan_l36, list):
            return _emit(gandora_lsp.py_intel.goto(root, import_line(chain), chain))
        case ([] as _gan_l37, _root,) if isinstance(_gan_l37, list):
            return _usage_halt()
    raise GanMatchError("no clause of pygoto_q/2 matched " + repr(_gan_args))


def _pysig_q(*_gan_args):
    match _gan_args:
        case ([chain, *_] as _gan_l38, root,) if isinstance(_gan_l38, list):
            return _emit(gandora_lsp.py_intel.signatures(root, import_line(chain), chain))
        case ([] as _gan_l39, _root,) if isinstance(_gan_l39, list):
            return _usage_halt()
    raise GanMatchError("no clause of pysig_q/2 matched " + repr(_gan_args))


def _doc_one(target, root, brief):
    info = core.doc(target, root)
    if (info is None):
        _gan_tmp40 = _symbol_stub(target, root)
    else:
        _gan_tmp40 = info
    info = _gan_tmp40
    card = gandora_lsp.construct_docs.card(target)
    if _gan_truthy(_gan_and(not ((info is None)), lambda: brief)):
        return _brief_doc(target, info)
    elif not ((info is None)):
        return info
    elif not ((card is None)):
        return {"label": target, "construct": card}
    elif _gan_truthy(gandora_std.string.match_p(target, re.compile("^[a-z_][a-z0-9_.]*$"))):
        project = _project_doc(target, root)
        if not ((project is None)):
            return project
        else:
            md = gandora_lsp.py_intel.hover_markdown(root, import_line(target), target)
            if (md is None):
                return None
            else:
                return {"label": target, "pydoc": md}
    else:
        return None


def _symbol_stub(target, root):
    parts = target.rsplit(".", 1)
    if gandora_std.enum.count(parts) != 2:
        return None
    else:
        mod = gandora_std.enum.at(parts, 0)
        name = gandora_std.enum.at(parts, 1)
        try:
            _gan_tmp41 = core.symbols(mod, root)
        except Exception as _e__gan82:
            _gan_tmp41 = []
        syms = _gan_tmp41
        hit = gandora_std.enum.find(syms, lambda s, *, name=name: gandora_std.map.get(s, "name") == name)
        if (hit is None):
            return None
        else:
            _gan_fstr42 = gandora_std.map.get(hit, "kind")
            return {"label": target, "kind": gandora_std.map.get(hit, "kind"), "head": gandora_std.map.get(hit, "head"), "entries": {"default": f"(undocumented — add @doc above the {_gan_fstr42})"}}


def _project_doc(name, root):
    try:
        _gan_tmp43 = core.wsymbols(name, root)
    except Exception as _e__gan85:
        _gan_tmp43 = []
    syms = _gan_tmp43
    hit = gandora_std.enum.find(syms, lambda s, *, name=name: gandora_std.map.get(s, "name") == name)
    if (hit is None):
        return None
    else:
        _gan_fstr44 = gandora_std.map.get(hit, "module")
        return core.doc(f"{_gan_fstr44}.{name}", root)


def _brief_doc(target, info):
    specs = gandora_std.map.get(info, "specs", [])
    if _gan_truthy(gandora_std.enum.empty_p(specs)):
        _gan_tmp45 = target
    else:
        _gan_tmp45 = gandora_std.enum.at(specs, 0)
    head = _gan_tmp45
    prose = gandora_std.map.get(gandora_std.map.get(info, "entries", {}), "default", "")
    summary = gandora_std.enum.at(prose.split("\n"), 0)
    return {"label": target, "head": head, "summary": summary}


def import_line(chain: str) -> str:
    """The import statement that brings a dotted Python chain into scope.

## Parameters

  - chain: The dotted `$module`-style reference.

    >>> import_line("os.path.join")
    'import os'
"""
    return "import " + gandora_std.enum.at(chain.split("."), 0)


def _take_root(args):
    _gan_case46 = gandora_std.enum.find_index(args, lambda a: a == "--root")
    match _gan_case46:
        case None:
            return (gandora_std.file.cwd_bang(), args)
        case i:
            root = gandora_std.enum.at(args, i + 1)
            return (root, gandora_std.enum.take(args, i) + gandora_std.enum.drop(args, i + 2))


def _read(file):
    return gandora_std.file.read_bang(file)


def _emit(value):
    return print(json.dumps(value))


if __name__ == "__main__":
    main()
