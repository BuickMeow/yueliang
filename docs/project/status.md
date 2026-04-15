# Yueliang 项目状态

最后更新：2026-04-15-13-40-02
当前版本：v0.0.2

---

## 项目定位

专为「黑乐谱 (Black MIDI)」设计的极限性能 VSTi 序列器/合成器插件。使用 Rust + nih-plug + xsynth-core 实现。

核心目标：让上亿音符在 DAW 中无卡顿回放与离线渲染。

---

## 当前阶段

**阶段 5：Egui UI 与文件持久化** 🟡 进行中 / 核心功能已完成

### 阶段 1 已完成 ✅

- [x] 项目初始化（Cargo.toml、rust-toolchain.toml）
- [x] 基础插件结构（Yueliang、YueliangParams）
- [x] 参数定义（Gain、Max Polyphony、Velocity Threshold 等）
- [x] Plugin trait 基础实现
- [x] VST3 导出配置
- [x] 编译通过
- [x] 模块文件创建（engine.rs、data.rs、utils.rs）
- [x] DAW 中加载测试通过（Cakewalk Next）
- [x] 参数自动化验证通过（Gain 曲线控制）

### 阶段 1 关键决策

**参数设计原理**：暴露给 DAW 的是「宏观混音参数」（Gain、Polyphony、Filter），而非 CC。详见 `docs/knowledge/parameter-design.md`

### 阶段 2 已完成 ✅

- [x] 研究 xsynth-core API（从源代码确认）
- [x] 实现 `engine::SynthEngine` 结构
- [x] 实现 `TestToneGenerator` 正弦波生成器
- [x] 集成到 lib.rs（initialize/process/reset）
- [x] 修复编译错误（mut channel_samples）
- [x] 修复相位连续性（立体声渲染）
- [x] 完善 Gain 参数设计（-inf ~ +6dB）
- [x] 创建 deploy.sh 快速部署脚本
- [x] DAW 中正弦波测试通过

### 阶段 2 关键决策

**Gain 参数范围**：使用 Skewed Range `-96dB ~ +6dB`（实际显示为 -inf ~ +6dB），符合行业标准调音台设计。

**音频通路验证**：正弦波测试证明插件音频输出链路完全正常。

### 阶段 3 已完成 ✅

- [x] 准备 SoundFont 音色库（GeneralUser-GS.sf2）
- [x] 实现音色加载（`engine.load_soundfont()`）
- [x] 修复 macOS 代码签名问题
- [x] 配置 XSynth 通道（64 通道）
- [x] 发送测试 MIDI 事件（NoteOn/NoteOff）
- [x] 验证 XSynth 音频输出（Sample sum > 0）
- [x] DAW 中成功听到声音
- [x] 代码结构拆分（`lib.rs` / `engine/` / `data/` 解耦）
- [x] 移除 `TestToneGenerator` 等历史遗留代码

### 阶段 3 关键决策与踩坑

**SoundFont 路径问题**：VST3 插件运行时使用绝对路径加载 SF2/SFZ 文件，相对路径因工作目录不同而失效。

**MIDI 事件时机**：NoteOn 在 `initialize()` 只触发一次，实际使用时需要根据 DAW 走带位置持续发送。

**调试技巧**：使用文件日志（`/tmp/yueliang_debug.log`）比控制台日志更可靠。详见 `docs/notes/debugging-techniques.md`

### 阶段 4 已完成 ✅

- [x] 搭建 `MidiPlayer` / `Pipeline` / `MidiMapper` / `MidiFilter` 子模块框架
- [x] 实现力度过滤（`midi_filter::apply_filter`）逻辑
- [x] MIDI 文件加载（`midly`）并解析为 `Vec<MidiEvent>`
- [x] 基于 DAW 走带位置的 MIDI 事件调度（tick → DAW beat 转换）
- [x] DAW BPM 驱动 MIDI 播放（忽略原 MIDI Tempo）
- [x] Scrub / 播放头跳转检测与 `event_index` 快速重置
- [x] 预过滤与批量发送优化生效
- [x] CC / PitchBend / ProgramChange 完整映射并修复 PitchBend 公式 bug
- [x] 走带暂停时 `AllNotesOff`，恢复播放前 `AllNotesKilled` 切断残留
- [x] **MIDI Chase 功能实现**：播放跳转时自动向前搜索 CC/PC/PB 状态，防止错音
- [x] **StateTable 方案弃用**：改为实时线性搜索，零预存储内存

### 阶段 4 关键决策

**Tick-based 事件存储**：用 `tick` 代替 `sample_offset`，使 MIDI 播放速度完全由 DAW BPM 控制，同时天然支持播放头跳转。详见 `docs/decisions/tick-based-event-storage.md`

**解析与调度分离**：`data::midi_loader` 负责非实时文件解析，`engine::midi_player` 负责实时调度，确保音频线程零分配。详见 `docs/decisions/midi-loader-separation.md`

**MIDI Chase 实时搜索方案**：弃用预计算 StateTable，改为播放跳转时实时向前搜索。内存占用 O(1)，支持任意规模 MIDI 文件。详见 `docs/decisions/midi-chase-implementation.md`

### 阶段 5 已完成 ✅

- [x] Egui UI 基础框架（`editor.rs`）
- [x] 左侧栏导航（Transport / Soundfonts / Channels）
- [x] 文件选择器（MIDI `.mid`、SoundFont `.sf2`/`.sfz`）
- [x] `rfd::AsyncFileDialog` 集成（修复 macOS 同步对话框崩溃）
- [x] 路径持久化（DAW 保存/打开工程时自动恢复用户选择的文件）
- [x] 去掉默认硬编码音色库与 MIDI 引用
- [x] **16 端口音色库管理器**（Port A-P）
  - [x] 每个端口独立的音色库列表
  - [x] 多音色库叠加（从下到上覆盖）
  - [x] 拖拽排序（多选拖拽、插入线提示、幽灵预览）
  - [x] 启用/禁用开关
  - [x] 多选编辑（Ctrl/Cmd + 点击、Shift + 范围选择）
  - [x] 复制当前端口配置到所有端口（📑 按钮）
  - [x] JSON 配置导入/导出
- [x] MIDI 走带面板（文件加载完成，播放控制预留）

### 阶段 5 关键决策与踩坑

**音色库缓存**：`SynthEngine` 使用 `HashMap<String, Arc<dyn SoundfontBase>>` 缓存已加载的 SF2，避免 16 端口复制时重复加载导致内存爆炸。详见 `docs/decisions/soundfont-cache.md`

**egui 拖拽排序实现**：使用 `Sense::click_and_drag()` + `allocate_new_ui()` + `ui.interact()` 实现整行拖拽，插入索引按"删除后可见项"计算。详见 `docs/knowledge/egui-drag-sort.md` 与 `docs/notes/drag-sort-insert-index.md`

### 阶段 6 部分完成 🟡

  - [x] **通道矩阵（Channel Matrix）**
  - [x] 16×16 按钮网格（Port A-P × Channel 1-16 = 256 通道）
  - [x] 表头整行/整列一键开关
  - [x] 右键 Solo / 取消 Solo（支持单格、整行、整列）
  - [x] 音频线程静音过滤（`process()` 内零锁数组索引）
  - [x] 静音时自动发送 `AllNotesOff` + `Sustain Pedal Off (CC64=0)`
  - [x] 恢复发声时自动 Chase 该通道最新 CC/PC/PB 状态
  - [ ] ~~鼠标拖动连选~~（已尝试多种实现，因 egui 交互捕获机制复杂暂时搁置）
  - [-] **Drum 模式切换**（设计完成，部分实现中）
    - [-] 左上角菱形按钮切换 Mute/Drum 表
    - [-] Drum 模式下 256 按钮控制 XSynth `SetPercussionMode`
    - [-] MIDI 加载时自动推断鼓通道（默认 port×16+9，扫描 Bank Select MSB 覆盖）
    - [-] `drum_matrix` 持久化参数（`Vec<bool>`）
- [ ] 路由矩阵（MIDI 通道 → VST 输出总线）
- [ ] 参数可视化（当前 voice 数、过滤统计）

### 阶段 6 关键决策与踩坑

**实时过滤位置**：在 `midi_filter.rs` 中基于局部 `[bool; 256]` 数组进行过滤，每个 buffer 只 `lock()` 一次 `Vec<bool>`，避免音频线程反复竞争 Mutex。

**Chase 时序问题**：通道恢复的 Chase 事件必须放在 `midi_player.process()` 内部、所有 `system_reset()` 之后发送，否则会被 reset 覆盖而"失效"。

**egui Grid 默认列宽陷阱**：`egui::Grid` 的 `min_col_width` 默认值为 `interact_size.x`（40px），会导致 24px 按钮被强制拉宽，视觉间距严重不均。必须显式设置 `.min_col_width(0.0)`。

**serde 数组长度限制**：`[bool; 256]` 不支持 `Serialize/Deserialize`（serde 仅原生支持到 32 长度），通道矩阵持久化改用 `Vec<bool>`。

**Drum 状态跟踪位置**：`last_drums` 跟随 `last_mutes` 一起放在 `midi_player.rs` 中，由 `MidiPlayer::process()` 统一检测变化并发送 XSynth `SetPercussionMode` 事件。详见 `docs/decisions/drum-channel-placement.md`。

---

## 下一阶段目标

1. **完善路由矩阵**：支持 MIDI 通道到多总线输出映射
2. **性能监控 UI**：实时显示 voice 数、CPU 占用
3. **压力测试**：验证上亿音符场景下的稳定性

---

## 已知问题

- 无
