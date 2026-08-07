# Concurrency

Gandora's coroutine world is Python's, spoken natively: `async def`
and `await` compile one-to-one to Python's own syntax, and the `Task`
module is a thin library over `asyncio` — deadlines that return
verdicts, ordered joins, bounded fan-out, and one bridge for blocking
code (GEP-0029, GEP-0030).

```elixir
defmodule Fetcher do
  async def fetch_both(a, b) do          # → async def fetch_both(a, b):
    ta = Task.async(fetch(a))            # spawn: asyncio.ensure_future
    tb = Task.async(fetch(b))
    await Task.all([ta, tb])             # join in input order
  end

  async defp fetch(url) do
    resp = await http.get(url)           # → await http.get(url) — bare
    resp.body
  end

  def main() do
    IO.puts(Task.run(fetch_both(a, b)))  # the rim: asyncio.run
  end
end
```

`main` stays synchronous; a program's entire async portion hangs from
one `Task.run`. Calling an async function returns a coroutine — inert
until awaited or spawned.

## await

`await expr` is a prefix expression compiling to Python's bare
`await`: **no deadline, no wrapper**. It binds tighter than every
binary operator — `await fetch(u) |> parse()` pipes the awaited value
— and is legal only inside an async body: outside one, or inside a
`fn` closure (Python lambdas cannot await), it is a compile error.
Comprehension bodies are fine: `for t <- ts, do: await t` emits the
native `[await t for t in ts]`.

## The Task library

| Call | Meaning |
| --- | --- |
| `Task.run(coro)` | run a coroutine from sync code — the rim |
| `Task.async(coro)` | spawn; returns the task handle |
| `await t` | the join — language syntax, not a library call |
| `Task.try_await(t, ms)` | join with a deadline: `{:ok, v}` / `{:error, :timeout}` / `{:error, e}` |
| `Task.all(ts)` | values in input order (awaited) |
| `Task.try_all(ts, ms)` | one verdict per task under one group deadline |
| `Task.async_stream(xs, fun, max)` | bounded fan-out, results in input order |
| `Task.blocking(fun)` | run a blocking function on a worker thread |
| `Task.map / tap / recover` | lazy combinators over an awaitable |
| `Task.race(ts)` | first to settle, value or crash |
| `Task.shutdown(t)` | cancel — a running coroutine task really stops |
| `Task.sleep(ms)` / `Task.completed(v)` | pause; an already-held value |

Timeouts are **milliseconds**, as in Elixir. A deadline is a verdict,
not an exception — and on timeout the task is cancelled, `asyncio`'s
own `wait_for` semantics:

```elixir
case await Task.try_await(task, 5000) do
  {:ok, value} -> value
  {:error, :timeout} -> fallback        # the task was cancelled
  {:error, e} -> handle(e)              # the crash, as a value
end
```

## Blocking code crosses on the bridge

Threads are not a second world, just an edge: `Task.blocking(fn ->
requests.get(url) end)` returns an awaitable running the closure on a
worker thread (`asyncio.to_thread`). Spawn it to overlap blocking
work with coroutines:

```elixir
async def mixed() do
  a = Task.async(Task.blocking(fn -> slow_io() end))
  b = Task.async(compute())
  await Task.all([a, b])
end
```

## Doctests go through the rim

An `@example` on an async function is an ordinary doctest — write it
through `Task.run`, and it executes under `gan test` like every
other:

```elixir
@example """
    gan> Task.run(Fetcher.fetch_both("a", "b"))
    ['A', 'B']
"""
```

## What the build guarantees

`await` outside an async body, `await` inside `fn`, mixed
`def`/`async def` clauses, and `async def main` are compile errors.
The artifact is the Python a reviewer would write by hand — an
`async def` with bare `await`s — and its doctests really ran.
