# MIDI Chase 行为说明

最后更新：2026-04-07-14-32-00

---

## Chase 是什么

MIDI Chase 是在播放位置跳转时，自动恢复 MIDI 通道状态（CC/PC/PB）的机制，防止错音。

---

## 触发条件

Chase 在以下情况触发：

1. **播放恢复**：从暂停状态开始播放
2. **播放头跳转**：用户手动 scrub 或 DAW 循环跳转

```rust
if (current_tick - last_tick).abs() > ppqn * 0.5 {
    // 触发 Chase
}
```

---

## 不触发的情况

### 暂停时光标移动不 Chase

很多 DAW（Ableton、FL Studio）在暂停时会**停止调用**插件的 `process()`，因此无法检测光标移动。

**表现**：用户在暂停时移动光标，听不到音色变化；按播放后才 Chase。

**这不是 bug**，是 VST 架构限制。

---

## 支持的 Chase 事件

### CC（按顺序）

| CC | 名称 | 说明 |
|----|------|------|
| 101 | RPN MSB | 弯音范围等 RPN 参数 |
| 100 | RPN LSB | |
| 6 | Data Entry MSB | RPN 值设置 |
| 38 | Data Entry LSB | |
| 0 | Bank Select MSB | 音色库选择 |
| 32 | Bank Select LSB | |
| 7 | Volume | 通道音量 |
| 10 | Pan | 声像 |
| 11 | Expression | 表情（相对音量） |
| 64 | Sustain | 延音踏板 |
| 73 | Attack |  attack 时间 |
| 72 | Release | release 时间 |
| 74 | Cutoff | 滤波器截止频率 |
| 71 | Resonance | 滤波器共振 |

### PC

Program Change 会在 Bank Select 之后发送。

### PB

Pitch Bend 最后发送，因为其他 CC 可能影响它的效果。

---

## 性能特征

| 事件数量 | 估计扫描时间 | buffer 512@44.1kHz |
|---------|-------------|-------------------|
| 1万 | ~0.1ms | 安全 |
| 10万 | ~1ms | 安全 |
| 100万 | ~10ms | 临界 |
| 500万 | ~50ms | 可能超时 |

**超时后果**：该 buffer 的音频延迟，DAW 会补静音，用户听到卡顿。

---

## 故障排查

### 跳转后音色错误

**可能原因**：
1. Bank Select 和 PC 之间有其他事件干扰
2. SF2 音色库不支持 Bank Select

**排查**：检查 MIDI 文件在跳转位置的 Bank/PC 事件序列。

### 跳转后音量异常

**可能原因**：Volume/Expression 在跳转前被设置为 0。

**排查**：检查 MIDI 文件的 Volume 曲线。

### 跳转后有卡顿

**可能原因**：MIDI 文件过大，Chase 扫描超时。

**解决**：减小 MIDI 文件规模，或限制搜索范围（修改 `CHASE_MAX_SEARCH_TICKS`）。

---

## 与其他 DAW 的兼容性

| DAW | 暂停时 Chase | 说明 |
|-----|-------------|------|
| Reaper | ✅ 支持 | 暂停时仍调用 process |
| Ableton Live | ❌ 不支持 | 暂停时停止调用 |
| FL Studio | ❌ 不支持 | 暂停时停止调用 |
| Cakewalk | ✅ 支持 | 暂停时仍调用 process |

无论暂停时是否支持，播放时都会正确 Chase。