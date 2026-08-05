# 结构体与注解

## 结构体

`defstruct` 声明一个冻结的数据类；字面量、模式匹配和更新操作均可在其上使用：

```elixir
defmodule App.User do
  defstruct name: nil, age: 0, tags: []
end

u = %App.User{name: "MJ"}               # 冻结的数据类实例
%App.User{name: n} = u                  # 模式匹配（isinstance + 字段）
older = %App.User{u | age: u.age + 1}   # dataclasses.replace
m2 = %{m | count: 2}                    # 普通映射更新：{**m, ...}
```

`%{x | ...}` 用于普通映射；结构体值使用结构体拼写 `%Mod{x | ...}` 更新——构建过程会在两者混淆时提醒你。结构体类型在规范中显示为 `App.User()`。

## 文档注解

在 `def` 之前的注解会累积到它身上——包括 `@doc`、`@doc_trans`、`@param`、`@param_trans`、`@spec`、`@example`、`@decorate`、`@allow`：

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

- `@param` 的名称必须与子句头部的变量匹配——在编译时验证。
- `@example` 是唯一的文档测试渠道：`gan test` 会将 `gan>` 行作为原生 Python 文档测试运行。预期输出是 Python 的 `repr`：原子打印为 `'ok'`，元组为 `('ok', 21)`，布尔值为 `True`。
- 标准：**每个公开的 `def` 都带有 `@doc` + `@spec`**；面向用户的接口额外添加 `_trans` 配对。构建的覆盖率报告会记录得分。

文档的*语言*是开发者偏好，而非项目配置：`gandora.local.jsonc`（`{"docLocale": "zh-CN"}`，已加入 gitignore）→ `GAN_DOC_LOCALE` → 英语。

## 模块属性与装饰器

属性保存导入时的状态；`@decorate` 附加 Python 装饰器：

```elixir
@app $fastapi.FastAPI(title: "API")

@decorate @app.get("/")
def root(), do: %{hello: "world"}

@decorate $functools.lru_cache(maxsize: 64)
def fib(n), do: ...
```

生成的模块是一个普通的 ASGI 目标：`uvicorn app.api:app --app-dir dist`。

## 装饰器：两层

运行时装饰属于 `$` 世界；编译时装饰属于宏。具体来说：

- **`@decorate <expr>`** 将任何 Python 装饰器附加到下一个 def 上——可以是库的（`$functools.lru_cache(maxsize: 64)`、`@app.get("/")`），也可以是返回包装器的 Gandora 函数。多个装饰器可以堆叠；最靠近 def 的那个优先包装，与 Python 中一样。
- 一个 **Gandora 编写的包装器是严格匹配参数数量的**（`fn x -> ... f.(x) end` 仅包装 1 个参数的函数）——Gandora 有意不提供 `*args`。*通用* 的任意参数装饰器是 Python 的工作：将其放在源文件旁边的 `.py` 文件中，并引用为 `$mymod.deco`。
- **编译时重写**——类 Elixir 风格的装饰器——是 `defattr :name` + 一个 `@on_definition` 宏 (GEP-0008)：它看到真实的函数头，保持零运行时开销，并且可以自身为 Python 端发出 `@decorate`。教程中的 `@cache` 章节是具体示例。
- 从 Gandora `fn` 构建的包装器底层是 lambda——它会丢弃 `__name__`/`__doc__`；如果内省很重要，请用 Python 编写该装饰器。

## 错误处理

`try/rescue/after` 映射到 Python 异常；rescue 子句通过异常类进行匹配，并通过模块路径拼写：

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

`try` 是一个表达式。`try` 内部的*尾调用*不会被优化（帧必须保留以处理处理程序）——其中的 `recur` 会导致编译错误，而非静默堆栈消耗。
