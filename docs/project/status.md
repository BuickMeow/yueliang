# Yueliang 项目状态

最后更新：2026-04-03-16-59-50
当前版本：v0.0.1

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

### 阶段 4 关键决策

**Tick-based 事件存储**：用 `tick` 代替 `sample_offset`，使 MIDI 播放速度完全由 DAW BPM 控制，同时天然支持播放头跳转。详见 `docs/decisions/tick-based-event-storage.md`

**解析与调度分离**：`data::midi_loader` 负责非实时文件解析，`engine::midi_player` 负责实时调度，确保音频线程零分配。详见 `docs/decisions/midi-loader-separation.md`

### 阶段 5 进行中 🟡

- [x] Egui UI 基础框架（`editor.rs`）
- [x] 文件选择器（MIDI `.mid`、SoundFont `.sf2`/`.sfz`）
- [x] `rfd::AsyncFileDialog` 集成（修复 macOS 同步对话框崩溃）
- [x] 路径持久化（DAW 保存/打开工程时自动恢复用户选择的文件）
- [x] 去掉默认硬编码音色库与 MIDI 引用
- [ ] 路由矩阵（MIDI 通道 → VST 输出总线）
- [ ] 参数可视化（当前 voice 数、过滤统计）

### 阶段 5 关键决策与踩坑

**macOS 文件选择器崩溃**：在 egui UI 线程直接使用 `rfd::FileDialog::pick_file()`（同步版）会与 `baseview` 事件循环冲突，导致 `SIGABRT`。改为 `rfd::AsyncFileDialog` + 子线程 `simple_block_on` 解决。详见 `docs/knowledge/nih-plug-egui-integration.md`

**路径持久化**：使用 `#[persist = "..."]` + `Arc<parking_lot::Mutex<String>>` 存储用户选择的路径，DAW 自动保存/恢复，采样率变更时 `initialize()` 也能重新加载。

---

## 开发阶段规划

| 阶段 | 目标 | 状态 |
|------|------|------|
| 阶段 1 | 基础脚手架与参数打通 | ✅ 完成 |
| 阶段 2 | XSynth 引擎集成与音频通路验证 | ✅ 完成 |
| 阶段 3 | 启用 XSynth 引擎（音色+MIDI）+ 代码拆分 | ✅ 完成 |
| 阶段 4 | 内部走带同步与预过滤 | ✅ 完成 |
| 阶段 5 | Egui UI 与动态路由 | 🟡 核心完成，路由矩阵待做 |

---

## P0 优先任务

1. 多总线音频输出与路由矩阵
2. 性能基准测试（Black MIDI Stress Test）
3. 预分配环形缓冲区替代 `Vec::resize`

---

## P1 任务

1. UI 参数可视化（voice 计数、MIDI 过滤统计）
2. 修复 MIDI 跳转后状态注入（CC / PB / RPN 顺序与 scrub 阈值）
3. 支持更多 MIDI Meta Event（如 Marker、Lyric 的显示/忽略策略）
4. 发布第一个公开测试版
