# Yueliang 项目状态

最后更新：2026-04-02-10-56-42  
当前版本：v0.0.1

---

## 项目定位

专为「黑乐谱 (Black MIDI)」设计的极限性能 VSTi 序列器/合成器插件。使用 Rust + nih-plug + xsynth-core 实现。

核心目标：让上亿音符在 DAW 中无卡顿回放与离线渲染。

---

## 当前阶段

**阶段 4：内部走带同步与预过滤** 🟡 进行中

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

**SoundFont 路径问题**：VST3 插件运行时使用绝对路径加载 SF2 文件，相对路径因工作目录不同而失效。

**MIDI 事件时机**：NoteOn 在 `initialize()` 只触发一次，实际使用时需要根据 DAW 走带位置持续发送。

**调试技巧**：使用文件日志（`/tmp/yueliang_debug.log`）比控制台日志更可靠。详见 `docs/notes/debugging-techniques.md`

### 阶段 4 进行中

- [x] 搭建 `MidiPlayer` / `Pipeline` / `MidiMapper` 子模块框架
- [x] 实现力度过滤（`apply_filter`）逻辑框架
- [ ] MIDI 文件加载（midly）
- [ ] 基于走带位置的 MIDI 事件调度（`transport.pos_samples()`）
- [ ] 预过滤与批量发送优化生效

---

## 开发阶段规划

| 阶段 | 目标 | 状态 |
|------|------|------|
| 阶段 1 | 基础脚手架与参数打通 | ✅ 完成 |
| 阶段 2 | XSynth 引擎集成与音频通路验证 | ✅ 完成 |
| 阶段 3 | 启用 XSynth 引擎（音色+MIDI）+ 代码拆分 | ✅ 完成 |
| 阶段 4 | 内部走带同步与预过滤 | 🟡 进行中 |
| 阶段 5 | Egui UI 与动态路由 | ⬜ 未开始 |

---

## P0 优先任务（阶段 4）

1. MIDI 文件加载与解析（`midly`）
2. 基于走带位置的 MIDI 事件调度（tick → sample_offset 转换）
3. 让 `velocity_threshold` 和 `force_max_velocity` 在 `midi_player.process()` 中真正生效

## P1 任务

1. `pipeline.rs` 缓冲区零分配优化（固定大小 ring buffer）
2. 多端口 MIDI 映射方案实现（参考 `docs/architecture/port-mapping.md`）
