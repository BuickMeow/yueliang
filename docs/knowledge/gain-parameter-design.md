# Gain 参数设计详解

> 最后更新：2026-04-01  
> 问题：为什么 -30dB ~ +30dB 的 Gain 旋钮默认位置不在中间？

---

## 核心概念：dB vs Gain

### 数学关系

```
gain = 10^(dB / 20)

dB   = 20 * log10(gain)
```

**这是非线性的对数关系！**

### 数值对照表

| dB  | Gain (倍数) | 感知音量 |
|-----|------------|----------|
| -60 | 0.001      | 几乎静音 |
| -30 | 0.032      | 很小 |
| -20 | 0.1        | 小 |
| -12 | 0.25       | 较小 |
| -6  | 0.5        | 一半 |
| 0   | 1.0        | 原始音量 |
| +6  | 2.0        | 2倍 |
| +12 | 4.0        | 4倍 |
| +20 | 10.0       | 10倍 |
| +30 | 31.6       | 31.6倍 |

**关键发现**：
- -30dB 到 0dB：gain 从 0.032 变到 1.0（增长 31 倍）
- 0dB 到 +30dB：gain 从 1.0 变到 31.6（也增长 31 倍）
- **但视觉上，-30dB 到 0dB 的范围应该和 0dB 到 +30dB 一样大！**

---

## nih-plug 的 FloatRange 类型

### 1. Linear（线性）

```rust
FloatRange::Linear { min: -30.0, max: 30.0 }
```

**特点**：
- 旋钮位置与数值线性对应
- -30dB 在最左，0dB 在中间偏左（约 33% 位置），+30dB 在最右

**问题**：
- dB 值线性变化，但**音量感知不是线性的**
- 从 -30dB 调到 -20dB，gain 只从 0.032 变到 0.1（变化很小）
- 从 +6dB 调到 +16dB，gain 从 2.0 变到 6.3（变化很大）

### 2. Skewed（倾斜/对数）

```rust
FloatRange::Skewed {
    min: util::db_to_gain(-30.0),  // 0.032
    max: util::db_to_gain(30.0),   // 31.6
    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
}
```

**特点**：
- 旋钮位置与 **gain 值** 线性对应
- 不是与 dB 值线性对应

**换算关系**：
```
旋钮位置 0%  → gain = 0.032  → dB = -30
旋钮位置 50% → gain = 1.0    → dB = 0
旋钮位置 100%→ gain = 31.6   → dB = +30
```

**优点**：
- 0dB 正好在旋钮中间！
- 音量变化更自然（人耳感知是对数的）

### 3. Reversed（反向）

很少用于 Gain，主要用于滤波器等反向参数。

---

## 推荐的 Gain 参数写法

### 方案 A：Skewed Range（推荐用于专业音频）

```rust
use nih_plug::util;

gain: FloatParam::new(
    "Gain",
    util::db_to_gain(0.0),  // 默认值 1.0（0dB）
    FloatRange::Skewed {
        min: util::db_to_gain(-60.0),  // 最小 -60dB
        max: util::db_to_gain(12.0),   // 最大 +12dB
        factor: FloatRange::gain_skew_factor(-60.0, 12.0),
    },
)
.with_smoother(SmoothingStyle::Logarithmic(50.0))
.with_unit(" dB")
// 显示为 dB
.with_value_to_string(formatters::v2s_f32_gain_to_db(2))
// 从字符串解析 dB
.with_string_to_value(formatters::s2v_f32_gain_to_db()),
```

**使用**：
```rust
fn process(&mut self, buffer: &mut Buffer, ...) {
    let gain = self.params.gain.smoothed.next();  // 直接是 gain 值
    // 不需要 db_to_gain 转换！
}
```

**特点**：
- ✅ 0dB 在旋钮中间
- ✅ 音量变化符合人耳感知
- ✅ 显示为 dB，但存储为 gain
- ⚠️ 需要 `formatters` 转换显示

### 方案 B：Linear Range（简单直观）

```rust
gain: FloatParam::new(
    "Gain",
    0.0,  // 默认 0dB
    FloatRange::Linear { min: -30.0, max: 30.0 },
)
.with_unit(" dB")
.with_value_to_string(formatters::v2s_f32_gain_to_db(2)),
```

**使用**：
```rust
fn process(&mut self, buffer: &mut Buffer, ...) {
    let gain_db = self.params.gain.smoothed.next();
    let gain = util::db_to_gain(gain_db);  // 需要手动转换
}
```

**特点**：
- ✅ 简单易懂
- ✅ 数值就是 dB
- ❌ 0dB 不在中间（-30~30 范围下，0 在 50% 位置...等等，其实应该在中间？）

**等等，让我重新计算**：

Linear Range -30.0 ~ 30.0：
- 最小值：-30.0
- 最大值：30.0
- 默认值：0.0
- 位置：(0.0 - (-30.0)) / (30.0 - (-30.0)) = 30 / 60 = 50%

**0dB 确实在中间！**

但问题在于：**音量感知不是线性的**

从 -30dB 调到 -10dB：
- dB 变化：20dB（旋钮移动 33%）
- gain 变化：0.032 → 0.316（10倍）

从 -10dB 调到 +10dB：
- dB 变化：20dB（旋钮移动 33%）
- gain 变化：0.316 → 3.16（也是10倍）

**线性 dB 在数学上是对的，但人耳感觉 "左边变化小，右边变化大"**

### 方案 C：百分比形式（最简单）

```rust
gain: FloatParam::new(
    "Gain",
    1.0,  // 100%
    FloatRange::Linear { min: 0.0, max: 2.0 },  // 0% ~ 200%
)
.with_unit(" %")
.with_value_to_string(formatters::v2s_f32_percentage(0)),
```

**特点**：
- ✅ 最简单
- ✅ 100% 在中间
- ❌ 不符合音频行业习惯（都用 dB）

---

## 为什么你的 Gain 默认位置不对？

**你的代码**：
```rust
FloatRange::Linear {
    min: util::db_to_gain(-30.0),  // 0.032
    max: util::db_to_gain(30.0),   // 31.6
}
```

**问题**：
- 范围是 0.032 ~ 31.6
- 默认值是 `util::db_to_gain(0.0)` = 1.0
- 位置：(1.0 - 0.032) / (31.6 - 0.032) = 0.968 / 31.568 ≈ 3%

**1.0 在这个范围里非常接近最小值！**

**修复**：使用 Skewed Range，或者用 Linear 但范围是 dB 值不是 gain 值。

---

## 总结对比

| 方案 | 默认位置 | 音量感知 | 代码复杂度 | 推荐度 |
|------|---------|---------|-----------|--------|
| Skewed (gain) | ✅ 中间 | ✅ 对数 | 中等 | ⭐⭐⭐⭐⭐ |
| Linear (dB) | ✅ 中间 | ❌ 线性 | 简单 | ⭐⭐⭐ |
| Linear (gain) | ❌ 偏左 | ✅ 对数 | 简单 | ⭐ |
| Percentage | ✅ 中间 | ❌ 线性 | 最简单 | ⭐⭐ |

---

## 最终推荐

**专业音频插件**：用方案 A（Skewed Range）

```rust
.with_value_to_string(formatters::v2s_f32_gain_to_db(2))
```

这会显示 "0.00 dB"、"-6.02 dB" 等，符合行业标准。

**简单测试用**：用方案 B（Linear dB）

```rust
FloatRange::Linear { min: -30.0, max: 30.0 }
```

然后在 process 中手动 `db_to_gain()`。
