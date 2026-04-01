# Yueliang 项目状态

最后更新：2026-04-01  
当前版本：v0.0.1

---

## 项目定位

专为「黑乐谱 (Black MIDI)」设计的极限性能 VSTi 序列器/合成器插件。使用 Rust + nih-plug + xsynth-core 实现。

核心目标：让上亿音符在 DAW 中无卡顿回放与离线渲染。

---

## 当前阶段

**阶段 3：启用 XSynth 引擎** 🟡 进行中

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

### 阶段 3 进行中

- [ ] 准备 SoundFont 音色库
- [ ] 实现音色加载（engine.load_soundfont）
- [ ] 配置 XSynth 通道（16/64 通道）
- [ ] 发送测试 MIDI 事件（NoteOn/NoteOff）
- [ ] 验证 XSynth 音频输出

---

## 开发阶段规划

| 阶段 | 目标 | 状态 |
|------|------|------|
| 阶段 1 | 基础脚手架与参数打通 | ✅ 完成 |
| 阶段 2 | XSynth 引擎集成与音频通路验证 | ✅ 完成 |
| 阶段 3 | 启用 XSynth 引擎（音色+MIDI）| 🟡 进行中 |
| 阶段 4 | 内部走带同步与预过滤 | ⬜ 未开始 |
| 阶段 5 | Egui UI 与动态路由 | ⬜ 未开始 |

---

## P0 优先任务（阶段 3）

1. 准备 SoundFont 音色库（SF2/SFZ 格式）
2. 实现 `engine.load_soundfont()` 方法
3. 发送测试 MIDI 事件验证 XSynth 发声

## P1 任务

- 配置 XSynth 多通道（16/64）
- 实现端口映射（A/B/C/D）
- 实现基础力度过滤

## P2 任务

- 实现 MIDI 文件加载（midly）
- 内部序列器架构设计
