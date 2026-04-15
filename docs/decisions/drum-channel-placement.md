# 决策：Drum 模式状态跟踪放在何处

## 结论

将 `last_drums: [bool; 256]` 与 `last_mutes` 一起放在 `engine/midi_player.rs` 的 `MidiPlayer` 结构体中，由 `MidiPlayer::process()` 统一检测变化并发送 XSynth `SetPercussionMode` 事件。

## 原因

1. **职责一致性**：`MidiPlayer::process()` 已经是"通道参数变化 → XSynth 事件"的协调中心。`last_mutes` 的检测（AllNotesOff、Chase） already 在此处处理，`last_drums` 的变化（SetPercussionMode）与之同类。
2. **减少 `lib.rs` 复杂度**：`lib.rs` 只需负责从参数锁中读取 `drums` 数组并传递给 `midi_player.process()`，无需额外维护状态字段。
3. **音频线程安全**：所有对 `SynthEngine` 的状态同步事件都集中在同一函数中发送，避免分散在多个模块导致时序混乱。

## 被否决方案

- **放在 `lib.rs` 的 `Yueliang` 中**：虽然也能工作，但会导致 `lib.rs` 需要同时跟踪 `last_mutes`（已在 `MidiPlayer` 中）和 `last_drums`（在 `lib.rs` 中），破坏状态集中管理的对称性。

## 适用阶段/版本

v0.0.2 阶段 6（Channel Matrix Drum 模式实现期间）
