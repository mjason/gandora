# 并发

Gandora 的协程世界就是 Python 的协程世界，以母语表达：`async def`
与 `await` 一比一编译为 Python 自身的语法，`Task` 模块则是 `asyncio`
之上的薄封库——返回裁决的截止线、保序汇合、有界扇出，以及一座供阻塞
代码通行的桥（GEP-0029、GEP-0030）。

```elixir
defmodule Fetcher do
  async def fetch_both(a, b) do          # → async def fetch_both(a, b):
    ta = Task.async(fetch(a))            # 派生：asyncio.ensure_future
    tb = Task.async(fetch(b))
    await Task.all([ta, tb])             # 按输入顺序汇合
  end

  async defp fetch(url) do
    resp = await http.get(url)           # → await http.get(url)——裸等
    resp.body
  end

  def main() do
    IO.puts(Task.run(fetch_both(a, b)))  # rim 入口：asyncio.run
  end
end
```

`main` 保持同步；程序的整个异步部分悬挂在一次 `Task.run` 之下。调用
async 函数返回一个协程——在被 await 或派生之前保持惰性。

## await

`await expr` 是前缀表达式，编译为 Python 的裸 `await`：**无截止线，
无包装**。它比一切二元运算符结合更紧——`await fetch(u) |> parse()`
管道传递的是 await 后的值——且仅在 async 体内合法：在体外、或在
`fn` 闭包内（Python 的 lambda 无法 await）都是编译错误。推导式体内
没问题：`for t <- ts, do: await t` 发射为原生的
`[await t for t in ts]`。

## Task 库

| 调用 | 含义 |
| --- | --- |
| `Task.run(coro)` | 在同步代码中运行协程——rim 入口 |
| `Task.async(coro)` | 派生；返回 task 句柄 |
| `await t` | 汇合——语言语法，不是库调用 |
| `Task.try_await(t, ms)` | 带截止线的汇合：`{:ok, v}` / `{:error, :timeout}` / `{:error, e}` |
| `Task.all(ts)` | 按输入顺序取值（须 await） |
| `Task.try_all(ts, ms)` | 整组一条截止线，每个 task 一个裁决 |
| `Task.async_stream(xs, fun, max)` | 有界扇出，结果保序 |
| `Task.blocking(fun)` | 在工作线程上运行阻塞函数 |
| `Task.map / tap / recover` | 可等待对象上的惰性组合子 |
| `Task.race(ts)` | 最先落定者，值或崩溃 |
| `Task.shutdown(t)` | 取消——运行中的协程 task 真的会停 |
| `Task.sleep(ms)` / `Task.completed(v)` | 暂停；已持有的值 |

超时单位是**毫秒**，同 Elixir。截止线是裁决而非异常——超时后 task
被取消，即 `asyncio` 自身的 `wait_for` 语义：

```elixir
case await Task.try_await(task, 5000) do
  {:ok, value} -> value
  {:error, :timeout} -> fallback        # task 已被取消
  {:error, e} -> handle(e)              # 崩溃，作为值
end
```

## 阻塞代码走桥

线程不再是第二个世界，只是一条边：`Task.blocking(fn ->
requests.get(url) end)` 返回一个在工作线程上运行闭包的可等待对象
（`asyncio.to_thread`）。派生它即可让阻塞工作与协程重叠：

```elixir
async def mixed() do
  a = Task.async(Task.blocking(fn -> slow_io() end))
  b = Task.async(compute())
  await Task.all([a, b])
end
```

## doctest 从 rim 进入

async 函数上的 `@example` 就是普通 doctest——借 `Task.run` 书写，
它会像其他一切一样在 `gan test` 下真实执行：

```elixir
@example """
    gan> Task.run(Fetcher.fetch_both("a", "b"))
    ['A', 'B']
"""
```

## 构建保证什么

async 体外的 `await`、`fn` 内的 `await`、`def` 与 `async def` 混写
的多子句、以及 `async def main`，都是编译错误。工件就是评审者会亲手
写出的 Python——带裸 `await` 的 `async def`——且它的 doctest 真的
跑过。
