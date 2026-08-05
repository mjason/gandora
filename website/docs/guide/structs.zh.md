# 结构体与注解

## 结构体

`defstruct` 声明一个冻结的数据类；字面量、模式匹配和更新均可作用于其上：

```elixir
defmodule App.User do
  defstruct name: nil, age: 0, tags: []
end

u = %App.User{name: "MJ"}               # 冻结的数据类实例
%App.User{name: n} = u                  # 模式（类型检查 + 字段提取）
older = %App.User{u | age: u.age + 1}   # 数据类替换
m2 = %{m | count: 2}                    # 普通映射更新：{**m, ...}
```

`%{x | ...}` 用于普通映射；结构体值使用结构体拼写 `%Mod{x | ...}` 进行更新——编译器会在两者混淆时提醒你。结构体类型在规格说明中以 `App.User()` 形式出现。

## 文档注释

在 `def` 之前的注解会累积到该 `def` 上——包括 `@doc`、`@doc_trans`、`@param`、`@param_trans`、`@spec`、`@example`、`@decorate`、`@allow`：

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

- `@param` 名称必须与子句头部变量匹配——在编译时验证。
- `@example` 是唯一的 doctest 通道：`gan test` 会将 `gan>` 行作为原生 Python doctest 运行。预期输出为 Python 的 `repr`：原子输出为 `'ok'`，元组输出为 `('ok', 21)`，布尔值输出为 `True`。
- 标准：**每个公共 `def` 都应携带 `@doc` + `@spec`**；面向用户的接口应添加 `_trans` 对。构建的覆盖率报告会记录得分。

文档*语言*是开发者偏好，而非项目配置：`gandora.local.jsonc`（`{"docLocale": "zh-CN"}`，已被 git 忽略）→ `GAN_DOC_LOCALE` → 英语。

## 模块属性和装饰器

属性持有导入时的状态；`@decorate` 附加 Python 装饰器：

```elixir
@app $fastapi.FastAPI(title: "API")

@decorate @app.get("/")
def root(), do: %{hello: "world"}

@decorate $functools.lru_cache(maxsize: 64)
def fib(n), do: ...
```

生成的模块是一个普通的 ASGI 目标：`uvicorn app.api:app --app-dir dist`。

## 错误处理

`try/rescue/after` 映射到 Python 异常；rescue 子句通过异常类匹配，异常类通过其模块拼写：

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

`try` 是一个表达式。`try` 内部的尾调用 *不* 会被优化（帧必须为处理程序存活）——其中的 `recur` 是编译错误，而非静默的栈消耗者。
