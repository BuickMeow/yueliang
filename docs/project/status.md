# Yueliang 项目状态

最后更新：2026-03-30  
当前版本：v0.0.1

---

## 项目定位

专为「黑乐谱 (Black MIDI)」设计的极限性能 VSTi 序列器/合成器插件。使用 Rust + nih-plug + xsynth-core 实现。

核心目标：让上亿音符在 DAW 中无卡顿回放与离线渲染。

---

## 当前阶段

**阶段 2：XSynth 引擎集成** 🟡 进行中

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

**参数设计原理**：暴露给 DAW 的是「宏观混音参数」（Gain、Polyphony、Filter），而非 CC。因为：

1. **DAW 隔离**：音符/CC 等微观数据在内部处理，DAW 只控制全局
2. **性能优化**：内部序列器承载上亿音符，DAW 只走带
3. **职责分离**：DAW = 录音室调音台，Yueliang = 乐手+乐器

详见 `docs/knowledge/parameter-design.md`

### 阶段 2 进行中

- [ ] 集成 xsynth-core
- [ ] 实现 `engine::SynthEngine` 结构
- [ ] 实现基础音频输出（正弦波测试）

---

## 开发阶段规划

| 阶段 | 目标 | 状态 |
|------|------|------|
| 阶段 1 | 基础脚手架与参数打通 | ✅ 完成 |
| 阶段 2 | XSynth 引擎集成 | 🟡 进行中 |
| 阶段 3 | 内部走带同步与预过滤 | ⬜ 未开始 |
| 阶段 4 | Egui UI 与动态路由 | ⬜ 未开始 |

---

## P0 优先任务（阶段 2）

1. 研究 xsynth-core API，确定集成方式
2. 实现 `engine::SynthEngine` 结构
3. 在 `process()` 中调用引擎生成音频

## P1 任务

- 加载 SoundFont 音色库
- 实现基础 MIDI 事件处理

## P2 任务

- 实现 MIDI 文件加载
- 实现力度过滤逻辑
