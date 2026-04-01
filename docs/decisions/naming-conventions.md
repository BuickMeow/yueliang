# 命名规范决策

> 最后更新：2026-03-31  
> 决策：统一使用 `max_voices`（包含余韵）和 `max_simultaneous_notes`（同时按下）
>
> 人类开发者「节能酱」宣布该决策已经完成，最终使用max_voices，也是各黑乐谱软件统一的命名规范
> 不如说，节能酱压根就没听说过 `max_simultaneous_notes` 这个概念

---

## 问题

需要区分两个概念：
1. **同时按下的琴键数**（polyphony）
2. **正在播放的音频数**（包含余韵/包络释放阶段）

## 术语定义

| 术语 | 定义 | 常见叫法 |
|------|------|----------|
| `max_voices` | 引擎同时播放的最大音频数（含 ADSR 释放阶段）| voices, active voices |
| `max_simultaneous_notes` | 同时按下的琴键数（仅 NoteOn 状态）| polyphony, note poly |
| `max_layers` | 单个键触发的采样层数（同键多采样叠加）| layers, velocity layers |

## 关系

```
max_voices ≥ max_simultaneous_notes
```

因为一个琴键按下后，即使松开，余韵（Release 阶段）仍占用 voice。

### 示例

| 场景 | max_simultaneous_notes | max_voices |
|------|------------------------|------------|
| 按下 10 个键 | 10 | 10 |
| 松开 5 个键（余韵中）| 5 | 10（余韵还在）|
| 余韵结束 | 5 | 5 |

## XSynth 中的对应

```rust
// XSynth 内部统计
pub fn voice_count(&self) -> u64;  // 对应 max_voices

// XSynth 的 layer 限制
pub layers: Option<usize>;  // 对应 max_layers（每个键）
```

XSynth 没有直接的 "同时按下键数" 统计，需要自己维护。

## Yueliang 项目命名

### 参数命名

```rust
pub struct YueliangParams {
    /// 最大复音数（播放中的音频数，含余韵）
    #[id = "max_voices"]
    pub max_voices: IntParam,
    
    /// 同时按下的最大键数（仅统计，不限制）
    #[id = "max_notes"]
    pub max_simultaneous_notes: IntParam,
    
    /// 力度分层数（每键触发的采样层数）
    #[id = "layers"]
    pub velocity_layers: IntParam,
}
```

### 引擎内部命名

```rust
pub struct SynthEngine {
    /// XSynth 限制
    max_voices: usize,
    
    /// 监控用：当前同时按下的键数
    active_notes: AtomicUsize,
    
    /// 每个键的最大采样层
    max_layers: usize,
}
```

## 显示名称（DAW 中）

| 参数 ID | 显示名称 | 单位 |
|---------|----------|------|
| max_voices | Max Voices | voices |
| max_notes | Max Simultaneous Notes | notes |
| layers | Velocity Layers | layers |

## 决策理由

1. **voice** = 音频行业通用术语（采样器、合成器都用 voices）
2. **simultaneous_notes** = 比 polyphony 更明确（polyphony 有时也指 voices）
3. **layer** = SoundFont 标准术语（velocity layers）
4. 避免使用 `max_polyphony`（歧义大）

## 例外

如果用户习惯了 `polyphony` 叫法，可以在 UI 显示为：
- `"Polyphony (Voices)"` - 明确表示这是 voices 数
- `"Polyphony (Notes)"` - 明确表示这是按下键数
