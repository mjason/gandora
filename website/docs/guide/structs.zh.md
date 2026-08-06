# 结构体与注解

## 结构体

`defstruct` 声明一个冻结的数据类；字面量、模式匹配和更新操作都适用于它：

```elixir
defmodule App.User do
  defstruct name: nil, age: 0, tags: []
end

u = %App.User{name: "MJ"}               # frozen dataclass instance
%App.User{name: n} = u                  # pattern (isinstance + fields)
older = %App.User{u | age: u.age + 1}   # dataclasses.replace
m2 = %{m | count: 2}                    # plain-MAP update: {**m, ...}
```

`%{x | ...}` 用于普通映射；结构体值的更新需要使用结构体写法 `%Mod{x | ...}` — 构建工具会在两者混淆时提醒你。结构体类型在规格说明中表示为 `App.User()`。

## 文档注释

放在 `def` 之前的注释会累积到该定义上——包括 `@doc`、`@doc_trans`、`@param`、`@param_trans`、`@spec`、`@example`、`@decorate`、`@allow`：

```elixir
@doc "Word frequencies of a sentence, as a map."
@doc_trans zh_CN: "统计句子的词频，返回映射。"
@param sentence, "Case-folded and split on whitespace."
@param_trans sentence, zh_CN: "会转小写并按空白切分的句子。"
@spec word_count(string()) :: map(string(), integer())
@example """
    gan> word_count("the quick the")
    {'the': 2, 'quick': 1}
"""
def word_count(sentence), do: ...
```

- `@param` 的名称必须与子句头部的变量匹配——编译时验证。
- `@example` 是唯一的 doctest 通道：`gan test` 将 `gan>` 行作为原生 Python doctest 运行。预期输出是 Python 的 `repr`：原子打印为 `'ok'`，元组为 `('ok', 21)`，布尔值为 `True`。
- 标准：**每个公共 `def` 必须包含 `@doc` + `@spec`**；面向用户的接口需额外添加 `_trans` 对。构建的覆盖率报告会进行评分。


!!! note "没有 `## 示例` 章节"
    Elixir 中在 `@doc` 内部嵌入示例块的习惯不适用：`@doc` 仅用于说明文字。示例始终放在单独的 `@example` 属性中——在 `def` 上作为可运行的 doctest，在宏上作为显示的文档。构建系统会在遇到时自动教导这一规则。

文档的*语言*是开发者个人偏好，而非项目配置：`gandora.local.jsonc`（`{"docLocale": "zh-CN"}`，已被 `.gitignore` 忽略）→ `GAN_DOC_LOCALE` → 英语。

## 模块属性与装饰器

属性保存导入时的状态；`@decorate` 用于附加 Python 装饰器：

```elixir
@app $fastapi.FastAPI(title: "API")

@decorate @app.get("/")
def root(), do: %{hello: "world"}

@decorate $functools.lru_cache(maxsize: 64)
def fib(n), do: ...
```

生成的模块是一个普通的 ASGI 目标：`uvicorn app.api:app --app-dir dist`。

## 装饰器：两个层级

运行时装饰属于 `$` 世界；编译时装饰属于宏。具体来说：

- **`@decorate <expr>`** 将任何 Python 装饰器附加到下一个 def 上——可以是库的（`$functools.lru_cache(maxsize: 64)`、`@app.get("/")`），也可以是返回包装器的 Gandora 函数。多个装饰器可以堆叠；最靠近 def 的先包装，与 Python 类似。
- **Gandora 编写的包装器是参数数量精确的**（`fn x -> ... f.(x) end` 仅包装 1 个参数的函数）—— Gandora 有意没有 `*args`。*通用*的任意参数数量装饰器是 Python 的工作：将它放在源码旁边的 `.py` 文件中，并作为 `$mymod.deco` 引用。
- **编译时重写**——Elixir 风格的装饰器——是 `defattr :name` 加上一个 `@on_definition` 宏（GEP-0008）：它看到真实的函数头，保持零运行时开销，并且自身可以为 Python 侧发出 `@decorate`。教程中的 `@cache` 章节是示例。
- 由 Gandora 的 `fn` 构建的包装器底层是 lambda——它会丢失 `__name__`/`__doc__`；如果内省很重要，请用 Python 编写该装饰器。

## 错误处理

`try/rescue/after` 映射到 Python 异常；rescue 子句按异常类匹配，通过其模块拼写：

```elixir
try do
  risky()
rescue
  e in $builtins.ValueError -> {:bad_value, to_string(e)}
  e in $requests.HTTPError -> {:http, e.response.status_code}
  _e -> :error                     # deliberate catch-all
after
  cleanup()                        # always runs, contributes no value
end
```

`try` 是一个表达式。`try` 内部的尾调用*不*会被优化（栈帧必须为处理程序保留）—— 其中的 `recur` 是一个编译错误，而不是静默的栈消耗者。
