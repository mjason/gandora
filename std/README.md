# gandora-std

The [Gandora](https://github.com/mjason/gandora) standard library:
`Enum`, `String`, `Map`, `List`, `Keyword` — written in Gandora,
data-first, bilingual (`gan doc Enum.map --locale zh`), fully doctested.

```console
uv add git+https://github.com/mjason/gandora#subdirectory=std
```

```elixir
[3, 1, 2]
|> Enum.sort()
|> Enum.map(fn x -> x * 10 end)
|> Enum.join("-")          # "10-20-30"
```

An ordinary package (GEP-0010): versioned independently of the
compiler, no compiler embedding, no runtime beyond this wheel. MIT.
