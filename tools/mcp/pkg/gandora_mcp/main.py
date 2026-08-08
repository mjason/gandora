"""`gan-mcp` — the Gandora MCP server, written in Gandora (GEP-0028).

The tool schemas are not declared twice: `@spec` becomes the Python
signature, the signature becomes the MCP `inputSchema`, and `@doc`
becomes the tool description. What the language already demands of
every public function is exactly what the protocol needs.
"""

import collections.abc
import gandora_core as core
import mcp.server.mcpserver as ms
import re
import gandora_mcp.composer
import gandora_mcp.intel
import gandora_mcp.sandbox
import gandora_std.enum
import gandora_std.file
import gandora_std.map
import gandora_std.path
import gandora_std.string
import gandora_std.system


def _gan_truthy(value):
    return value is not None and value is not False

def _gan_or(value, then):
    return value if _gan_truthy(value) else then()

class GanMatchError(Exception):
    pass

instructions = "Gandora is an Elixir-flavored language compiling to readable Python.\nCall `gan_briefing` once at the start of a session. Ask `gan_example`\nwhen you need to know how a feature or a piece of syntax is really\nwritten — it answers with prose plus a module that compiled and whose\ndoctests ran. Call `gan_verify` on every snippet before you hand it to\nthe user, including snippets you wrote yourself: a verdict of\n`clean: true` with `doctests.passed: true` is the only evidence that\nthe code works. `gan_doc` and `gan_pack` answer what a name means, and\nare the same answers as `gan lsc doc` / `gan lsc pack` if you have a\nshell. To find code before touching it: `gan_map` orients (project,\ninstalled packages, docs), `gan_search_files`/`gan_search_content`\nlocate (the corpus includes documentation and every installed\npackage's shipped .gan sources), and `gan_read` fetches exactly one\nblock — a line range, a module, or `Mod.fun` with its annotations —\ninstead of a whole file.\n"

param_line = re.compile("^\\s*- ([a-z_][A-Za-z0-9_]*): (.*)$")


def gan_verify(source: str) -> dict:
    """Compiles a Gandora module in a throwaway project and really runs its
`@example` doctests; returns errors, practice suggestions, and the
doctest output.

## Parameters

  - source: The Gandora source text of one module.
"""
    return gandora_mcp.sandbox.verdict(source, root())


async def gan_example(requirement: str) -> dict:
    """Explains one Gandora feature or piece of syntax and demonstrates it
with a complete module — returned only once that module has compiled,
passed the practice pass, and had its `@example` doctests really run
(GEP-0028-R006). The prose explains; every line of code in the answer
is code that ran.

## Parameters

  - requirement: The feature, syntax, or capability to demonstrate.
"""
    return (await gandora_mcp.composer.example(requirement, root()))


def gan_doc(target: str) -> object:
    """The documentation for one target — `Enum.map`, a bare function name,
or a language construct such as `defmacro`.

## Parameters

  - target: What to look up.
"""
    return gandora_mcp.intel.doc(target, root())


def gan_pack(modules: collections.abc.Sequence[str]) -> object:
    """The one-call context pack for the current project: std function lists,
project signatures, the construct index, and the verdict summary.

## Parameters

  - modules: Module names whose full docs ride along; `[]` for the overview.
"""
    return gandora_mcp.intel.pack(modules, root())


def gan_check() -> object:
    """The verdict for the current project: `ok`, `clean`, and why not."""
    return gandora_mcp.intel.check(root())


def gan_briefing() -> str:
    """The session briefing: how to work in Gandora, plus this project."""
    return gandora_mcp.intel.briefing(root())


def gan_map(query: str, ranked: bool) -> object:
    """The code atlas (GEP-0031): project modules with paths and doc-signed
public heads, every installed package's modules with the `.gan`
sources they ship (the standard library included), and the
documentation files. With `ranked`, the query searches the atlas
itself — every definition is a BM25 document of name, head, and
`@doc` sentence — answering "which function does X" without knowing
its name.

## Parameters

  - query: A module-name fragment ("" for the full atlas), or — with `ranked` — the words describing what the definition should do.
  - ranked: Rank the atlas's definitions by BM25 instead of listing the map.
"""
    if _gan_truthy(ranked):
        _gan_tmp0 = ["map", query, "--query"]
    else:
        _gan_tmp0 = ["map", query]
    return gandora_mcp.intel.lsc(_gan_tmp0, root())


def gan_search_files(pattern: str, deps: bool) -> object:
    """Corpus files whose name matches `pattern` — a glob when it carries
`*`/`?`, a substring otherwise. The corpus is project sources and
docs; `deps` adds the `.gan` sources of every installed package
(GEP-0031).

## Parameters

  - pattern: The file-name pattern.
  - deps: Whether installed packages' sources join the search.
"""
    return gandora_mcp.intel.lsc(["find", pattern] + _flag(deps, "--deps"), root())


def gan_search_content(pattern: str, ranked: bool, deps: bool) -> object:
    """Content search over the corpus (GEP-0031): a regular expression by
default, or BM25 ranking when `ranked` — use ranked mode for prose
questions, regex for exact shapes. `deps` adds the `.gan` sources of
every installed package, so library code and its `@doc` prose are
searchable too. Capped output says so via `truncated`.

## Parameters

  - pattern: The regular expression, or the words to rank by.
  - ranked: BM25 ranking instead of regex matching.
  - deps: Whether installed packages' sources join the search.
"""
    return gandora_mcp.intel.lsc(["grep", pattern] + (_flag(ranked, "--ranked") + _flag(deps, "--deps")), root())


def gan_read(target: str, from_line: int, to_line: int) -> object:
    """One precise read (GEP-0031): a corpus path with a 1-based inclusive
line range (0, 0 for the whole file), a module name (its whole
source, project or installed), or `Mod.fun` (that definition's
block, annotations included). Prefer this over paging whole files.

## Parameters

  - target: A corpus path, `Mod`, or `Mod.fun`.
  - from_line: First line, 1-based; 0 for a named or whole-file read.
  - to_line: Last line, inclusive; 0 for a named or whole-file read.
"""
    return gandora_mcp.intel.lsc(["read", target, str(from_line), str(to_line)], root())


def _flag(on, name):
    if _gan_truthy(on):
        return [name]
    else:
        return []


def tools() -> list[tuple]:
    """The tool table the hook accumulated from the `@tool` markers — the
single source the server registers from: each entry is the
function's own name and a capture of it.

    >>> gandora_std.enum.count(tools())
    6
"""
    return [("gan_verify", gan_verify), ("gan_example", gan_example), ("gan_doc", gan_doc), ("gan_pack", gan_pack), ("gan_check", gan_check), ("gan_briefing", gan_briefing), ("gan_map", gan_map), ("gan_search_files", gan_search_files), ("gan_search_content", gan_search_content), ("gan_read", gan_read)]


def root() -> str:
    """The project the queries run against: `$GAN_MCP_ROOT` when set,
otherwise the nearest ancestor of the working directory holding a
`gandora.jsonc`. A client chooses where it launches a server, and it
may well choose a subdirectory — so the project is discovered, not
configured.
"""
    env = gandora_std.system.get_env("GAN_MCP_ROOT")
    if (env is None):
        return _discover(gandora_std.file.cwd_bang())
    else:
        return env


def _discover(dir):
    while True:
        if _gan_truthy(gandora_std.file.exists_p(gandora_std.path.join(dir, "gandora.jsonc"))):
            return dir
        elif gandora_std.path.dirname(dir) == dir:
            return gandora_std.file.cwd_bang()
        else:
            dir = gandora_std.path.dirname(dir)
            continue


def server() -> object:
    """The configured server: every table entry registered with its whole
annotation surface wired into the protocol — the signature (from
`@spec`) is the inputSchema, the `@doc` prose is the description,
and each `@param` text lands on its schema property. Declared once
on the definition, visible to every client.
"""
    s = ms.MCPServer("gandora", version=core.version(), instructions=instructions)
    def _gan_fn0(*_gan_args, s=s):
        match _gan_args:
            case ((name, f) as _gan_t1,) if isinstance(_gan_t1, tuple):
                return _register(s, name, f)
        raise GanMatchError("no clause of _gan_fn0/1 matched " + repr(_gan_args))
    gandora_std.enum.each(tools(), _gan_fn0)
    return s


def _register(s, name, f):
    s.add_tool(f, name=name, description=doc_prose(f.__doc__))
    return _annotate(s._tool_manager.get_tool(name), param_docs(f.__doc__))


def _annotate(tool, docs):
    if not ((tool is None)):
        props = gandora_std.map.get(tool.parameters, "properties", {})
        def _gan_fn1(*_gan_args, props=props):
            match _gan_args:
                case ((pname, text) as _gan_t2,) if isinstance(_gan_t2, tuple):
                    if _gan_truthy(gandora_std.map.has_key_p(props, pname)):
                        _none = gandora_std.map.get(props, pname).update({"description": text})
                        return None
                    else:
                        return None
            raise GanMatchError("no clause of _gan_fn1/1 matched " + repr(_gan_args))
        return gandora_std.enum.each(gandora_std.map.to_list(docs), _gan_fn1)
    else:
        return None


def doc_prose(doc: str) -> str:
    """The tool description from a compiled docstring: the `@doc` prose,
with the `## Parameters` section (which rides the schema instead)
cut off.

## Parameters

  - doc: The function docstring; nil for an undocumented function.

    >>> doc_prose("Does things.\\n\\n## Parameters\\n\\n  - x: The input.\\n")
    'Does things.'
"""
    if (doc is None):
        return ""
    else:
        return gandora_std.string.trim(gandora_std.enum.at(gandora_std.string.split_on(doc, "## Parameters"), 0))


def param_docs(doc: str) -> dict:
    """The `@param` texts by parameter name, parsed from the compiled
`## Parameters` section — the exact shape our own codegen emits
(GEP-0018), so the parse is a contract, not a guess.

## Parameters

  - doc: The function docstring; nil for an undocumented function.

    >>> param_docs("D.\\n\\n## Parameters\\n\\n  - x: The input.\\n  - y: The other.\\n")
    {'x': 'The input.', 'y': 'The other.'}
"""
    if (doc is None):
        _gan_tmp3 = [doc]
    else:
        _gan_tmp3 = gandora_std.string.split_on(doc, "## Parameters")
    parts = _gan_tmp3
    if gandora_std.enum.count(parts) < 2:
        return {}
    else:
        matches = gandora_std.enum.map(gandora_std.enum.take_while(gandora_std.string.split_on(gandora_std.enum.at(parts, 1), "\n"), lambda l: _gan_or(gandora_std.string.trim(l) == "", lambda: gandora_std.string.match_p(l, param_line))), lambda l: param_line.match(l))
        _gan_tmp4 = [(m.group(1), m.group(2)) for m in matches if not ((m is None))]
        pairs = _gan_tmp4
        return gandora_std.map.new(pairs)


def main() -> None:
    """Serves the tool table over stdio until the client disconnects."""
    return server().run("stdio")


if __name__ == "__main__":
    main()
