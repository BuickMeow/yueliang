# 实时 MIDI 管线注意事项

最后更新：2026-04-02-14-24-11
当前阶段：v0.0.1

---

## 1. 走带停止时必须切断所有声音

当 DAW 停止播放时，如果之前处于 `playing` 状态，必须立即向所有 XSynth 通道发送 `AllNotesOff`。否则残留音符会继续发声，造成"测试音关不掉"或 MIDI 循环时音符悬停的现象。

**实现方式**：在 `midi_player.process()` 中检测 `!transport.playing && self.was_playing`，调用 `engine.reset()`。`synth.rs` 的 `reset()` 会遍历 `0..NUM_CHANNELS` 发送 `AllNotesOff`。

---

## 2. Scrub / 播放头跳转检测

用户使用 DAW 的 scrub 功能或循环回放时，`transport.pos_beats()` 会发生跳变。需要在 `midi_player` 中检测这种跳变，并用二分查找（`partition_point`）快速重置 `event_index`，避免漏播或重复播放。

**阈值设定**：当 tick 差值超过 1 个 beat（即 `ppqn`）时，认为发生了跳转。

**性能**：`partition_point` 是 `O(log n)`，对于百万级事件也能在微秒级完成。

---

## 3. 力度过滤在映射前执行

`midi_filter::apply_filter` 在 `midi_mapper::map_midi_event` 之前调用。这样可以：
- 减少发给 XSynth 的事件数量
- 避免被过滤的音符触发无意义的 voice 分配
- 降低 XSynth 内部振幅计算开销

---

## 4. NoteOn(velocity=0) 等价于 NoteOff

MIDI 标准中，`NoteOn { key, vel: 0 }` 是 `NoteOff` 的另一种写法。在 `midi_loader.rs` 中加载时必须做这层转换，否则会导致 XSynth 中对应音符永不释放。
