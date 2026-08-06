"""The one-call agent context (GEP-0026): everything a model needs to
start writing in a project — std function lists, project signatures,
the construct index, the spec cheat sheet, and a verdict summary —
sized for a prompt instead of a query loop.
"""

import collections.abc
import gandora_core as core
import gandora_lsp.construct_docs
import gandora_std.enum
import gandora_std.map


class GanMatchError(Exception):
    pass

std_modules = ["Enum", "List", "Map", "Keyword", "String", "Test"]


def known_std() -> list[str]:
    """The standard-library modules the pack enumerates.

    >>> known_std()
    ['Enum', 'List', 'Map', 'Keyword', 'String', 'Test']
"""
    return std_modules


def build(*_gan_args) -> dict:
    """The overview pack for `root`; `deep` names modules whose full docs
ride along.

## Parameters

  - root: The project root.
  - deep: Module names to include with full member docs.
"""
    while True:
        match _gan_args:
            case (root, deep,):
                return {"language": _language(), "std": _std_lists(root), "project": _project(root), "verdict": _verdict(root), "deep": _deep_docs(root, deep), "next": "details: `gan lsc pack <Mod>` for full module docs, `gan lsc doc <Mod.fun|construct>` for one item; then write -> `gan build` -> fix every finding -> `gan test`."}
            case (root,):
                _gan_args = (root, [])
                continue
        raise GanMatchError("no clause of build/1,2 matched " + repr(_gan_args))


def _language():
    return {"constructs": gandora_lsp.construct_docs.names(), "spec": gandora_lsp.construct_docs.card("spec"), "notes": ["No return/while; last expression is the value; only false and nil are falsy.", "Interop: $math.sqrt(x) one-off; pyimport json for repeated use; $python(expr) for a raw Python expression.", "Prompts: ~p(raw text, no escaping); data is maps %{\"k\" => v}; runtime JSON via $json.loads(s).", "Docs: @doc is prose only; examples go in the separate @example attribute (gan> call / Python-repr output) — never an Examples: section inside @doc."]}


def _std_lists(root):
    _gan_tmp0 = [(mod, _member_names(mod, root)) for mod in std_modules]
    pairs = _gan_tmp0
    return gandora_std.map.new(pairs)


def _member_names(mod, root):
    try:
        _gan_tmp1 = core.symbols(mod, root)
    except Exception as _e:
        _gan_tmp1 = []
    syms = _gan_tmp1
    return [gandora_std.map.get(s, "name") for s in syms if gandora_std.map.get(s, "kind") == "def"]


def _project(root):
    try:
        _gan_tmp2 = core.wsymbols("", root)
    except Exception as _e:
        _gan_tmp2 = []
    syms = _gan_tmp2
    def _gan_fn0(s, acc):
        if gandora_std.map.get(s, "kind") == "def":
            mod = gandora_std.map.get(s, "module", "?")
            heads = gandora_std.map.get(acc, mod, [])
            return gandora_std.map.put(acc, mod, heads + [gandora_std.map.get(s, "head", gandora_std.map.get(s, "name"))])
        else:
            return acc
    return gandora_std.enum.reduce(syms, {}, _gan_fn0)


def _verdict(root):
    try:
        _gan_tmp4 = core.check(root)
    except Exception as _e:
        _gan_tmp4 = []
    diags = _gan_tmp4
    errors = gandora_std.enum.count(gandora_std.enum.filter(diags, lambda d: gandora_std.map.get(d, "severity") == "error"))
    return {"ok": errors == 0, "errors": errors, "warnings": gandora_std.enum.count(diags) - errors}


def _deep_docs(root, deep):
    _gan_tmp5 = [(mod, _module_docs(mod, root)) for mod in deep]
    pairs = _gan_tmp5
    return gandora_std.map.new(pairs)


def _module_docs(mod, root):
    return [_doc_or_stub(f"{mod}.{name}", root) for name in _member_names(mod, root)]


def _doc_or_stub(target, root):
    try:
        _gan_tmp6 = core.doc(target, root)
    except Exception as _e:
        _gan_tmp6 = None
    info = _gan_tmp6
    if (info is None):
        return {"label": target}
    else:
        return info
