# 实时 MIDI 管线注意事项

最后更新：2026-04-03-15-55-09
当前阶段：v0.0.1

---

## 1. AllNotesOff vs AllNotesKilled

XSynth 提供了两个全通道静音事件，行为截然不同：

- `AllNotesOff`（MIDI CC#123）：让所有音符进入 **release 衰减阶段**。对于超高密度 Black MIDI，release 阶段会堆积大量 voice，导致暂停后仍然"嗡嗡响"，用户感觉"没生效"。
- `AllNotesKilled`（MIDI CC#120 / All Sound Off）：**立即杀死所有 voice**，没有任何 release，声音瞬间消失。

**推荐做法**：
- DAW **暂停**时发送 `AllNotesOff`，给音符一个自然衰减的机会。
- 从暂停**恢复播放**前发送 `AllNotesKilled`，彻底清除残留旧声，保证从新位置干净播放。
- DAW **Reset/Stop** 时也发送 `AllNotesKilled`，确保完全静音。

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

---

## 5. PitchBend 映射公式易错点

`midly` 的 `PitchBend::as_int()` 返回的是 **signed `i16`**，范围 `[-8192, 8191]`，中心点 `0` 表示无弯音。

**错误做法**（会导致严重跑调）：
```rust
let normalized = (*value as f32 - 8192.0) / 8192.0;  // ❌ 对 signed 值重复偏移
```

**正确做法**：
```rust
let normalized = *value as f32 / 8192.0;  // ✅ -1.0 ~ ~1.0
```

---

## 6. macOS 上 rfd 同步对话框与 baseview 冲突

在 `nih_plug_egui` 的 `update` 闭包中直接调用同步版 `rfd::FileDialog::new().pick_file()`，会在 macOS 上引发 `baseview` 事件循环与 `NSSavePanel runModal` 的 reentrant 冲突，导致 `SIGABRT` 崩溃。

**解决方案**：改用 `rfd::AsyncFileDialog` + 子线程自旋等待（`simple_block_on`），通过 `pending` 状态传回 UI 线程处理。详见 `docs/knowledge/nih-plug-egui-integration.md`。
