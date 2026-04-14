# 模块结构

> 最后更新：2026-04-14-03-49-32
> 当前阶段：v0.0.2

## 文件组织

```
src/
├── lib.rs              # 插件主入口
├── editor.rs           # UI 模块入口
├── editor/
│   ├── left_bar.rs     # 左侧导航栏（Transport/Soundfonts/Channels）
│   ├── transport.rs    # MIDI 走带面板（文件加载 + 播放控制预留）
│   ├── sf_manager.rs   # 音色库管理器（16端口 + 多音色 + 拖拽排序）
│   └── sf_list.rs      # 音色库列表（多选、右键菜单、开关）
├── engine.rs           # 模块聚合入口（pub mod / pub use）
├── engine/
│   ├── synth.rs        # xsynth 封装（渲染、音色加载）
│   ├── pipeline.rs     # 音频处理管线（gain、缓冲区写入）
│   ├── midi_player.rs  # MIDI 时间线调度器（走带同步、事件分发、Chase）
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

**数据结构**
- `SoundfontEntry`：单个音色库配置（路径、名称、乐器类型、启用状态）
- `PortSoundfonts`：单个端口的音色库列表（`Vec<SoundfontEntry>`）
- `YueliangParams.port_soundfonts`：16 个端口的配置数组

### editor
- 基于 `nih_plug_egui` 的 UI 实现
- 使用 `rfd::FileDialog` 提供文件选择器（支持多选）
- 左侧栏导航使用 `egui::SidePanel`，自动分隔线

**editor/left_bar.rs**
- 左侧 48px 导航栏（Transport / Soundfonts / Channels）
- 图标按钮悬停变色，选中显示左侧竖条指示器

**editor/transport.rs**
- MIDI 文件加载按钮
- 走带控制按钮预留（播放/暂停/停止等）

**editor/sf_manager.rs**
- 16 端口选择器（Port A-P）
- 工具栏：添加、全选、移除、复制到所有端口（📑）、导入/导出菜单
- 管理每个端口的音色库列表
- 处理拖拽释放后的条目插入与引擎重载
- 实时通知引擎重新加载音色

**editor/sf_list.rs**
- 音色库条目显示（名称 + 路径）
- 启用/禁用开关（checkbox）
- 多选支持（Ctrl/Cmd+点击、Shift+范围选择）
- **拖拽排序**：整行 `Sense::click_and_drag()` 触发，支持多选批量拖拽
  - 拖拽中：原位置项变淡，鼠标旁显示半透明幽灵列表
  - 插入线提示：根据鼠标 Y 坐标计算可见项插入位置
- 禁止文字选中（`Label::selectable(false)`），避免干扰点击/拖拽
- 右键菜单（上移/下移/移除）

### engine
音频处理核心模块聚合层。

> 注意：`engine.rs` 本身只包含 `pub mod` 和 `pub use`，不含实际逻辑。

**engine/synth.rs**
- 封装 `xsynth-core` 的 `ChannelGroup`
- 音色库加载（`load_soundfont`，支持 `.sf2` 和 `.sfz` 双格式）
- **多音色库加载**（`load_soundfonts_to_port`）：
  - 支持每个端口加载多个音色库
  - 音色库按顺序叠加（后加载的覆盖先加载的）
  - 自动应用到该端口的 16 个通道
- 音频读取（`read_samples` 包装，用于 `pipeline`）
- 兼容渲染（`render`，保留用于测试）
- 发送 XSynth 事件（`send_event`）
- 全通道静音控制：
  - `all_notes_off()` → `AllNotesOff`（release 衰减）
  - `all_notes_killed()` → `AllNotesKilled`（立即切断）
- 暴露 `NUM_CHANNELS` 常量（256 通道 = 16 端口 × 16 通道）

**engine/pipeline.rs**
- 预分配交错采样缓冲区，避免音频线程堆分配
- 从 `synth` 读取音频块
- 应用 `gain`（含平滑器）
- 写入 DAW `Buffer`

**engine/midi_player.rs**
- 与 DAW Transport 同步
- 管理内部 MIDI 事件队列
- 播放头跳转检测（scrub / 循环 / 暂停恢复）
- **MIDI Chase**：跳转时向前搜索 CC/PC/PB 最新状态并注入
  - 实时线性搜索，零预存储内存
  - 支持百万级事件规模
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

---

## 数据流

```
文件加载阶段（非实时）：
  MIDI文件 → midi_loader.rs → Vec<MidiEvent> → midi_player.load()
  SF2/SFZ文件 → synth.rs → xsynth ChannelGroup

播放阶段（实时，每buffer）：
  DAW Transport → midi_player.process()
    ↓
  [Chase检测] → 跳转时向前搜索CC/PC/PB → synth.send_event()
    ↓
  [事件分发] → midi_filter → midi_mapper → synth.send_event()
    ↓
  synth.read_samples() → pipeline (gain) → DAW Buffer
