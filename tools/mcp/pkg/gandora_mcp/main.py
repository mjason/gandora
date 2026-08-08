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

instructions = "Gandora is an Elixir-flavored language compiling to readable Python.\nCall `gan_briefing` once at the start of a session. Ask `gan_example`\nwhen you need to know how a feature or a piece of syntax is really\nwritten — it answers with prose plus a module that compiled and whose\ndoctests ran. Call `gan_verify` on every snippet before you hand it to\nthe user, including snippets you wrote yourself: a verdict of\n`clean: true` with `doctests.passed: true` is the only evidence that\nthe code works. `gan_doc` and `gan_pack` answer what a name means, and\nare the same answers as `gan lsc doc` / `gan lsc pack` if you have a\nshell.\n"

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


def tools() -> list[tuple]:
    """The tool table the hook accumulated from the `@tool` annotations —
the single source the server registers from: each entry is the
function's own name, a capture of it, and the blurb that sits on it.

    >>> gandora_std.enum.count(tools())
    6
"""
    return [("gan_verify", gan_verify, "Compile a Gandora module and run its @example doctests; returns the verdict."), ("gan_example", gan_example, "Explain a Gandora feature or syntax and demonstrate it with a module that compiled and whose doctests ran."), ("gan_doc", gan_doc, "Look up documentation for Mod.fun, a bare name, or a language construct."), ("gan_pack", gan_pack, "The one-call context pack for the project."), ("gan_check", gan_check, "The build verdict for the whole project."), ("gan_briefing", gan_briefing, "The session briefing for working in Gandora.")]


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
            case ((name, f, _blurb) as _gan_t0,) if isinstance(_gan_t0, tuple):
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
                case ((pname, text) as _gan_t1,) if isinstance(_gan_t1, tuple):
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
        _gan_tmp2 = [doc]
    else:
        _gan_tmp2 = gandora_std.string.split_on(doc, "## Parameters")
    parts = _gan_tmp2
    if gandora_std.enum.count(parts) < 2:
        return {}
    else:
        matches = gandora_std.enum.map(gandora_std.enum.take_while(gandora_std.string.split_on(gandora_std.enum.at(parts, 1), "\n"), lambda l: _gan_or(gandora_std.string.trim(l) == "", lambda: gandora_std.string.match_p(l, param_line))), lambda l: param_line.match(l))
        _gan_tmp3 = [(m.group(1), m.group(2)) for m in matches if not ((m is None))]
        pairs = _gan_tmp3
        return gandora_std.map.new(pairs)


def main() -> None:
    """Serves the tool table over stdio until the client disconnects."""
    return server().run("stdio")


if __name__ == "__main__":
    main()
