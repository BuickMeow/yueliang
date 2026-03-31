# XSynth Core API 速查（v0.3.4）

> 最后更新：2026-03-31  
> 来源：本地源代码 `~/.cargo/registry/src/xsynth-core-0.3.4/`

---

## 核心概念

XSynth 是一个专为高复音数（8000+ voices）和低延迟设计的 Rust SoundFont 合成器。

### 架构层次

```
┌─────────────────────────────────────────────┐
│           ChannelGroup                      │ ← 主入口，管理 16 个 MIDI 通道
│  ┌─────────┐ ┌─────────┐     ┌─────────┐   │
│  │Channel 0│ │Channel 1│ ... │Channel15│   │ ← VoiceChannel (ch 9 默认打击乐)
│  └────┬────┘ └────┬────┘     └────┬────┘   │
│       └────────────┴───────────────┘        │
│              SoundFont 采样层               │
└─────────────────────────────────────────────┘
```

### 关键实现细节

- **事件缓存**：`send_event` 内部有缓存机制，超过 1MB (`MAX_EVENT_CACHE_SIZE`) 自动 flush
- **多线程渲染**：使用 rayon 并行处理多个 channel
- **实时安全**：`send_event` 和 `read_samples` 都是实时安全的

### 关键类型

| 类型 | 用途 | 所在模块 |
|------|------|----------|
| `ChannelGroup` | 主合成器，管理多个通道 | `channel_group` |
| `ChannelGroupConfig` | 初始化配置 | `channel_group` |
| `SynthEvent` | MIDI 事件包装 | `channel_group` |
| `AudioStreamParams` | 音频流参数 | 根模块 |
| `SampleSoundfont` | SoundFont 加载 | `soundfont` |

---

## ChannelGroup API

### 创建实例

```rust
use xsynth_core::channel_group::{ChannelGroup, ChannelGroupConfig};
use xsynth_core::AudioStreamParams;

let config = ChannelGroupConfig {
    channel_init_options: /* ... */,
    format: SynthFormat::Midi,  // 或 SF2
    audio_params: AudioStreamParams {
        sample_rate: 48000,
        channels: 2,
    },
    parallelism: ParallelismOptions::default(),
};

let mut synth = ChannelGroup::new(config);
```

### 发送 MIDI 事件

```rust
use xsynth_core::channel_group::SynthEvent;
use xsynth_core::channel::ChannelEvent;

// Note On
synth.send_event(SynthEvent::Midi(
    channel,  // 0-15
    ChannelEvent::NoteOn {
        key: 60,       // C4
        vel: 100,      // 力度 0-127
    }
));

// Note Off
synth.send_event(SynthEvent::Midi(
    channel,
    ChannelEvent::NoteOff { key: 60 }
));
```

### 读取音频（实现 AudioPipe）

```rust
// 实现 AudioPipe trait，可直接 read_samples
let mut left_buffer = vec![0.0f32; num_samples];
let mut right_buffer = vec![0.0f32; num_samples];

// 方式1: 读取交错采样（LRLRLR...）
let mut interleaved = vec![0.0f32; num_samples * 2];
synth.read_samples(&mut interleaved);

// 方式2: 分离通道（如果需要）
for i in 0..num_samples {
    left_buffer[i] = interleaved[i * 2];
    right_buffer[i] = interleaved[i * 2 + 1];
}
```

### 获取状态

```rust
// 获取当前活跃 voice 数（用于监控）
let active_voices: u64 = synth.voice_count();

// 获取音频参数
let params: &AudioStreamParams = synth.stream_params();
```

---

## SynthEvent 事件类型（从源代码确认）

```rust
// channel_group/events.rs
pub enum SynthEvent {
    /// A channel event to be sent to the specified channel.
    Channel(u32, ChannelEvent),  // (channel, event)
    
    /// A channel event to be sent to all available channels.
    AllChannels(ChannelEvent),
}

// channel/event.rs
pub enum ChannelEvent {
    Audio(ChannelAudioEvent),
    Config(ChannelConfigEvent),
}

pub enum ChannelAudioEvent {
    /// 音符开
    NoteOn { key: u8, vel: u8 },
    
    /// 音符关
    NoteOff { key: u8 },
    
    /// 所有音符关
    AllNotesOff,
    
    /// 强制停止所有音符（无衰减）
    AllNotesKilled,
    
    /// 重置所有 CC 到默认值
    ResetControl,
    
    /// 控制器事件
    Control(ControlEvent),
    
    /// 程序变更
    ProgramChange(u8),
}

pub enum ChannelConfigEvent {
    /// 设置 SoundFont
    SetSoundfonts(Vec<Arc<dyn SoundfontBase>>),
    
    /// 设置 layer 数
    SetLayerCount(Option<usize>),
    
    /// 设置打击乐模式（通道 9 默认 true）
    SetPercussionMode(bool),
}

pub enum ControlEvent {
    /// 原始 CC (cc, value)
    Raw(u8, u8),
    
    /// 弯音灵敏度（音分）
    PitchBendSensitivity(f32),
    
    /// 弯音值 (-1 到 1)
    PitchBendValue(f32),
    
    /// 弯音 (sensitivity * value)
    PitchBend(f32),
    
    /// 微调 (cents)
    FineTune(f32),
    
    /// 粗调 (semitones)
    CoarseTune(f32),
}
```

---

## 配置详解

### ChannelGroupConfig

```rust
pub struct ChannelGroupConfig {
    /// 通道初始化选项
    pub channel_init_options: ChannelInitOptions,
    
    /// 合成器格式
    pub format: SynthFormat,
    
    /// 音频流参数（采样率、通道数）
    pub audio_params: AudioStreamParams,
    
    /// 多线程选项
    pub parallelism: ParallelismOptions,
}
```

### ChannelInitOptions

```rust
pub struct ChannelInitOptions {
    /// 每个键的最大 layers 数
    pub layers: Option<usize>,
    
    /// 是否启用多线程
    pub multithread: bool,
    
    /// 其他选项...
}
```

### SynthFormat

```rust
pub enum SynthFormat {
    /// MIDI 模式（16通道）
    Midi,
    
    /// SF2 模式
    Sf2,
}
```

---

## VST 集成模式

### 1. 初始化时创建引擎

```rust
fn initialize(
    &mut self,
    _audio_io_layout: &AudioIOLayout,
    buffer_config: &BufferConfig,
    _context: &mut impl InitContext<Self>,
) -> bool {
    let audio_params = AudioStreamParams {
        sample_rate: buffer_config.sample_rate as u32,
        channels: 2,
    };
    
    let config = ChannelGroupConfig {
        channel_init_options: ChannelInitOptions::default(),
        format: SynthFormat::Midi,
        audio_params,
        parallelism: ParallelismOptions::default(),
    };
    
    self.engine = Some(ChannelGroup::new(config));
    
    // 加载音色库
    if let Some(ref mut engine) = self.engine {
        // 加载 SF2...
    }
    
    true
}
```

### 2. Process 中读取音频

```rust
fn process(
    &mut self,
    buffer: &mut Buffer,
    _aux: &mut AuxiliaryBuffers,
    _context: &mut impl ProcessContext<Self>,
) -> ProcessStatus {
    if let Some(ref mut engine) = self.engine {
        let num_samples = buffer.samples();
        
        // XSynth 输出交错采样 [L, R, L, R, ...]
        let mut temp_buffer = vec![0.0f32; num_samples * 2];
        engine.read_samples(&mut temp_buffer);
        
        // 写入 DAW 缓冲区
        for (i, channel_samples) in buffer.iter_samples().enumerate() {
            for (ch, sample) in channel_samples.iter_mut().enumerate() {
                *sample = temp_buffer[i * 2 + ch];
            }
        }
    }
    
    ProcessStatus::Normal
}
```

### 3. 注意事项（从源代码确认）

⚠️ **音频线程安全**:
- `send_event` 是实时安全的（内部使用缓存）
- `read_samples` 是实时安全的
- 但加载 SoundFont **不是** 实时安全的，应在初始化时完成

⚠️ **事件缓存机制**:
```rust
// channel_group/mod.rs
const MAX_EVENT_CACHE_SIZE: u32 = 1024 * 1024; // 1MB

// send_event 内部会自动缓存，超过阈值自动 flush
```

⚠️ **缓冲区对齐**:
- XSynth 输出的是交错采样 `[L, R, L, R, ...]`
- nih-plug 的 Buffer 是平面布局 `[L...L, R...R]` 或逐采样迭代

⚠️ **特殊通道处理**:
- 通道 9（第 10 通道）默认设置为打击乐模式
- 这是 MIDI 标准，XSynth 在初始化时自动设置

---

## 相关链接

- GitHub: https://github.com/BlackMIDIDevs/xsynth
- API 文档: https://docs.rs/xsynth-core/0.3.4/
- ChannelGroup: https://docs.rs/xsynth-core/0.3.4/xsynth_core/channel_group/
- SynthEvent: https://docs.rs/xsynth-core/0.3.4/xsynth_core/channel_group/enum.SynthEvent.html
