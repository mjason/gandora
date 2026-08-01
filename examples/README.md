# Gandora examples

## tour

A small multi-module project exercising the v0 surface: multi-clause
functions with guards, `cond`, pipelines, private functions, a hygienic
macro imported with `require`, destructuring, and `:module` Python interop.

```console
cd examples/tour
gan run src/main.gan
```

Module-to-path mapping (GEP-0001-R013):

| File | Module |
| --- | --- |
| `src/main.gan` | `Main` |
| `src/app/cli.gan` | `App.Cli` |
| `src/app/mathy.gan` | `App.Mathy` |
| `src/app/macros.gan` | `App.Macros` |
| `src/app/shop.gan` | `App.Shop` — structs, module attributes, and a decorator registry (GEP-0004) |
| `src/sigils.gan` | `Sigils` — `~w`/`~s`/`~r` and embedded-Python `~python` (GEP-0005) |

`gan build` writes the generated Python into `dist/`; `gan expand
src/app/cli.gan` shows the module after macro expansion.
