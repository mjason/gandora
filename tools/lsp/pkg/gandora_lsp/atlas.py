"""The code map and its corpus (GEP-0031): where everything lives — the
project's modules with their rendered heads, every installed
package's modules with the `.gan` sources they ship (GEP-0006, the
standard library included), and the documentation files. No
tree-sitter, no index: the compiler is the parser, and the sources
of every dependency are already on disk.
"""

import gandora_core as core
import tomllib
import gandora_std.enum
import gandora_std.file
import gandora_std.map
import gandora_std.path
import gandora_std.string
from gandora_tool.safe import *


def _gan_truthy(value):
    return value is not None and value is not False

class GanMatchError(Exception):
    pass


def build(root: str, needle: str) -> dict:
    """The atlas for `root`: project modules (path + public heads, each
carrying its `@doc` first sentence), installed packages' modules
(shipped source + the same signed heads), and the documentation
files. `needle` narrows module lists to names containing it; `""`
keeps everything.

## Parameters

  - root: The project root.
  - needle: A module-name fragment; "" for the full atlas.
"""
    low = gandora_std.string.downcase(needle)
    return {"project": _project(root, low), "packages": packages(root, low), "docs": docs(root)}


def _project(root, low):
    try:
        _gan_tmp0 = core.wsymbols("", root)
    except Exception as _e__gan1:
        _gan_tmp0 = []
    syms = _gan_tmp0
    def _gan_fn0(s, acc, *, root=root):
        mod = gandora_std.map.get(s, "module", "?")
        entry = gandora_std.map.get(acc, mod, {"path": relative(gandora_std.map.get(s, "path", ""), root), "heads": []})
        if _gan_truthy(_private_p(s)):
            _gan_tmp1 = entry
        else:
            _gan_tmp1 = gandora_std.map.put(entry, "heads", gandora_std.map.get(entry, "heads") + [_signed(s)])
        entry = _gan_tmp1
        return gandora_std.map.put(acc, mod, entry)
    grouped = gandora_std.enum.reduce(syms, {}, _gan_fn0)
    sorted = gandora_std.enum.sort(gandora_std.map.to_list(grouped))
    return [gandora_std.map.put(e, "module", mod) for _gan_for2 in sorted if isinstance(_gan_for2, tuple) and len(_gan_for2) == 2 for (mod, e,) in [(_gan_for2[0], _gan_for2[1],)] if _gan_truthy(gandora_std.string.contains_p(gandora_std.string.downcase(mod), low))]


def _signed(s):
    head = gandora_std.map.get(s, "head", gandora_std.map.get(s, "name", ""))
    doc = _doc_sentence(s)
    if doc == "":
        return head
    else:
        return head + (" — " + doc)


def _doc_sentence(s):
    doc = gandora_std.map.get(s, "doc")
    if (doc is None):
        return ""
    else:
        return gandora_std.enum.join(gandora_std.string.split(str(doc)), " ")


def _private_p(s):
    return gandora_std.string.contains_p(gandora_std.map.get(s, "kind", ""), "defp")


def packages(root: str, low: str) -> dict:
    """The installed packages and their modules, from the `gandora.toml`
markers under the project venv (GEP-0006): package name, then per
module its shipped `.gan` source (absolute) and its signed heads.

## Parameters

  - root: The project root.
  - low: The lowercased module-name filter; "" keeps all.
"""
    def _gan_fn1(*_gan_args):
        match _gan_args:
            case ((_pkg, mods) as _gan_t3,) if isinstance(_gan_t3, tuple):
                return not (_gan_truthy(gandora_std.enum.empty_p(mods)))
        raise GanMatchError("no clause of _gan_fn1/1 matched " + repr(_gan_args))
    return gandora_std.map.new(gandora_std.enum.filter(gandora_std.enum.map(markers(root), lambda marker, *, low=low, root=root: (gandora_std.path.basename(gandora_std.path.dirname(marker)), _marker_modules(marker, root, low))), _gan_fn1))


def markers(root: str) -> list[str]:
    """Every `gandora.toml` marker under the project's venv, sorted.

## Parameters

  - root: The project root.
"""
    return gandora_std.path.wildcard(gandora_std.path.join(root, ".venv/lib/*/site-packages/*/gandora.toml"))


def _marker_modules(marker, root, low):
    site = gandora_std.path.dirname(gandora_std.path.dirname(marker))
    try:
        _gan_tmp4 = tomllib.loads(gandora_std.file.read_bang(marker))
    except Exception as _e__gan2:
        _gan_tmp4 = {}
    data = _gan_tmp4
    return [_dep_entry(m, site, root) for m in gandora_std.map.get(data, "modules", []) if _gan_truthy(_named_p(m, low))]


def _named_p(m, low):
    return gandora_std.string.contains_p(gandora_std.string.downcase(gandora_std.map.get(m, "name", "")), low)


def _dep_entry(m, site, root):
    name = gandora_std.map.get(m, "name", "")
    return {"module": name, "source": gandora_std.path.join(site, gandora_std.map.get(m, "source", "")), "heads": _public_heads(name, root)}


def _public_heads(mod, root):
    try:
        _gan_tmp5 = core.symbols(mod, root)
    except Exception as _e__gan3:
        _gan_tmp5 = []
    return [_signed(s) for s in _gan_tmp5 if not (_gan_truthy(_private_p(s)))]


def docs(root: str) -> list[str]:
    """The documentation files of the project: `README*` and root-level
`*.md`, plus everything under `docs/` — documentation is a
first-class member of the corpus (GEP-0031-R002).

## Parameters

  - root: The project root.
"""
    return gandora_std.enum.map(gandora_std.enum.sort(gandora_std.enum.uniq(gandora_std.path.wildcard(gandora_std.path.join(root, "README*")) + (gandora_std.path.wildcard(gandora_std.path.join(root, "*.md")) + gandora_std.path.wildcard(gandora_std.path.join(gandora_std.path.join(gandora_std.path.join(root, "docs"), "**"), "*.md"))))), lambda p, *, root=root: relative(p, root))


def corpus(root: str, deps: bool) -> list[str]:
    """The searchable corpus (GEP-0031-R002): the project's `.gan` sources
and documentation, plus — when `deps` — the `.gan` sources every
installed package ships. Absolute paths, sorted, project first.

## Parameters

  - root: The project root.
  - deps: Whether dependency sources join the corpus.
"""
    project_files = _sources(root) + gandora_std.enum.map(docs(root), lambda d, *, root=root: gandora_std.path.join(root, d))
    if _gan_truthy(deps):
        def _gan_fn2(marker):
            site = gandora_std.path.dirname(gandora_std.path.dirname(marker))
            try:
                _gan_tmp7 = tomllib.loads(gandora_std.file.read_bang(marker))
            except Exception as _e__gan4:
                _gan_tmp7 = {}
            data = _gan_tmp7
            return gandora_std.enum.map(gandora_std.map.get(data, "modules", []), lambda m, *, site=site: gandora_std.path.join(site, gandora_std.map.get(m, "source", "")))
        _gan_tmp6 = gandora_std.enum.sort(gandora_std.enum.flat_map(markers(root), _gan_fn2))
    else:
        _gan_tmp6 = []
    dep_files = _gan_tmp6
    return gandora_std.enum.uniq(project_files + dep_files)


def _sources(root):
    try:
        _gan_tmp8 = gandora_std.enum.sort(gandora_std.enum.uniq(gandora_std.enum.map(core.wsymbols("", root), lambda s: gandora_std.map.get(s, "path"))))
    except Exception as _e__gan5:
        _gan_tmp8 = []
    walked = _gan_tmp8
    if _gan_truthy(gandora_std.enum.empty_p(walked)):
        _gan_tmp9 = gandora_std.path.wildcard(gandora_std.path.join(root, "src/**/*.gan")) + gandora_std.path.wildcard(gandora_std.path.join(root, "tests/*.gan"))
    else:
        _gan_tmp9 = walked + gandora_std.path.wildcard(gandora_std.path.join(root, "tests/*.gan"))
    files = _gan_tmp9
    return gandora_std.enum.uniq(files)


def symbols(root: str) -> list[dict]:
    """Every public definition the atlas knows — project and installed
alike — as one flat stream: `target`, `head`, `doc`, `path`. These
are the documents the `--query` search ranks (GEP-0031-R003A).

## Parameters

  - root: The project root.
"""
    return _project_symbols(root) + _dep_symbols(root)


def _project_symbols(root):
    try:
        _gan_tmp10 = core.wsymbols("", root)
    except Exception as _e__gan6:
        _gan_tmp10 = []
    return [_symbol_entry(s, gandora_std.map.get(s, "module", "?"), relative(gandora_std.map.get(s, "path", ""), root)) for s in _gan_tmp10 if not (_gan_truthy(_private_p(s)))]


def _dep_symbols(root):
    def _gan_fn3(marker, *, root=root):
        site = gandora_std.path.dirname(gandora_std.path.dirname(marker))
        try:
            _gan_tmp11 = tomllib.loads(gandora_std.file.read_bang(marker))
        except Exception as _e__gan7:
            _gan_tmp11 = {}
        data = _gan_tmp11
        def _gan_fn4(m, *, root=root, site=site):
            mod = gandora_std.map.get(m, "name", "")
            src = gandora_std.path.join(site, gandora_std.map.get(m, "source", ""))
            try:
                _gan_tmp12 = core.symbols(mod, root)
            except Exception as _e__gan8:
                _gan_tmp12 = []
            return [_symbol_entry(s, mod, src) for s in _gan_tmp12 if not (_gan_truthy(_private_p(s)))]
        return gandora_std.enum.flat_map(gandora_std.map.get(data, "modules", []), _gan_fn4)
    return gandora_std.enum.flat_map(markers(root), _gan_fn3)


def _symbol_entry(s, mod, path):
    return {"target": mod + ("." + gandora_std.map.get(s, "name", "")), "head": gandora_std.map.get(s, "head", gandora_std.map.get(s, "name", "")), "doc": _doc_sentence(s), "path": path}


def relative(path: str, root: str) -> str:
    """A path made project-relative when it lives under `root`.

## Parameters

  - path: The path.
  - root: The project root.

    >>> relative("/a/b/src/m.gan", "/a/b")
    'src/m.gan'
"""
    return gandora_std.string.replace(str(path), root + "/", "")
