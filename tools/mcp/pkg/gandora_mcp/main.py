"""`gan-mcp` — the Gandora MCP server, written in Gandora (GEP-0028).

The tool schemas are not declared twice: `@spec` becomes the Python
signature, the signature becomes the MCP `inputSchema`, and `@doc`
becomes the tool description. What the language already demands of
every public function is exactly what the protocol needs.
"""

import collections.abc
import gandora_core as core
import mcp.server.mcpserver as ms
import gandora_mcp.composer
import gandora_mcp.intel
import gandora_mcp.sandbox
import gandora_std.enum
import gandora_std.file
import gandora_std.path
import gandora_std.system


def _gan_truthy(value):
    return value is not None and value is not False

class GanMatchError(Exception):
    pass

instructions = "Gandora is an Elixir-flavored language compiling to readable Python.\nCall `gan_briefing` once at the start of a session. Ask `gan_example`\nwhen you need to know how a feature or a piece of syntax is really\nwritten — it answers with prose plus a module that compiled and whose\ndoctests ran. Call `gan_verify` on every snippet before you hand it to\nthe user, including snippets you wrote yourself: a verdict of\n`clean: true` with `doctests.passed: true` is the only evidence that\nthe code works. `gan_doc` and `gan_pack` answer what a name means, and\nare the same answers as `gan lsc doc` / `gan lsc pack` if you have a\nshell.\n"


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


def main() -> None:
    """Serves the tool table over stdio until the client disconnects."""
    server = ms.MCPServer("gandora", version=core.version(), instructions=instructions)
    def _gan_fn0(*_gan_args, server=server):
        match _gan_args:
            case ((name, f, desc) as _gan_t0,) if isinstance(_gan_t0, tuple):
                return server.add_tool(f, name=name, description=desc)
        raise GanMatchError("no clause of _gan_fn0/1 matched " + repr(_gan_args))
    gandora_std.enum.each(tools(), _gan_fn0)
    return server.run("stdio")


if __name__ == "__main__":
    main()
