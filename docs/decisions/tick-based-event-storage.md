# 决策：MIDI 事件使用 tick 而非 sample_offset 作为时间戳

最后更新：2026-04-02-14-24-11
当前阶段：v0.0.1

---

## 结论

`MidiEvent` 使用 `tick: u64` 作为时间戳，而不是 `sample_offset: usize`。

---

## 原因

- `sample_offset` 把"速度"固化在数据里，换算公式为：
  ```
  sample = tick / PPQN * (60 / BPM) * sample_rate
  ```
- 项目要求**不使用 MIDI 自带 BPM，而是使用 DAW 的实时 BPM**。如果事件存的是 `sample_offset`，DAW BPM 变化或播放头跳转时，所有事件的触发时间都会错乱。
- `tick` 是速度无关的音乐时间单位。在音频线程中，根据 `transport.tempo` 和 `transport.pos_beats()` 实时把 `tick` 映射到当前 buffer 范围，即可完美适配任意 BPM 变化和 scrub 操作。

---

## 被否决方案

在 `midi_loader` 中预计算 `sample_offset`：加载时计算简单，但失去 DAW 速度控制能力，且播放头跳转时需要重新计算整个事件流。

---

## 适用阶段/版本

v0.0.1，阶段 4 起启用。
