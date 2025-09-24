## 实现功能
在 TaskManner 里加入了一个 BTreeMap 数组用于记录每个应用系统调用的次数。

实现了系统调用 trace 用于读写特定内存数据和查询系统调用次数记录。

## 问答

### 1. 三个 bad 测例

```
[kernel] PageFault in application, bad addr = 0x0, bad instruction = 0x804003a4, kernel killed it.
[kernel] IllegalInstruction in application, kernel killed it.
[kernel] IllegalInstruction in application, kernel killed it.
```
触发了对应错误的 trap。

### 2 trap.S

1. 刚进入 __restore 时 sp 代表系统栈的栈指针。两种使用场景是从 trap 中恢复和开始运行第一个程序。
2. 处理了 csr 寄存器，这些寄存器存储了特权级相关设置，得恢复。
3. 因为 x2 x4 是 sp 和 tp
4. 换回了用户栈
5. sret，执行后从 S 特权级返回
6. 换上系统栈
7. ecall 后发生的