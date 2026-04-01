# 振荡器相位连续性原理

> 最后更新：2026-04-01  
> 问题：正弦波生成器输出出现卡顿/断裂声

---

## 问题现象

使用 `TestToneGenerator` 生成正弦波时，音频听起来"卡卡的"、不柔顺，有明显的断裂感。

## 错误代码

```rust
// 错误的实现
pub fn render(&mut self, buffer: &mut [f32]) {
    for sample in buffer.iter_mut() {
        *sample = self.phase.sin();
        self.phase += self.phase_increment;
    }
}

// 调用方式（错误！）
tone.render(&mut left);   // phase 前进到 position A
tone.render(&mut right);  // phase 从 A 继续，不是从头开始！
```

## 问题原因

### 相位（Phase）的概念

```
正弦波：y = sin(phase)

phase 是角度，范围 0 ~ 2π
        ↗ 峰值
       /  \
      /    \
_____/      \____  零点
            ↘    ↗
              谷值
```

### 发生了什么？

```
时间轴：

调用 render(&mut left):
[0] phase=0.0      → sin(0.0) = 0.0
[1] phase=0.1      → sin(0.1) = 0.1  
[2] phase=0.2      → sin(0.2) = 0.2
...
[N] phase=6.28     → sin(6.28) ≈ 0

此时 phase = 6.28（接近 2π）

调用 render(&mut right):
[0] phase=6.28     → sin(6.28) ≈ 0   ← 断裂！应该是 0.0
[1] phase=6.38     → sin(6.38) ≈ -0.1  ← 不连续！
```

**右声道从 6.28 开始，而不是 0.0，导致左右声道相位不一致！**

## 正确的解决方案

### 方案 1：使用同一个 phase 生成立体声（推荐）

```rust
pub fn render_stereo(&mut self, left: &mut [f32], right: &mut [f32]) {
    for i in 0..left.len() {
        let sample = self.phase.sin();
        left[i] = sample;
        right[i] = sample;  // 同一时刻相同的相位
        
        self.phase += self.phase_increment;
    }
}
```

### 方案 2：为每个声道独立维护 phase

```rust
pub struct TestToneGenerator {
    left_phase: f32,
    right_phase: f32,
    // ...
}

pub fn render_left(&mut self, buffer: &mut [f32]) {
    for sample in buffer.iter_mut() {
        *sample = self.left_phase.sin();
        self.left_phase += self.phase_increment;
    }
}

pub fn render_right(&mut self, buffer: &mut [f32]) {
    for sample in buffer.iter_mut() {
        *sample = self.right_phase.sin();
        self.right_phase += self.phase_increment;
    }
}
```

### 方案对比

| 方案 | 特点 | 适用场景 |
|------|------|----------|
| 方案 1 | 单声道信号复制到立体声 | 测试音、简单的振荡器 |
| 方案 2 | 真正的立体声（可添加声像、立体声效果）| 复杂合成器 |

## 实际应用中的注意事项

### 1. 跨 buffer 的连续性

```rust
// 第一个 buffer
render_stereo(&mut left1, &mut right1);  // phase 结束在 6.28

// 第二个 buffer  
render_stereo(&mut left2, &mut right2);  // 自动从 6.28 继续，✅ 正确！
```

**同一个 `render_stereo` 调用内部保持连续，跨调用也保持连续。**

### 2. reset() 方法

```rust
pub fn reset(&mut self) {
    self.phase = 0.0;  // 播放停止时重置相位
}
```

### 3. 频率变化时的处理

如果运行时改变频率，`phase_increment` 会变化，但 `phase` 应该保持连续：

```rust
pub fn set_frequency(&mut self, freq: f32) {
    // 只更新增量，不改变当前 phase
    self.phase_increment = 2.0 * PI * freq / self.sample_rate;
}
```

## 总结

| 错误做法 | 正确做法 |
|---------|---------|
| 多次调用 `render()` 共享同一个 phase | 一次调用生成所有声道，或每个声道独立 phase |
| 假设每次调用都从头开始 | 维护状态，确保采样点级别的连续性 |

**核心原则**：数字音频是离散的，但模拟的模拟信号必须是连续的。phase 就是维持这种连续性的状态。
