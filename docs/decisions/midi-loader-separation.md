# 决策：MIDI 文件解析与实时调度彻底分离

最后更新：2026-04-02-14-24-11
当前阶段：v0.0.1

---

## 结论

将 `midly` 解析逻辑放入 `data::midi_loader`（非实时模块），`engine::midi_player` 仅负责实时调度。

---

## 原因

- **实时安全**：音频线程（`process()`）严禁堆分配和文件 IO。`midly` 解析会大量分配 `Vec`。
- **职责单一**：`midi_loader` 只关心"如何把文件变成事件流"，`midi_player` 只关心"当前 buffer 该发哪些事件"。
- **可测试性**：`midi_player` 可以独立测试调度逻辑，只需传入构造好的 `Vec<MidiEvent>`，无需依赖真实 MIDI 文件。
- **可扩展性**：未来如果从网络流或内存加载 MIDI，只需要替换 `midi_loader` 的实现，无需改动 `midi_player`。

---

## 被否决方案

把 `load_from_file` 直接放在 `midi_player` 里：实现简单，但污染了实时调度器，增加了耦合，且容易在音频线程中误调用文件 IO。

---

## 适用阶段/版本

v0.0.1，阶段 4 起启用。
