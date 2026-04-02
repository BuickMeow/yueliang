# Yueliang 文档索引

> 最后更新：2026-04-02-10-57-52

---

## 快速导航

| 文档 | 用途 | 阶段 |
|------|------|------|
| [project/status.md](project/status.md) | 项目当前状态与 TODO | 必读 |
| [architecture/overview.md](architecture/overview.md) | 架构设计理念与数据流 | 设计参考 |
| [architecture/modules.md](architecture/modules.md) | 模块划分与职责 | 开发参考 |
| [knowledge/vst3-deployment.md](knowledge/vst3-deployment.md) | VST3 部署指南 | 部署 |

---

## 按主题分类

### 🎯 项目状态

- **[project/status.md](project/status.md)** - 当前阶段、TODO、里程碑
- **[notes/redlines.md](notes/redlines.md)** - 实时音频开发红线（零分配、禁止操作）
- **[notes/debugging-techniques.md](notes/debugging-techniques.md)** - 调试技巧与踩坑记录（文件日志、代码签名等）

### 🏗️ 架构设计

- **[architecture/overview.md](architecture/overview.md)** - DAW 隔离、内部合成、预过滤架构
- **[architecture/modules.md](architecture/modules.md)** - lib/engine/data/utils 模块划分
- **[architecture/port-mapping.md](architecture/port-mapping.md)** - 多端口 MIDI 映射方案（A/B/C/D）

### 📚 技术知识

#### nih-plug
- **[knowledge/nih-plug-plugin-trait.md](knowledge/nih-plug-plugin-trait.md)** - Plugin trait 完整指南
- **[knowledge/parameter-design.md](knowledge/parameter-design.md)** - 参数设计原理（宏观混音参数）
- **[knowledge/vst3-deployment.md](knowledge/vst3-deployment.md)** - macOS VST3 部署步骤

#### XSynth
- **[knowledge/xsynth-core-api.md](knowledge/xsynth-core-api.md)** - XSynth Core API 速查（基于源代码）

#### 音频基础
- **[knowledge/oscillator-phase-continuity.md](knowledge/oscillator-phase-continuity.md)** - 振荡器相位连续性原理
- **[knowledge/gain-parameter-design.md](knowledge/gain-parameter-design.md)** - Gain 参数设计详解（dB vs Gain）

### 📝 决策记录

- **[decisions/naming-conventions.md](decisions/naming-conventions.md)** - max_voices / max_polyphony 命名规范

### 📦 外部资源

- **[external/copilot-soundfont-loading.md](external/copilot-soundfont-loading.md)** - SoundFont 加载参考（待整理）

---

## 开发阶段对应文档

### 阶段 1：基础脚手架 ✅
- [knowledge/parameter-design.md](knowledge/parameter-design.md)
- [knowledge/vst3-deployment.md](knowledge/vst3-deployment.md)

### 阶段 2：XSynth 引擎集成 ✅
- [knowledge/xsynth-core-api.md](knowledge/xsynth-core-api.md)
- [architecture/port-mapping.md](architecture/port-mapping.md)
- [decisions/naming-conventions.md](decisions/naming-conventions.md)
- [knowledge/oscillator-phase-continuity.md](knowledge/oscillator-phase-continuity.md)
- [knowledge/gain-parameter-design.md](knowledge/gain-parameter-design.md)

### 阶段 3：启用 XSynth 引擎 ✅
- [external/copilot-soundfont-loading.md](external/copilot-soundfont-loading.md)
- [notes/debugging-techniques.md](notes/debugging-techniques.md) - 排查无声音问题的完整流程
- [architecture/modules.md](architecture/modules.md) - 代码拆分后的模块职责

### 阶段 4：内部走带同步 🟡
- [architecture/modules.md](architecture/modules.md) - `midi_player` / `pipeline` / `midi_mapper` 设计

---

## 常用命令

```bash
# 快速部署（编译 + 复制到 VST3 目录）
./deploy.sh

# 仅编译
cargo build --release

# 清理
cargo clean
```

---

## 项目信息

- **名称**：Yueliang（月亮）
- **版本**：v0.0.1
- **技术栈**：Rust + nih-plug + xsynth-core
- **目标**：黑乐谱 (Black MIDI) 极限性能 VSTi
