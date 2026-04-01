# 多端口 MIDI 映射方案

> 最后更新：2026-03-31  
> 需求：支持黑乐谱中 A/B/C/D 端口（共 64+ 通道）

---

## 问题背景

MIDI 标准只有 16 通道（0x0n-0xEn），但黑乐谱常用多端口扩展：
- 端口选择事件：`FF 21 01 pp`（pp = 端口号 0-255）
- 每个端口有 16 通道
- A 端口 = 0-15, B 端口 = 16-31, C 端口 = 32-47...

XSynth 的 `ChannelGroup` 原生只支持单一端口，需要在上层做映射。

---

## 方案对比

| 方案 | 复杂度 | 性能 | 灵活性 | 推荐度 |
|------|--------|------|--------|--------|
| 单一 ChannelGroup + 通道映射 | ⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ | ✅ 推荐 |
| 多个 ChannelGroup（每端口一个） | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ | 备用 |
| 自定义 XSynth Fork | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | 长期考虑 |

---

## 推荐方案：单一 ChannelGroup + 通道映射

### 核心思想

将 `(端口, 通道)` 二维坐标映射到 XSynth 的一维通道号：

```
逻辑通道 = 端口 × 16 + 通道
```

| 端口 | MIDI 通道 | XSynth 通道 |
|------|-----------|-------------|
| A (0) | 0-15 | 0-15 |
| B (1) | 0-15 | 16-31 |
| C (2) | 0-15 | 32-47 |
| D (3) | 0-15 | 48-63 |
| ... | ... | ... |

### 数据结构

```rust
/// 当前活跃端口状态
pub struct PortState {
    /// 当前选中的端口（0-255）
    current_port: u8,
    /// 每个 XSynth 通道对应的 (端口, 原始通道)
    channel_mapping: [(u8, u8); 64], // XSynth 通道 -> (端口, 通道)
}

impl PortState {
    /// 处理端口选择事件 FF 21 01 pp
    pub fn select_port(&mut self, port: u8) {
        self.current_port = port;
    }
    
    /// 将 MIDI 事件映射到 XSynth 通道
    pub fn map_event(&self, midi_channel: u8) -> u32 {
        let xsynth_channel = (self.current_port as u32) * 16 + (midi_channel as u32);
        xsynth_channel.min(63) // 限制在 64 通道内
    }
}
```

### 事件处理流程

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│   MIDI 文件      │────▶│  端口状态机       │────▶│  XSynth 通道    │
│  FF 21 01 pp    │     │  维护 current_port│     │  0-63          │
│  9n kk vv       │     │                  │     │                 │
└─────────────────┘     └──────────────────┘     └─────────────────┘
```

### 代码实现思路

```rust
// 在 data 模块中维护端口状态
pub struct MidiRouter {
    current_port: u8,
    // 可选：每个端口独立的音色设置
    port_programs: [u8; 256], // 端口 -> 最后使用的 program
}

impl MidiRouter {
    /// 处理 Meta Event
    pub fn process_meta_event(&mut self, event: &MetaMessage) {
        match event {
            MetaMessage::PortNumber(port) => {
                self.current_port = *port;
            }
            _ => {}
        }
    }
    
    /// 将 MIDI 通道事件转换为 XSynth 事件
    pub fn route_event(&self, channel: u8, message: MidiMessage) -> (u32, MidiMessage) {
        let xsynth_channel = (self.current_port as u32) * 16 + (channel as u32);
        (xsynth_channel.min(63), message)
    }
}
```

---

## 备用方案：多个 ChannelGroup

如果单一 ChannelGroup 64 通道不够，或需要端口隔离：

```rust
pub struct MultiPortEngine {
    /// 每端口一个 ChannelGroup
    groups: Vec<ChannelGroup>,
    /// 当前活跃端口
    current_port: usize,
}

impl MultiPortEngine {
    pub fn new(sample_rate: f32) -> Self {
        // 创建 4 个 ChannelGroup（A/B/C/D）
        let mut groups = Vec::new();
        for _ in 0..4 {
            groups.push(ChannelGroup::new(
                ChannelGroupConfig {
                    format: SynthFormat::Midi, // 每端口 16 通道
                    ..Default::default()
                }
            ));
        }
        
        Self {
            groups,
            current_port: 0,
        }
    }
    
    pub fn send_event(&mut self, port: usize, channel: u8, event: MidiMessage) {
        if let Some(group) = self.groups.get_mut(port) {
            // 发送到对应端口
        }
    }
    
    pub fn render(&mut self, left: &mut [f32], right: &mut [f32]) {
        // 混合所有端口的输出
    }
}
```

### 优缺点

| 优点 | 缺点 |
|------|------|
| 真正的端口隔离 | 内存占用 ×N |
| 每端口独立音色库 | 渲染时需要混合多路音频 |
| 无限扩展端口数 | 更复杂的资源管理 |

---

## 决策建议

### 短期（阶段 2-3）
使用**单一 ChannelGroup + 通道映射**：
- 64 通道足够大部分黑乐谱
- 实现简单，性能最好
- 代码维护成本低

### 长期（如果不够）
迁移到**多个 ChannelGroup**：
- 当需要 128+ 通道时
- 或需要严格的端口隔离时

---

## 相关链接

- MIDI 端口选择 Meta Event: `FF 21 01 pp`
- XSynth ChannelGroup: 支持 `SynthFormat::Custom { channels: N }`
