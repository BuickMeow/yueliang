# 模块结构

> 最后更新：2026-04-02-10-56-11  
> 当前阶段：v0.0.1

## 文件组织

```
src/
├── lib.rs              # 插件主入口
├── editor.rs           # UI 界面（未来）
├── engine.rs           # 模块聚合入口（pub mod / pub use）
├── engine/
│   ├── synth.rs        # xsynth 封装（渲染、音色加载）
│   ├── midi_player.rs  # MIDI 时间线调度器（走带同步、预过滤）
│   ├── midi_mapper.rs  # MIDI 协议映射层（纯函数）
│   └── pipeline.rs     # 音频处理管线（gain、缓冲区写入）
├── data.rs             # 数据模块聚合入口
├── data/
│   └── event.rs        # MIDI 领域模型（MidiEvent / MidiMessage）
└── utils.rs            # 通用工具
```

---

## 模块职责

### lib.rs
- 插件主入口
- 实现 `Plugin` trait
- 定义 `Yueliang` 与 `YueliangParams`
- `process()` 仅负责"接线"：获取 Transport → 调用 `midi_player.process()` → 调用 `pipeline.render()`

### editor
- 基于 `nih_plug_egui` 的 UI 实现（阶段 5 启用）
- 功能：
  - 文件选择器（MIDI、SF2）
  - 路由矩阵（MIDI 通道 → VST 输出）
  - 参数显示

### engine
音频处理核心模块聚合层。

> 注意：`engine.rs` 本身只包含 `pub mod` 和 `pub use`，不含实际逻辑。

**engine/synth.rs**
- 封装 `xsynth-core` 的 `ChannelGroup`
- 音色库加载（`load_soundfont`）
- 音频读取（`read_samples` 包装，用于 `pipeline`）
- 兼容渲染（`render`，保留用于测试）
- 发送 XSynth 事件（`send_event`）
- 测试音符（`send_test_note`）

**engine/midi_player.rs**
- 与 DAW Transport 同步
- 管理内部 MIDI 事件队列
- 力度过滤（`velocity_threshold`）
- 强制最大力度（`force_max_velocity`）
- 时间戳转换（samples ↔ ticks，阶段 4 完善）

**engine/midi_mapper.rs**
- 无状态纯函数
- 将项目自定义 `MidiEvent` 映射为 `xsynth-core` 的 `SynthEvent`
- 方便单元测试

**engine/pipeline.rs**
- 预分配交错采样缓冲区，避免音频线程堆分配
- 从 `synth` 读取音频块
- 应用 `gain`（含平滑器）
- 写入 DAW `Buffer`

### data
数据模型，与音频线程解耦。

**data/event.rs**
- `MidiEvent` / `MidiMessage` 定义
- 被 `midi_player` 和 `midi_mapper` 共享引用

### utils
通用工具。

---

## 线程模型

```
┌──────────────┐         ┌──────────────┐
│   GUI 线程    │◄───────►│  后台线程    │
│  (editor)    │ arc-swap │  (loader)   │
└──────┬───────┘         └──────────────┘
       │
       │ 无锁队列
       ▼
┌──────────────┐
│  音频线程    │ ◄── 绝对不能分配内存
│  (process)   │ ◄── 绝对不能加锁
│  ├── midi_player.rs  │
│  ├── pipeline.rs     │
│  └── synth.rs        │
└──────────────┘
```

---

## 模块依赖关系

```
lib.rs
 ├── editor.rs
 │
 ├── engine.rs
 │   ├── synth.rs ──────► xsynth-core
 │   ├── midi_mapper.rs ─┐
 │   ├── midi_player.rs ─┼──► data::event
 │   └── pipeline.rs     │
 │
 ├── data.rs ◄───────────┘
 │   └── event.rs
 │
 └── utils.rs
```

---

## 实时音频红线

在 `pipeline.rs`、`synth.rs`、`midi_player.rs` 的 `process` 路径上：
- **禁止** `Vec::push()` / `String` 等堆分配
- **禁止** `Mutex` / `RwLock`
- `pipeline.rs` 使用 `Vec::resize()` 操作预分配缓冲区（容量足够时不触发新分配）
