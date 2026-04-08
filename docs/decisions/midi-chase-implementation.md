# 决策：MIDI Chase 实现方案

最后更新：2026-04-07-14-31-00

---

## 结论

采用**实时线性搜索方案**，弃用预计算的 StateTable。

- 播放跳转时实时向前搜索 CC/PC/PB 最新状态
- 零预存储内存，支持任意规模 MIDI 文件
- 单次搜索最坏情况 O(N)，100万事件约 10ms

---

## 背景

MIDI Chase（光标追踪）功能用于解决以下问题：

当用户在 DAW 中跳转播放位置（scrub、循环、暂停后恢复）时，MIDI 通道可能处于不确定状态：
- Bank Select + Program Change 决定了当前音色
- Volume/Expression 控制当前音量
- Pitch Bend 控制弯音偏移
- Sustain 控制延音踏板

如果跳转后不及时恢复这些状态，第一个音符会以错误的音色/音量/弯音发出。

---

## 被否决方案：StateTable 预计算

### 设计

```rust
struct StateTable {
    cc: Vec<Vec<Vec<(u64, u8)>>>,  // [channel][cc_num][(tick, value)]
    pc: Vec<Vec<(u64, u8)>>,       // [channel][(tick, value)]
    pb: Vec<Vec<(u64, i16)>>,      // [channel][(tick, value)]
}
```

- MIDI 加载时预计算所有历史状态
- Chase 时使用二分查找定位

### 否决原因

1. **内存占用不可控**：256通道 × 128CC × N事件点
2. **实现复杂**：需要维护增量更新逻辑
3. **性能收益有限**：二分查找 O(logN) vs 线性 O(N)，但常数因子大

---

## 采用方案：实时线性搜索

### 设计

```rust
fn chase_events(&self, target_tick: u64) -> Vec<MidiEvent> {
    // 1. 分配状态缓存
    let mut cc_state = vec![[None; 128]; NUM_CHANNELS];
    let mut pc_state = vec![None; NUM_CHANNELS];
    let mut pb_state = vec![None; NUM_CHANNELS];
    
    // 2. 线性扫描到 target_tick 之前
    for event in &self.events {
        if event.tick >= target_tick { break; }
        match event.message {
            ControlChange { cc, value } => cc_state[ch][cc] = Some(value),
            ProgramChange { pc } => pc_state[ch] = Some(pc),
            PitchBend { value } => pb_state[ch] = Some(value),
            _ => {}
        }
    }
    
    // 3. 生成 Chase 事件
    // ...
}
```

### 优点

1. **零预存储内存**：只依赖原始 `events` 列表
2. **代码简单**：无预处理、无增量更新、无复杂索引
3. **支持任意规模**：不受内存限制
4. **单次扫描**：跳转时只扫描一次，不是每 buffer 都扫描

### 缺点

1. **单次延迟**：100万事件最坏约 10ms
2. **无上限保证**：极端超大文件可能超出 buffer 时间

### 缓解措施

- 100万事件约 10ms，512sample buffer @ 44.1kHz = 11.6ms，刚好满足
- 如需要可限制搜索范围（`target_tick - 50000`），牺牲极端旧状态换取性能

---

## Chase 触发时机

```rust
pub fn process(...) {
    // 1. 暂停恢复时 Chase
    if is_playing && !self.was_playing {
        engine.all_notes_killed();
        self.chase_and_send(current_tick, engine);
    }
    
    // 2. 播放头跳转时 Chase
    if tick_diff > threshold {
        engine.all_notes_killed();
        self.chase_and_send(current_tick, engine);
    }
    
    // 3. 正常分发事件...
}
```

**注意**：暂停时不 Chase（很多 DAW 暂停时不调用 `process`），依赖播放时的 Chase。

---

## Chase 事件顺序

按以下顺序发送，确保依赖关系正确：

1. RPN MSB (101) + RPN LSB (100)
2. Data Entry MSB (6) + Data Entry LSB (38)
3. Bank Select MSB (0) + Bank Select LSB (32)
4. Program Change
5. Volume (7) / Expression (11) / Pan (10)
6. 其他 CC
7. Pitch Bend

---

## 适用版本

v0.0.1 及以后