# AGENTS.md

`flat_cont`：一个 `no_std` Rust 库，核心作用是定义 “flat continuation” IR
（`src/flatcont.rs` 的 `FlatContGraph`）。`basicblock`（源 IR）和 `bb2flatcont`
（BBG->FlatCont 降级 pass，当前已禁用）是配套部分。

## 构建 / 测试
- `cargo build` / `cargo check` —— 仅 lib，可以编译。
- `cargo test` 目前无法编译（见 “过渡状态”）。
- 运行单个测试（修好后）：`cargo test --test test_linear_flow`。

## 过渡状态（改测试或改 pass 前必读）
- `src/bb2flatcont.rs` 已禁用：`src/lib.rs` 中 `pub mod bb2flatcont;` 被注释掉。
- `flatcont.rs` 已重写为新 IR（`Value`、`ValueRef`、`Effect`、`EffectPath`、
  `Function`、`ValueId`、`EffectId`）。`bb2flatcont.rs` 和两个 `tests/*.rs`
  仍用旧 API（`flatcont::Node`、`flatcont::NodeId`、`Value::Node`、
  `RawType::Token`、`FlatContinuation.nodes`），因此无法编译。
- `RawType`（src/common.rs）没有 `Token` 变体；旧代码引用了它。

## 约定
- `#![no_std]`：集合类型从 `alloc::` 导入（`Vec`、`String`、`Box`），绝不用
  `std`。Edition 2024。
- 除非解释非显而易见的不变量，否则不加行内 `//` 注释；公开类型使用文档注释
  （`///`）。

## 架构
- `src/common.rs` —— 两个 IR 层共享的类型（`RawType`、`Opcode`、`ICond`、
  `StaticData`、各种 id 等）。
- `src/basicblock.rs` —— 源 IR：`BasicBlockGraph` = 全局 `nodes` 池 + `bbs`；
  SSA 值引用该池。
- `src/flatcont.rs` —— 目标 IR：由自包含 `FlatContinuation` 组成的 `Function`。
- 每一层都定义自己的 `Value`、`ContId`、`Terminator`；不要混用。测试按层用别名
  （`BbValue`/`FcValue`、`BbTerm`/`FcTerm`）。
- `bb2flatcont::bb_to_flat_cont` 是 BBG->FlatCont pass：两遍 forward-fold
  （Pass 1 把跨块依赖 push 到前驱 terminator args；Pass 2 通过 liveness
  end-map 传播 + 裁剪），然后把全局 `NodeId` 重映射为 continuation 内的本地索引。
- Effect token 走普通 SSA def-use 链；terminator args 里的 effect 名字仅用于调试，
  不参与路由。计划中有去函数化 `BBG->BBG` pass（见 git 历史里的
  `README.md`/`TODO.md`）。

## 说明
- `README.md` 和 `TODO.md`（中文设计文档）在工作树中已删除；可通过
  `git show HEAD:README.md` 找回。
