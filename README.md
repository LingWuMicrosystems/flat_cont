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

## 去函数化（Defunctionalization）

将整个程序或模块拍扁成一个巨大的状态机函数，用显式的全局控制流替代原本的函数调用。这是一个 `BBG → BBG` 的 Pass，紧邻 `BBG → FlatCont` 之前执行。

### 第一步：函数识别与 State_ID 分配

扫描整个 BBG，提取所有函数（Function），为每个函数的入口块分配一个唯一的 `State_ID`。

### 第二步：构建 Dispatcher

创建一个 `Dispatcher` 块，内部是一个巨大的 Switch 结构，根据当前的 `State_ID` 跳转到对应函数的入口块。

### 第三步：处理 Call 指令

遇到 `Call` 指令时，将其拆分为前后两半：

**前半段（调用点）：**
1. 将当前函数的 `Live-Out` 变量 Store 到 Context 内存中。
2. 将当前块的地址（返回地址）设为 `Return_ID`，写入 Context。
3. 将 `State_ID` 设为被调用函数的入口 `State_ID`。
4. 生成一条跳转边，指向 `Dispatcher`。

**后半段（调用返回点）：**
1. 这是一个全新的 BBG Block。为它分配一个新的 `State_ID`。
2. 在块开头生成 `Load` 指令，从 Context 内存中恢复之前的 `Live-Out` 变量。
3. 继续执行原有的后续逻辑。

### 第四步：处理 Return 指令

当遇到原来的 `Return` 指令时，不再执行硬件的 Ret：
1. 从 Context 中读取 `Return_ID`。
2. 将该 ID 设为 Next State。
3. 生成一条跳转边，指向 `Dispatcher`。

### 第五步：清理

完成重写后，旧的 `Call` 和 `Return` 指令被彻底从 BBG 中删除。整个程序变成一张巨大的、互相交织的控制流图。

### 工程注意事项

**Pass 的执行时机（极度重要）：**

这个 `BBG → BBG` 的状态机重写 Pass，必须放在**所有常规 BBG 优化（如死代码消除、常量折叠、循环展开等）全部完成之后**，也就是**紧挨着进入 `BBG → FlatCont` 之前**执行。

原因：转换后的 BBG 是一张充满了巨大 Switch 和复杂访存的图，原有的标准优化器根本看不懂这种代码，强行优化只会适得其反。

---

## BB → FlatCont 算法

输入：`BasicBlockGraph`（BasicBlock 列表 + 全局 Node 池）
输出：`FlatContGraph`（FlatContinuation 列表，每个 continuation 自包含）

### 两遍 forward fold

1. **Pass 1**：按拓扑序遍历 BB，发现跨 BB 的数据/效应依赖，将依赖 push 到前驱 BB 的 terminator args
2. **Pass 2**：继承前驱 terminator args 并传递到后继，用 liveness（end map）裁剪不再需要的值
3. **Remap**：全局 NodeId → 本地 continuation 内索引，函数参数和效应参数从 body 中分离到 params/effects 字段，body 只保留计算节点
