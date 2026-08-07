"""The library over Python's coroutine world (GEP-0029): `run` enters
it from synchronous code, `async` spawns, and the language's own
`await` joins — bare, with no deadline. Everything above a bare join
is here as a thin `asyncio` wrapper: deadlines that return verdicts
(`try_await`), ordered joins (`all`), bounded fan-out
(`async_stream`), combinators after Gleam's promise module, and
`blocking` — the one remaining role of threads, a bridge at the
edge, not a world. Cancellation is real: a cancelled coroutine task
actually stops. Timeouts are milliseconds, as in Elixir.
"""

import asyncio
import builtins
import collections.abc
import time
import gandora_std.enum


class GanMatchError(Exception):
    pass


def run(coro: object) -> object:
    """Runs a coroutine to completion from synchronous code and returns its
value — `asyncio.run`, verbatim. The only Task function meant for
the synchronous rim: `main` stays synchronous, and a program's
entire async portion hangs from one `run`.

## Parameters

  - coro: The coroutine to run.

    >>> run(completed(42))
    42
"""
    return asyncio.run(coro)


def async__kw(coro: object) -> object:
    """Spawns a coroutine as a task running concurrently with its caller
and returns the handle — `asyncio.ensure_future`. Join it with the
language's `await`; a task nobody awaits fails silently. Only legal
where a loop is running, i.e. inside the coroutine world.

## Parameters

  - coro: The coroutine to spawn.

    >>> run(demo())
    ([1, 2], True)
"""
    return asyncio.ensure_future(coro)


def blocking(fun: collections.abc.Callable) -> object:
    """A coroutine that runs the zero-argument blocking function on a
worker thread — `asyncio.to_thread`, verbatim. Await it for the
value, or spawn it with `async` to overlap blocking work with
coroutines. This is the whole remaining role of threads.

## Parameters

  - fun: A zero-argument blocking function.

    >>> run(blocking(lambda : 40 + 2))
    42
"""
    return asyncio.to_thread(fun)


async def try_await(task: object, timeout: int) -> tuple:
    """The join with a deadline, returning a verdict instead of raising:
`{:ok, value}` when the task finished, `{:error, :timeout}` when
`timeout` milliseconds elapsed first — the task is then cancelled,
asyncio's own `wait_for` semantics — and `{:error, error}` when it
raised (the exception as a value). The timeout is required: the
no-deadline join is the bare `await`.

## Parameters

  - task: The task or awaitable to join.
  - timeout: Milliseconds before the join gives up and cancels.

    >>> run(try_await(completed(7), 1000))
    ('ok', 7)
    >>> run(try_await(sleep(60000), 10))
    ('error', 'timeout')
"""
    try:
        return ("ok", (await asyncio.wait_for(task, timeout / 1000)))
    except builtins.TimeoutError as _e:
        return ("error", "timeout")
    except Exception as e:
        return ("error", e)


async def all(tasks: collections.abc.Sequence[object]) -> list[object]:
    """The values of `tasks` in input order. The tasks already run
concurrently, so joining them in order costs nothing — the artifact
is the sequential join it reads as. A crash re-raises at this join.

## Parameters

  - tasks: The tasks or awaitables to join.

    >>> run(all([completed(1), completed(2)]))
    [1, 2]
"""
    return [(await t) for t in tasks]


async def try_all(tasks: collections.abc.Sequence[object], timeout: int) -> list[tuple]:
    """One `try_await` verdict per task, in order, under one group deadline
of `timeout` milliseconds for all of them together.

## Parameters

  - tasks: The tasks or awaitables to join.
  - timeout: Milliseconds for the whole group.

    >>> run(try_all([completed(1), completed(2)], 1000))
    [('ok', 1), ('ok', 2)]
"""
    deadline = time.monotonic() + (timeout / 1000)
    return [(await try_await(t, builtins.max(deadline - time.monotonic(), 0) * 1000)) for t in tasks]


async def async_stream(xs: collections.abc.Sequence[object], fun: collections.abc.Callable, max_concurrency: int = 8) -> list[object]:
    """Applies `fun` to each element with at most `max_concurrency` running
at once, and returns the results in input order. `fun` must return
an awaitable — call an async function, or `blocking` for a blocking
one; the bound is a semaphore, visible in the artifact.

## Parameters

  - xs: The inputs.
  - fun: Applied to each input; must return an awaitable.
  - max_concurrency: How many may run at once (default 8).

    >>> run(async_stream([1, 2, 3], lambda x: completed(x * 2), 2))
    [2, 4, 6]
"""
    while True:
        sem = asyncio.Semaphore(max_concurrency)
        ts = gandora_std.enum.map(xs, lambda x, *, fun=fun, sem=sem: asyncio.ensure_future(_throttle(sem, fun, x)))
        return [(await t) for t in ts]


async def _throttle(sem, fun, x):
    _gan_tmp0 = (await sem.acquire())
    try:
        return (await fun(x))
    finally:
        sem.release()


async def map(task: object, fun: collections.abc.Callable) -> object:
    """The awaitable of `fun(value)`, without joining — a lazy coroutine,
runnable anywhere an awaitable goes: await it, spawn it with
`async`, hand it to `run`. A crash propagates untransformed, as a
rejected promise passes a `map` (GEP-0029-R007).

## Parameters

  - task: The task or awaitable whose value to transform.
  - fun: Applied to the value when it arrives.

    >>> run(map(completed(20), lambda v: v + 1))
    21
"""
    return fun((await task))


async def tap(task: object, fun: collections.abc.Callable) -> object:
    """Runs `fun(value)` for its effect and passes the original value
through — a probe on the wire, after Gleam's `tap`.

## Parameters

  - task: The task or awaitable to observe.
  - fun: Called with the value; its result is discarded.

    >>> run(tap(completed(5), lambda _v: None))
    5
"""
    v = (await task)
    _gan_tmp1 = fun(v)
    return v


async def recover(task: object, fun: collections.abc.Callable) -> object:
    """The awaitable of `fun(error)` when `task` crashed, of the value when
it did not. Gleam spells this `rescue`; `rescue` is a Gandora
keyword.

## Parameters

  - task: The task or awaitable whose crash to absorb.
  - fun: Applied to the exception; its result becomes the value.

    >>> run(recover(map(completed(0), lambda v: 1 / v), lambda _e: "fallback"))
    'fallback'
"""
    try:
        return (await task)
    except Exception as e:
        return fun(e)


async def race(tasks: collections.abc.Sequence[object]) -> object:
    """Settles with the first of `tasks` to settle — value or crash —
after Gleam's `race_list`. The losers keep running until the loop
closes.

## Parameters

  - tasks: The competing tasks or awaitables.

    >>> run(race([completed("first"), sleep(60000)]))
    'first'
"""
    _gan_tmp2 = [asyncio.ensure_future(t) for t in tasks]
    ts = _gan_tmp2
    _gan_val3 = (await asyncio.wait(ts, return_when=asyncio.FIRST_COMPLETED))
    match _gan_val3:
        case (done, _pending) as _gan_t4 if isinstance(_gan_t4, tuple):
            pass
        case _:
            raise GanMatchError("no match of right-hand side value: " + repr(_gan_val3))
    return builtins.next(builtins.iter(done)).result()


def shutdown(task: object) -> bool:
    """Requests cancellation and returns whether the request took. Unlike a
thread, a running coroutine task really is interrupted.

## Parameters

  - task: The task to cancel.

    >>> run(demo())
    ([1, 2], True)
"""
    return task.cancel()


async def sleep(ms: int) -> None:
    """Pauses the coroutine for `ms` milliseconds without blocking the
loop.

## Parameters

  - ms: Milliseconds to pause.

    >>> run(sleep(1))
"""
    return (await asyncio.sleep(ms / 1000))


async def completed(value: object) -> object:
    """An awaitable already holding `value`, for APIs that must return one
uniformly.

## Parameters

  - value: The value the awaitable holds.

    >>> run(completed("ok"))
    'ok'
"""
    return value


async def demo() -> tuple:
    """The doctests' tour of the coroutine world in one coroutine: spawn
two tasks, join them in order, cancel a third before it finishes —
`({joined}, {cancelled})`.

    >>> run(demo())
    ([1, 2], True)
"""
    a = async__kw(completed(1))
    b = async__kw(completed(2))
    joined = (await all([a, b]))
    c = async__kw(sleep(60000))
    return (joined, shutdown(c))
