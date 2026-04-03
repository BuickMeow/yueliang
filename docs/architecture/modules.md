# 模块结构

> 最后更新：2026-04-03-15-55-09
> 当前阶段：v0.0.1

## 文件组织

```
src/
├── lib.rs              # 插件主入口
├── editor.rs           # UI 界面（egui + rfd 文件选择器）
├── engine.rs           # 模块聚合入口（pub mod / pub use）
├── engine/
│   ├── synth.rs        # xsynth 封装（渲染、音色加载）
│   ├── pipeline.rs     # 音频处理管线（gain、缓冲区写入）
│   ├── midi_player.rs  # MIDI 时间线调度器（走带同步、事件分发）
│   ├── midi_mapper.rs  # MIDI 协议映射层（纯函数）
│   └── midi_filter.rs  # MIDI 预过滤层（力度过滤、强制最大力度）
├── data.rs             # 数据模块聚合入口
├── data/
│   ├── event.rs        # MIDI 领域模型（MidiEvent / MidiMessage）
│   └── midi_loader.rs  # MIDI 文件解析器（midly 封装，非实时）
└── utils.rs            # 通用工具
```

---

## 模块职责

### lib.rs
- 插件主入口
- 实现 `Plugin` trait
- 定义 `Yueliang` 与 `YueliangParams`
- `process()` 仅负责"接线"：获取 Transport → 调用 `midi_player.process()` → 调用 `pipeline.render()`
- `initialize()` 根据持久化路径加载 SoundFont 和 MIDI（不再使用硬编码路径）

### editor
- 基于 `nih_plug_egui` 的 UI 实现
- 使用 `rfd::AsyncFileDialog` 提供文件选择器（SoundFont `.sf2`/`.sfz`、MIDI `.mid`）
- 加载成功后更新 `params` 中的持久化路径，DAW 自动保存/恢复
- 显示当前已加载的文件名

### engine
音频处理核心模块聚合层。

> 注意：`engine.rs` 本身只包含 `pub mod` 和 `pub use`，不含实际逻辑。

**engine/synth.rs**
- 封装 `xsynth-core` 的 `ChannelGroup`
- 音色库加载（`load_soundfont`，支持 `.sf2` 和 `.sfz` 双格式）
- 音频读取（`read_samples` 包装，用于 `pipeline`）
- 兼容渲染（`render`，保留用于测试）
- 发送 XSynth 事件（`send_event`）
- 全通道静音控制：
  - `all_notes_off()` → `AllNotesOff`（release 衰减）
  - `all_notes_killed()` → `AllNotesKilled`（立即切断）
- 暴露 `NUM_CHANNELS` 常量

**engine/pipeline.rs**
- 预分配交错采样缓冲区，避免音频线程堆分配
- 从 `synth` 读取音频块
- 应用 `gain`（含平滑器）
- 写入 DAW `Buffer`

**engine/midi_player.rs**
- 与 DAW Transport 同步
- 管理内部 MIDI 事件队列
- 播放头跳转检测（scrub / 循环）
- 走带暂停时触发 `all_notes_off()`
- 恢复播放前触发 `all_notes_killed()` 清除残留
- 时间戳转换（DAW beats ↔ ticks）

> 注意：`midi_player.rs` 不直接依赖 `midly` 或 `std::fs`，只接收已解析的事件流。

**engine/midi_filter.rs**
- 无状态纯函数
- `velocity_threshold`：丢弃低于阈值的音符
- `force_max_velocity`：将力度统一设为 127
- 在 `midi_mapper` 之前执行，减少无效 voice 分配

**engine/midi_mapper.rs**
- 无状态纯函数
- 将项目自定义 `MidiEvent` 映射为 `xsynth-core` 的 `SynthEvent`
- 支持 NoteOn/NoteOff、ProgramChange、ControlChange（Raw CC 透传）、PitchBend
- 方便单元测试

### data
数据模型，与音频线程解耦。

**data/event.rs**
- `MidiEvent` / `MidiMessage` 定义
- 使用 `tick: u64` 作为速度无关的时间戳
- 被 `midi_player`、`midi_filter`、`midi_mapper` 共享引用

**data/midi_loader.rs**
- 非实时文件解析模块
- 使用 `midly` 读取 MIDI 文件
- 提取 `PPQN` 和音符事件（含 CC、PitchBend、ProgramChange）
- 忽略 MIDI 原生 `SetTempo`，完全交给 DAW BPM 控制
- 返回 `LoadedMidi { events, ppqn }`

### utils
通用工具。

---

## 线程模型

```
┌──────────────┐         ┌──────────────────────────────┐
│   GUI 线程    │◄───────►│        初始化/后台线程        │
│  (editor)    │ arc-swap │  data::midi_loader (midly)  │
└──────┬───────┘         └──────────────────────────────┘
       │
       │ 无锁队列
       ▼
┌─────────────────────────────────────────────────────────┐
│                        音频线程                          │
│  ◄── 绝对不能分配内存 / 绝对不能加锁                      │
│                                                         │
│  ├── engine::midi_player.rs   (走带同步 + 事件调度)      │
│  ├── engine::midi_filter.rs   (力度过滤)                 │
│  ├── engine::midi_mapper.rs   (协议转换)                 │
│  ├── engine::synth.rs         (合成渲染)                 │
│  └── engine::pipeline.rs      (gain + 缓冲区写入)        │
└─────────────────────────────────────────────────────────┘
```

---

## 模块依赖关系

```
lib.rs
├── engine
│   ├── synth ◄────── pipeline
│   ├── midi_player ◄── midi_filter
│   │                    └── midi_mapper
│   └── midi_mapper
├── data
│   ├── event ◄────── midi_loader
│   └── midi_loader (仅被 lib.rs initialize 调用)
├── editor (通过 create_egui_editor 注册到 lib.rs)
└── params (YueliangParams，含持久化路径字段)
```
