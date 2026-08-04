# Macros

Compile-time, hygienic, Elixir-shaped. Macros run in a deterministic
sandbox inside the compiler and leave **no runtime trace** — the
generated Python contains only their expansion.

```elixir
defmacro unless_nil(value, fallback) do
  quote do
    case unquote(value) do
      nil -> unquote(fallback)
      found -> found
    end
  end
end
```

## The toolkit

- `quote do ... end` returns the AST of its block; `unquote(x)`
  splices a value back in; `unquote_splicing(list)` splices sequences.
- Template variables are renamed per expansion (**hygiene**);
  `var!(name)` deliberately reaches the caller's scope.
- `def unquote(head)(...)` builds definitions — the pattern behind
  `use`-style code injection:

```elixir
defmacro __using__(_opts) do
  quote do
    def unquote(:injected)(), do: :from_using
  end
end
```

- Quoted code destructures as plain data — `{name, meta, args}`
  triples and do-block pairs — so macros can rewrite what they
  receive. Builders like `slug/1`, `downcase/1`, `replace/3`, and
  `to_atom/1` are available at expansion time.
- `compile_warn/1` lets a library macro emit a **spanned compiler
  warning** at its call site — extensions speak with the same voice as
  the kernel.

## Bringing macros in

`require Mod` makes `Mod`'s macros available; `import Mod` also brings
names in bare; `use Mod` invokes `Mod.__using__/1` to inject code.
The standard library's ExUnit-style test surface
([Testing](testing.md)) is built entirely from these pieces.

## Inspecting expansions

```console
gan expand src/x.gan        # the module after macro expansion
gan lsc expand src/x.gan    # the same as quoted AST (JSON)
```

or the editor's *Expand Macros* command. Macros shipped in a published
wheel keep working for consumers — packages carry their `.gan` sources
precisely so downstream expansion can run.
