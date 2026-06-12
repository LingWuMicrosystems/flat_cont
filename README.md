# Flat Continuation IR

将 BasicBlock-based IR 转换为 flat continuation 形式。

## Effect Chain 设计

### 核心原则：Effect 即 SSA def-use 链

Effect token 和普通数据值走同一套 SSA 机制。Load/Store 消耗旧 effect token、产出新 token（通过 Proj 访问）。算法使用标准的 def-use 活性分析传播 effect，不需要特殊的路由或替换逻辑。

### 多链并行（Multi-effect for Cross-block MLP）

Continuation 支持多条独立的 effect chain。每条 chain 对应一个别名类（TBAA），前端通过构造独立的 chain 表示 noalias——chain 的拓扑本身就是"不冲突"的证明，不需要元数据标注。

```
chain_a: Load → Store  ─→  continuation 2
chain_b: Load → Store  ─→  continuation 2
```

### Barrier 点

**Call**：cont call 接受多条 effect 入链，内部完全不透明。所有活跃 effect 链在 call 处汇聚，作为保序点。Callee 通过 `FlatContinuation.effects` 声明所需链数，调用方按签名匹配传入。

**Return**：对称于 Call。Callee 的最终 effect 状态通过 Return 传回调用方的 Call 的 Proj 上。

### 无需单独的 EffectMerge 节点

Barrier（Call/Return）本身就是隐式汇合点。分支合并走正常的 terminator args 传递，每条链在 continuation 之间各走各的。不需要专门的 merge IR 指令。

### Effect 名称是辅助标记

`Terminator.effect_args: Vec<(String, NodeId)>` 中的名字仅为调试和可读性，算法不依赖名称做路由。Effect chain 的结构由前端通过 `effect_state` 字段显式构造。

## BB → FlatCont 算法

输入：`BasicBlockGraph`（BasicBlock 列表 + 全局 Node 池）
输出：`FlatContGraph`（FlatContinuation 列表，每个 continuation 自包含）

### 两遍 forward fold

1. **Pass 1**：按拓扑序遍历 BB，发现跨 BB 的数据/效应依赖，将依赖 push 到前驱 BB 的 terminator args
2. **Pass 2**：继承前驱 terminator args 并传递到后继，用 liveness（end map）裁剪不再需要的值
3. **Remap**：全局 NodeId → 本地 continuation 内索引，函数参数和效应参数从 body 中分离到 params/effects 字段，body 只保留计算节点
