# 符号与嵌入式语言

## 经典用法

```elixir
~w(one two three)            # ["one", "two", "three"]
~s(no "escaping" needed)     # string, free delimiter choice
~r/\d+/                      # re.compile(r"\d+")
```

## 两个层级，两个符号

嵌入式内容根据主体*是什么*来划分（GEP-0009）：

**`~<name>` 是文本。** 任何名称——`~sql`、`~markdown`、`~html`等——都标记一个原始字符串模板。主体逐字节传递；`<%= expr %>`拼接运行时值；编辑器高亮内部语言：

```elixir
~sql"""
SELECT * FROM sales WHERE units >= <%= min_units %>
"""
~markdown(A report for <%= name %>)
```

**`$python(expr)` 是代码。** 一个逐字进入程序的Python表达式——`$`世界，与`$math.sqrt`相同：

```elixir
def evens_capped(xs, limit) do
  $python([x for x in <%= xs %> if x % 2 == 0][:<%= limit %>])
end
```

`$python`中的拼接是*编译后的Gandora表达式*；其他所有内容都是您的Python，不受影响。它是专为Python-only拼写提供的逃生舱口——极少使用。

## `~prompt` — 无需转义的 AI 散文

为提示词指定的受祝福的文本-sigil 名称（GEP-0009-R006）：引号、花括号、反斜杠和内联 JSON 均原始传递——再也不用与 `\\\"` 搏斗了：

```elixir
@task ~prompt(Parse the JSON string {"b": 2, "a": 1} and print its keys sorted.)

@system ~prompt"""
You are a coding agent. Reply with {"status": "ok"} when done.
The user's name is <%= name %>.
"""
```

## 数据并非内嵌——而是映射

刻意地**没有** JSON 字面量：Gandora 映射是唯一的数据写法（更丰富——原子键、任意表达式）。粘贴进来的 JSON 文档通过将 `:` 替换为 `=>` 变成映射（构建工具会即时识别此转换）；运行时的 JSON 文本使用 `$json.loads(s)`。

Heredoc（`"""`）体像普通 heredoc 一样取消缩进，因此模板在模块内自然缩进，同时生成左对齐的值。
