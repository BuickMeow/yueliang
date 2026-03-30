# 开发红线

## 音频线程绝对禁止

在 `engine/` 所有代码及 `lib.rs` 的 `process()` 函数内：

| 禁止项 | 原因 | 后果 |
|--------|------|------|
| `String` 操作 | 可能触发堆分配 | DAW 卡顿 |
| `Vec::push()` 等扩容操作 | 堆分配 | 爆音 (Buffer Underrun) |
| `Mutex` / `RwLock` 加锁 | 阻塞等待 | 音频中断 |
| File I/O | 阻塞操作 | 音频中断 |
| 打印日志 | 可能阻塞 | 卡顿 |

---

## 正确做法

### 内存预分配
```rust
// ❌ 错误：运行时分配
let mut events = Vec::new();
for e in midi_events {
    events.push(e);  // 可能触发扩容
}

// ✅ 正确：预分配固定大小
const MAX_EVENTS: usize = 1024;
let mut events: [MidiEvent; MAX_EVENTS] = [MidiEvent::default(); MAX_EVENTS];
let mut count = 0;
for e in midi_events {
    if count < MAX_EVENTS {
        events[count] = e;
        count += 1;
    }
}
```

### 线程通信
```rust
// ❌ 错误：加锁
let state = self.state.lock().unwrap();

// ✅ 正确：无锁交换
let state = self.state.load();
```

---

## 参数平滑

使用 `nih-plug` 的平滑器避免参数跳变：

```rust
// 参数定义时添加平滑器
gain: FloatParam::new(...)
    .with_smoother(SmoothingStyle::Logarithmic(50.0))

// 处理时获取平滑值
let gain = self.params.gain.smoothed.next();
```

---

## 采样率依赖

所有频率相关计算必须在 `initialize()` 中获取采样率：

```rust
fn initialize(&mut self, ..., buffer_config: &BufferConfig, ...) -> bool {
    self.sample_rate = buffer_config.sample_rate;
    // 计算滤波器系数、延迟线长度等
    true
}
```

---

## 调试技巧

音频线程不能打印，但可以：
- 使用原子变量记录状态
- 在 GUI 线程读取并显示
- 使用 `nih_plug` 的 `debug!()` 宏（仅在调试构建生效）

```rust
use nih_plug::debug;

fn process(...) {
    debug!("当前时间: {}", context.transport().pos_samples());
}
```
