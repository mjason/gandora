"""语言构造卡片:hover 与 lsc doc 共用的一句话文档(GEP-0015)。"""

import gandora_std.map

cards = {"def": "Defines a public function. `def f(x), do: expr` or a `do ... end` body. Multi-clause heads dispatch by pattern, top to bottom (GEP-0001).", "defp": "Defines a private function — callable only inside its module; compiled with a leading underscore (GEP-0001).", "defmodule": "Declares the module for this file. One `defmodule` per file; the name maps to the generated Python module path (GEP-0001-R013).", "defmacro": "Defines a compile-time macro: it receives quoted arguments and returns quoted code (GEP-0002).", "defstruct": "Declares the module's struct with defaulted fields; literals `%Mod{...}`, updates `%Mod{s | ...}` and patterns work on it (GEP-0004).", "defattr": "Registers a custom annotation attribute handled by the module's `@on_definition` hook (GEP-0008).", "quote": "Returns the quoted AST of its block instead of evaluating it; `unquote` splices values back in (GEP-0002).", "unquote": "Inside `quote`, splices an evaluated value into the quoted code (GEP-0002).", "case": "Pattern-matches a value against clauses, top to bottom; the whole form is an expression (GEP-0001).", "cond": "Evaluates conditions top to bottom and takes the first truthy branch (GEP-0001).", "with": "Chains `pattern <- expr` matches; the first failure falls to `else` (GEP-0001).", "try": "Runs a body with `rescue` clauses matching Python exception types and an always-run `after` (GEP-0014).", "rescue": "Clauses of a `try`: `e in $mod.Type -> ...` matches by exception class; a bare variable catches every Exception (GEP-0014).", "after": "The cleanup section of `try`: always runs, contributes no value (GEP-0014).", "recur": "Restarts the enclosing function with new arguments — the explicit, compile-checked spelling of tail recursion: must be in tail position, arity must match a clause (GEP-0019-R005).", "for": "A comprehension: `for pat <- enum, filter, do: body` compiles to a native Python comprehension; non-matching patterns are skipped, `into: %{}` builds a map (GEP-0020).", "fn": "An anonymous function: `fn x -> x * 2 end`, called with `f.(x)`; supports multiple clauses and guards (GEP-0001).", "pyimport": "Declares a Python import at module top: `pyimport numpy, as: np` (GEP-0003).", "use": "Invokes the target module's `__using__` macro to inject code here (GEP-0008).", "require": "Makes the target module's macros available in this file (GEP-0002).", "unless": "`if` with the condition negated (GEP-0001).", "when": "A guard on a clause head or case pattern (GEP-0001).", "spec": "@spec name(arg_types) :: return — the whole type language: scalars integer() float() number() string() boolean() atom() nil term(); containers list(t) tuple(a, b) map(k, v); abstract sequence(t) iterable(t) mapping(k, v) keyword(); unions a | b; ok/error tuples tuple(atom(), term()); named args name :: type; type variables are 1-2 lowercase letters; Python types $mod.Type(); struct types Mod.t(); functions fun() (GEP-0017).", "example": "@example \"\"\" / four-space-indented gan> call / expected output (Python repr) / \"\"\" — placed above the def; gan test runs every one (GEP-0007).", "test": "Official tests: tests/*.gan modules whose test_* functions use Test.assert_eq / assert_true / assert_nil / assert_raises / assert_contains; `gan test` compiles them with the project and runs pytest (GEP-0024)."}


def card(token: str) -> str | None:
    """The one-line card for a language construct, or nil.

## Parameters

  - token: A keyword such as "for", "recur", "defmodule".
"""
    hit = gandora_std.map.get(cards, token)
    if (hit is None):
        return gandora_std.map.get(cards, token.lower())
    else:
        return hit
