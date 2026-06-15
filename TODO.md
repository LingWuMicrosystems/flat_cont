# TODO

- [ ] 去函数化（Defunctionalization）：实现 `BBG → BBG` 状态机重写 Pass，将函数调用转换为显式全局控制流。必须在所有 BBG 优化之后、`BBG → FlatCont` 之前执行。
