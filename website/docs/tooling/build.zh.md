# 构建判决

`gan build` 是 Gandora 唯一的质量门（GEP-0025）：一条命令完成全部判断，然后进行编译。没有单独的 lint 步骤，没有需要记住的独立检查器——构建*就是*判决。

## 判决结果包含什么

1. **错误** — 程序无法编译（或无法运行：参见下面的工件验证）。构建在写入工件之前停止；退出码为1。
2. **警告** — 可静态证明但不阻止编译的事实：堆栈增长递归（GEP-0019）、未使用的绑定、不可达子句、丢弃的推导式（GEP-0022）。事实，而非观点。
3. **建议** — 顾问的教学通行证：最佳实践差距（`practice`）、跨语言习惯（`migration`）以及根据实际符号表检查的拼写错误名称（`did_you_mean`）。每条建议都带有其首次证据所在行和正确拼写。

```console
$ gan build
warning: src/prog.gan:8: fact/1 is self-recursive outside tail position ...
practice: src/prog.gan:3: Annotation coverage: missing @spec on: total ...
did_you_mean: src/prog.gan:12: `Enum.mpa` is not a function of Enum — did you mean `Enum.map`?
compiled 3 module(s)
```

## 工件验证

代码生成后，生成的 Python 代码将使用 [ty](https://docs.astral.sh/ty/) 进行检查，但仅基于 *解析规则*：未定义的名称、无法解析的导入、缺失的模块成员或参数数量错误的调用属于 **运行时致命性事实**，因此会报告为构建错误，并映射回你的 `.gan` 源行：

```console
error: src/prog.gan:13: Name `totl` used when not defined — did you mean `total`?
error: src/prog.gan:5: Cannot resolve imported module `requests` — `$x`/`pyimport x` needs an importable module
```

类型流 *观点* 永远不会阻塞构建。通过 `gan build --strict` 选择启用它们，它会将完整的 ty 分析报告为 `[type]` 警告。

## 交通信号灯

`gan lsc check` 返回一个 JSON 对象作为判决结果——即面向 AI 的界面：

```json
{"ok": true, "clean": false,
 "diagnostics": [...], "suggestions": [{"kind": "did_you_mean", "line": 3, ...}]}
```

- `ok: false` — **红色**：修复错误。
- `ok: true, clean: false` — **黄色**：应用所有建议。
- `clean: true` — **绿色**：可以发布。

## 零噪声信任线

一条规则只有在符合惯用代码时保持沉默，才能赢得在 Advisor 中的位置：Gandora 自身的标准库、工具链、示例教程和游乐场都在其自身构建下裁决为 `clean: true`。当裁决发言时，它值得一读——这就是契约。

跨多个文件的相同发现合并为一个带注释的条目；测试模块免除库注释覆盖；裁决覆盖 `src/` 加上顶层 `tests/*.gan`——正是 `gan test` 运行的内容。
