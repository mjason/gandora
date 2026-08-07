"""The atom corpus (GEP-0028-R009): one verified example per language
capability, read from `@example` blocks the test suite already runs.
An atom is evidence, not illustration — it earned its place by
passing `gan test` before anyone asked for it.
"""

import collections.abc
import gandora_core as core
import gandora_std.enum
import gandora_std.list
import gandora_std.map
import gandora_std.string


def _gan_truthy(value):
    return value is not None and value is not False

def _gan_and(value, then):
    return then() if _gan_truthy(value) else value

class GanMatchError(Exception):
    pass

staples = ["Enum.map", "Enum.reduce", "Enum.filter", "String.split", "Map.get"]

stopwords = ["the", "and", "for", "with", "that", "this", "from", "into", "list", "value", "values", "function", "please", "write", "module", "example", "gandora", "returns", "return", "given", "each", "all", "use", "using", "one", "two", "its", "them", "then"]


def modules() -> list[str]:
    """The standard-library modules the corpus draws from.

    >>> modules()
    ['Enum', 'List', 'Map', 'Keyword', 'String', 'Task', 'Test']
"""
    return ["Enum", "List", "Map", "Keyword", "String", "Task", "Test"]


def catalog(root: str) -> list[dict]:
    """Every atom visible from `root`: the target, its head, its spec, its
first doc sentence, and the example that proves it.

## Parameters

  - root: A project root whose config resolves the standard library.
"""
    return gandora_std.enum.flat_map(modules(), lambda m, *, root=root: _module_atoms(m, root))


def search(root: str, query: str, k: int) -> list[dict]:
    """The `k` atoms whose text best matches `query` — plain word overlap,
deterministic, and free: choosing what to show a model is not a
question that needs a model.

## Parameters

  - root: A project root whose config resolves the standard library.
  - query: The requirement text.
  - k: How many atoms to return.
"""
    _gan_tmp0 = [_stem(w) for w in keywords(query)]
    words = _gan_tmp0
    all = catalog(root)
    _gan_tmp1 = [(score(a, words), a) for a in all]
    scored = _gan_tmp1
    def _gan_fn0(*_gan_args):
        match _gan_args:
            case ((s, _a) as _gan_t2,) if isinstance(_gan_t2, tuple):
                return s > 0
        raise GanMatchError("no clause of _gan_fn0/1 matched " + repr(_gan_args))
    def _gan_fn1(*_gan_args):
        match _gan_args:
            case ((s, _a) as _gan_t3,) if isinstance(_gan_t3, tuple):
                return s
        raise GanMatchError("no clause of _gan_fn1/1 matched " + repr(_gan_args))
    top = gandora_std.enum.take(gandora_std.enum.reverse(gandora_std.enum.sort_by(gandora_std.enum.filter(scored, _gan_fn0), _gan_fn1)), k)
    _gan_tmp4 = [a for _gan_for5 in top if isinstance(_gan_for5, tuple) and len(_gan_for5) == 2 for (a,) in [(_gan_for5[1],)]]
    hits = _gan_tmp4
    return _pad(hits, all, k)


def _pad(hits, catalog, k):
    have = gandora_std.enum.map(hits, lambda a: gandora_std.map.get(a, "target"))
    extra = gandora_std.enum.filter(catalog, lambda a, *, have=have: _gan_and(gandora_std.enum.member_p(staples, gandora_std.map.get(a, "target")), lambda: not (_gan_truthy(gandora_std.enum.member_p(have, gandora_std.map.get(a, "target"))))))
    return gandora_std.enum.take(hits + extra, k)


def keywords(text: str) -> list[str]:
    """The lowercased words of `text` worth matching on.

## Parameters

  - text: Any prose.

    >>> keywords("Sum the Numbers, please")
    ['sum', 'numbers']
"""
    return gandora_std.enum.filter(gandora_std.string.split(gandora_std.string.replace(gandora_std.string.replace(gandora_std.string.downcase(text), ",", " "), ".", " ")), lambda w: (gandora_std.string.length(w) > 2) and not (_gan_truthy(gandora_std.enum.member_p(stopwords, w))))


def score(atom: collections.abc.Mapping[str, object], words: collections.abc.Sequence[str]) -> int:
    """How strongly the atom answers `words`: a hit in the name is worth
three in the prose, because a name is what the caller will type.

## Parameters

  - atom: One catalog entry.
  - words: The query keywords.

    >>> score({"target": "Enum.sum", "doc": "Adds the numbers."}, ["sum", "numbers"])
    4
"""
    _gan_tmp6 = [_stem(w) for w in _name_words(gandora_std.map.get(atom, "target", ""))]
    name = _gan_tmp6
    _gan_tmp7 = [_stem(w) for w in keywords(gandora_std.map.get(atom, "doc", ""))]
    prose = _gan_tmp7
    _gan_tmp8 = [_hit(name, prose, _stem(w)) for w in words]
    return gandora_std.enum.sum(_gan_tmp8)


def _hit(name, prose, stem):
    if _gan_truthy(gandora_std.enum.member_p(name, stem)):
        _gan_tmp9 = 3
    else:
        _gan_tmp9 = 0
    name_hit = _gan_tmp9
    if _gan_truthy(gandora_std.enum.member_p(prose, stem)):
        _gan_tmp10 = 1
    else:
        _gan_tmp10 = 0
    prose_hit = _gan_tmp10
    return name_hit + prose_hit


def _name_words(target):
    return gandora_std.string.split_on(gandora_std.string.replace(gandora_std.string.downcase(target), ".", "_"), "_")


def _stem(word):
    if gandora_std.string.length(word) > 4:
        return gandora_std.string.slice(word, 0, 4)
    else:
        return word


def _module_atoms(module, root):
    try:
        _gan_tmp11 = core.symbols(module, root)
    except Exception as _e:
        _gan_tmp11 = []
    syms = _gan_tmp11
    return gandora_std.enum.flat_map(gandora_std.enum.uniq(gandora_std.enum.map(syms, lambda s: gandora_std.map.get(s, "name"))), lambda n, *, module=module, root=root: gandora_std.list.wrap(_atom(module, n, root)))


def _atom(module, name, root):
    target = module + ("." + name)
    try:
        _gan_tmp12 = core.doc(target, root)
    except Exception as _e:
        _gan_tmp12 = None
    info = _gan_tmp12
    if (info is None):
        return None
    else:
        examples = gandora_std.map.get(info, "examples", [])
        if _gan_truthy(gandora_std.enum.empty_p(examples)):
            return None
        else:
            return {"target": target, "head": gandora_std.enum.join(gandora_std.map.get(info, "signatures", []), " | "), "spec": gandora_std.enum.join(gandora_std.map.get(info, "specs", []), " | "), "doc": gandora_std.map.get(gandora_std.map.get(info, "entries", {}), "default", ""), "example": gandora_std.string.trim(gandora_std.enum.join(examples, "\n"))}
