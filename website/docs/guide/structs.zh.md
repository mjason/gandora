# 结构体与注解

## 结构体

`defstruct` 声明一个冻结的数据类；字面量、模式匹配和更新操作均在其上工作：

```elixir
defmodule App.User do
  defstruct name: nil, age: 0, tags: []
end

u = %App.User{name: "MJ"}               # frozen dataclass instance
%App.User{name: n} = u                  # pattern (isinstance + fields)
older = %App.User{u | age: u.age + 1}   # dataclasses.replace
m2 = %{m | count: 2}                    # plain-MAP update: {**m, ...}
```

`%{x | ...}` 用于普通映射；结构体值的更新应使用结构体拼写形式 `%Mod{x | ...}`——若两者混淆，构建过程会提醒你。结构体类型在类型规范中表示为 `App.User.t()`。

## 文档注释

`def` 之前的注释会累积到它上面 —— 包括 `@doc`、`@doc_trans`、`@param`、`@param_trans`、`@spec`、`@example`、`@decorate`、`@allow`：

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

- `@param` 名称必须与子句头部的变量匹配 —— 编译时验证。
- `@example` 是唯一的 doctest 通道：`gan test` 将 `gan>` 行作为原生 Python doctest 运行。预期输出是 Python 的 `repr`：原子打印为 `'ok'`，元组打印为 `('ok', 21)`，布尔值打印为 `True`。
- 标准：**每个公开的 `def` 都带有 `@doc` + `@spec`**；面向用户的接口添加 `_trans` 成对注解。构建的覆盖率报告会记录分数。

文档*语言*是开发者偏好，而非项目配置：`gandora.local.jsonc`（`{"docLocale": "zh-CN"}`，被 git 忽略）→ `GAN_DOC_LOCALE` → 英语。

## 模块属性和装饰器

属性（Attributes）保存导入时状态；`@decorate` 附加 Python 装饰器：

```elixir
@app $fastapi.FastAPI(title: "API")

@decorate @app.get("/")
def root(), do: %{hello: "world"}

@decorate $functools.lru_cache(maxsize: 64)
def fib(n), do: ...
```

生成的模块是一个普通的 ASGI 目标：
`uvicorn app.api:app --app-dir dist`。

## 错误处理

`try/rescue/after` 映射到 Python 异常；rescue 子句通过异常类进行匹配，异常类通过其模块拼写：

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

`try` 是一个表达式。`try` 内部的尾调用*不会*被优化（框架必须为处理器保留）——其中的 `recur` 是一个编译错误，而非静默的栈消耗器。
