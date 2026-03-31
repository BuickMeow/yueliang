# nih-plug Plugin Trait 详解

本文档介绍如何编写 nih-plug 的 `Plugin` trait，以及各开源项目的实现模式。

---

## 目录

1. [Plugin Trait 基础结构](#1-plugin-trait-基础结构)
2. [必需实现的关联类型和常量](#2-必需实现的关联类型和常量)
3. [生命周期方法](#3-生命周期方法)
4. [参数系统](#4-参数系统)
5. [音频处理](#5-音频处理)
6. [GUI 集成](#6-gui-集成)
7. [多格式导出](#7-多格式导出)
8. [开源项目实现模式对比](#8-开源项目实现模式对比)

---

## 1. Plugin Trait 基础结构

```rust
use nih_plug::prelude::*;
use std::sync::Arc;

pub struct MyPlugin {
    params: Arc<MyParams>,
    // 其他状态...
}

impl Plugin for MyPlugin {
    // 实现 trait 方法...
}
```

### 核心组件关系

```
┌─────────────────┐
│   MyPlugin      │ ← 实现 Plugin trait
│  ┌───────────┐  │
│  │ MyParams  │  │ ← 参数结构体 (derive Params)
│  │  ┌─────┐  │  │
│  │  │param│  │  │ ← FloatParam/IntParam/EnumParam...
│  │  └─────┘  │  │
│  └───────────┘  │
└─────────────────┘
         │
         ▼
┌─────────────────┐
│   VST3/CLAP     │ ← nih_export_vst3!/nih_export_clap!
└─────────────────┘
```

---

## 2. 必需实现的关联类型和常量

### 2.1 插件元数据

```rust
impl Plugin for MyPlugin {
    const NAME: &'static str = "My Plugin";           // 插件名称
    const VENDOR: &'static str = "My Company";        // 开发商
    const URL: &'static str = "https://example.com";  // 网站
    const EMAIL: &'static str = "contact@example.com"; // 联系邮箱
    const VERSION: &'static str = env!("CARGO_PKG_VERSION"); // 版本号
```

### 2.2 音频 IO 配置

```rust
    // 定义支持的音频布局
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),   // 主输入通道数
            main_output_channels: NonZeroU32::new(2),  // 主输出通道数
            aux_input_ports: &[],                       // 辅助输入端口
            aux_output_ports: &[],                      // 辅助输出端口
            names: PortNames::const_default(),          // 端口名称
        },
        // 可以定义多个布局，DAW 会选择合适的
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];
```

**常见配置模式：**

| 插件类型 | 输入 | 输出 |
|---------|------|------|
| 效果器 (Effect) | 2 通道 | 2 通道 |
| 乐器 (Instrument) | None | 2 通道 |
| MIDI 效果器 | 2 通道 | 2 通道 + MIDI |

### 2.3 MIDI 配置

```rust
    // MIDI 输入配置
    const MIDI_INPUT: MidiConfig = MidiConfig::None;    // 不接受 MIDI
    // 或
    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;   // 基础 MIDI
    // 或  
    const MIDI_INPUT: MidiConfig = MidiConfig::BasicMidiFx; // MIDI 效果器模式

    // MIDI 输出配置
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;
```

### 2.4 关联类型

```rust
    // SysEx 消息类型（通常不需要）
    type SysExMessage = ();
    
    // 后台任务类型（通常不需要）
    type BackgroundTask = ();
```

### 2.5 参数访问

```rust
    // 返回参数结构的引用
    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }
```

---

## 3. 生命周期方法

### 3.1 initialize - 初始化

在插件被激活时调用，用于设置采样率、初始化资源等。

```rust
fn initialize(
    &mut self,
    _audio_io_layout: &AudioIOLayout,       // 音频 IO 配置
    buffer_config: &BufferConfig,           // 采样率和缓冲区大小
    _context: &mut impl InitContext<Self>,  // DAW 初始化上下文
) -> bool {
    // 存储采样率供后续使用
    self.sample_rate = buffer_config.sample_rate;
    
    // 初始化音频引擎、滤波器等...
    // self.engine.initialize(self.sample_rate);
    
    true // 返回 true 表示初始化成功
}
```

**实际项目示例：**

| 项目 | 初始化内容 |
|------|-----------|
| rust-audio-plugin | 空实现（`true`） |
| Actuate | 存储采样率，检查走带状态 |
| Yueliang | 计划：初始化 XSynth 引擎 |

### 3.2 reset - 重置

当 DAW 需要重置插件状态时调用（如播放停止后重新开始）。

```rust
fn reset(&mut self) {
    // 清除滤波器状态、包络发生器等...
    // self.filter.reset();
}
```

### 3.3 deactivate - 停用（可选）

当插件被停用时调用，用于清理特殊资源。

```rust
fn deactivate(&mut self) {
    // 清理资源（大多数插件不需要）
}
```

---

## 4. 参数系统

### 4.1 参数结构定义

```rust
#[derive(Params)]
struct MyParams {
    // 持久化编辑器状态（如果使用 GUI）
    #[persist = "editor_state"]
    pub editor_state: Arc<EguiState>,
    
    // 浮点参数
    #[id = "gain"]
    pub gain: FloatParam,
    
    // 整数参数
    #[id = "voices"]
    pub voice_count: IntParam,
    
    // 布尔参数
    #[id = "enable"]
    pub enabled: BoolParam,
    
    // 枚举参数
    #[id = "filter_type"]
    pub filter_type: EnumParam<FilterType>,
}
```

### 4.2 参数配置详解

#### FloatParam - 浮点参数

```rust
FloatParam::new(
    "Gain",                                    // 显示名称
    util::db_to_gain(0.0),                    // 默认值
    FloatRange::Skewed {                      // 值范围
        min: util::db_to_gain(-30.0),
        max: util::db_to_gain(30.0),
        factor: FloatRange::gain_skew_factor(-30.0, 30.0),
    },
)
.with_smoother(SmoothingStyle::Logarithmic(50.0))  // 平滑处理
.with_unit(" dB")                                   // 单位
.with_value_to_string(formatters::v2s_f32_gain_to_db(2))  // 值→字符串
.with_string_to_value(formatters::s2v_f32_gain_to_db())   // 字符串→值
```

**FloatRange 类型：**

| 类型 | 用途 | 示例 |
|------|------|------|
| `Linear` | 线性范围 | 音量 0.0-1.0 |
| `Skewed` | 对数/指数范围 | 频率 20Hz-20kHz |
| `Reversed` | 反向范围 | 滤波器 Q 值 |

#### IntParam - 整数参数

```rust
IntParam::new(
    "Max Voices",
    64,                                       // 默认值
    IntRange::Linear { min: 1, max: 512 },   // 线性范围
)
```

#### EnumParam - 枚举参数

```rust
// 定义枚举
derive(Enum, Debug, Clone, Copy, PartialEq)]
pub enum FilterType {
    #[name = "Low Pass"]
    LowPass,
    #[name = "High Pass"]
    HighPass,
    #[name = "Band Pass"]
    BandPass,
}

// 使用
EnumParam::new("Filter Type", FilterType::LowPass)
```

### 4.3 嵌套参数组

```rust
#[derive(Params)]
struct OscillatorParams {
    #[id = "waveform"]
    pub waveform: IntParam,
    #[id = "detune"]
    pub detune: FloatParam,
}

#[derive(Params)]
struct MyParams {
    #[nested(group = "Oscillator 1")]
    pub osc1: OscillatorParams,
    
    #[nested(group = "Oscillator 2")]
    pub osc2: OscillatorParams,
    
    // 参数数组
    #[nested(array, group = "Oscillator Array")]
    pub osc_array: [OscillatorParams; 3],
}
```

### 4.4 持久化非参数数据

```rust
#[derive(Params)]
struct MyParams {
    // 持久化自定义数据（需要实现 Serialize/Deserialize）
    #[persist = "wavetable"]
    pub wavetable: Mutex<WavetableData>,
    
    #[persist = "user_config"]
    pub config: Mutex<UserConfig>,
}
```

---

## 5. 音频处理

### 5.1 process 方法

这是音频处理的核心方法。

```rust
fn process(
    &mut self,
    buffer: &mut Buffer,                      // 音频缓冲区
    _aux: &mut AuxiliaryBuffers,              // 辅助缓冲区
    context: &mut impl ProcessContext<Self>,  // 处理上下文
) -> ProcessStatus {
    // 获取走带信息
    let transport = context.transport();
    let playing = transport.playing;
    let sample_rate = transport.sample_rate;
    
    // 遍历采样
    for channel_samples in buffer.iter_samples() {
        // 获取参数值（自动平滑）
        let gain = self.params.gain.smoothed.next();
        
        // 处理每个通道
        for sample in channel_samples {
            *sample *= gain;
        }
    }
    
    ProcessStatus::Normal
}
```

### 5.2 处理状态

```rust
pub enum ProcessStatus {
    Normal,           // 正常处理
    Bypass,           // 旁路（让音频直通）
    KeepAlive,        // 保持激活（不处理但保持状态）
}
```

### 5.3 处理模式

| 模式 | 调用方式 | 适用场景 |
|------|----------|----------|
| 块处理 (Block) | 按缓冲区调用 | 简单效果器 |
| 逐采样 (Sample) | buffer.iter_samples() | 需要精确时序 |

---

## 6. GUI 集成

### 6.1 editor 方法

```rust
fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
    // 创建编辑器实例
    create_editor(
        self.params.clone(),
        self.editor_state.clone(),
    )
}
```

### 6.2 支持的 GUI 框架

| 框架 | 说明 | 示例项目 |
|------|------|----------|
| egui | 简单、纯 Rust | Actuate |
| VIZIA | 声明式 UI | simple-panner |
| Slint | 跨平台 | nih-plug-slint |
| Webview | Web 技术 | nih-plug-webview |

### 6.3 egui 集成示例

```rust
use nih_plug_egui::{create_egui_editor, EguiState};

fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
    create_egui_editor(
        self.params.editor_state.clone(),
        (), // 初始状态
        |ctx, _state, params| {
            // 绘制 UI
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label("Gain");
                ui.add(
                    egui::Slider::from_get_set(
                        -30.0..=30.0,
                        |v| params.gain.get() as f64,
                        |v| params.gain.set(v as f32),
                    )
                );
            });
        },
    )
}
```

---

## 7. 多格式导出

### 7.1 VST3 导出

```rust
impl Vst3Plugin for MyPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"MyPluginExampleX";  // 16 字节 ID
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Fx,        // 效果器
        Vst3SubCategory::Tools,     // 工具类
        // 或
        // Vst3SubCategory::Instrument, // 乐器
        // Vst3SubCategory::Synth,      // 合成器
    ];
}

nih_export_vst3!(MyPlugin);
```

### 7.2 CLAP 导出

```rust
impl ClapPlugin for MyPlugin {
    const CLAP_ID: &'static str = "com.example.my-plugin";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A simple gain plugin");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
    ];
}

nih_export_clap!(MyPlugin);
```

---

## 8. 开源项目实现模式对比

### 8.1 项目对比表

| 项目 | 类型 | 复杂度 | 特色功能 | GUI |
|------|------|--------|----------|-----|
| rust-audio-plugin | 示例 | ⭐ | 极简结构 | 无 |
| Actuate | 合成器 | ⭐⭐⭐⭐⭐ | 3 振荡器、采样器、FX | egui |
| simple-panner | 效果器 | ⭐⭐ | 声像控制 | VIZIA |
| nih-plug-template | 模板 | ⭐⭐ | 基础结构 | 可选 |

### 8.2 简单效果器模式（rust-audio-plugin）

```rust
// 结构简单
struct MyPlugin {
    params: Arc<PluginParams>,
}

// process 直接处理
fn process(&mut self, buffer: &mut Buffer, ...) -> ProcessStatus {
    for channel_samples in buffer.iter_samples() {
        let gain = db_to_gain(self.params.gain.smoothed.next());
        for sample in channel_samples {
            *sample *= gain;
        }
    }
    ProcessStatus::Normal
}
```

### 8.3 复杂乐器模式（Actuate）

```rust
pub struct Actuate {
    pub params: Arc<ActuateParams>,
    pub sample_rate: f32,
    
    // 多模块
    audio_module_1: Arc<Mutex<AudioModule>>,
    audio_module_2: Arc<Mutex<AudioModule>>,
    audio_module_3: Arc<Mutex<AudioModule>>,
    
    // 多个 LFO
    lfo_1: LFOController,
    lfo_2: LFOController,
    lfo_3: LFOController,
    
    // 大量 FX
    compressor: Compressor,
    delay: Delay,
    reverb: [StereoReverb; 8],
    // ... 更多
}

// process 中处理 MIDI
fn process(&mut self, buffer: &mut Buffer, ...) -> ProcessStatus {
    // 更新 LFO
    // 处理 MIDI 事件
    self.process_midi(context, buffer);
    ProcessStatus::Normal
}
```

### 8.4 推荐模式选择

| 你的需求 | 推荐模式 | 参考项目 |
|---------|----------|----------|
| 简单效果器 | 基础模式 | rust-audio-plugin |
| 合成器 | 复杂乐器模式 | Actuate |
| MIDI 工具 | MIDI 处理模式 | midiometry |
| 带 GUI | GUI 集成模式 | Actuate / simple-panner |

---

## 9. 完整最小示例

```rust
use nih_plug::prelude::*;
use std::sync::Arc;

struct GainPlugin {
    params: Arc<GainParams>,
}

#[derive(Params)]
struct GainParams {
    #[id = "gain"]
    pub gain: FloatParam,
}

impl Default for GainPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(GainParams::default()),
        }
    }
}

impl Default for GainParams {
    fn default() -> Self {
        Self {
            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(30.0),
                    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
        }
    }
}

impl Plugin for GainPlugin {
    const NAME: &'static str = "Gain";
    const VENDOR: &'static str = "Example";
    const URL: &'static str = "";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = "0.1.0";

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for channel_samples in buffer.iter_samples() {
            let gain = self.params.gain.smoothed.next();
            for sample in channel_samples {
                *sample *= gain;
            }
        }
        ProcessStatus::Normal
    }
}

impl Vst3Plugin for GainPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"GainPluginExmpX";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = 
        &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}

nih_export_vst3!(GainPlugin);
```

---

## 参考资源

- [nih-plug 官方文档](https://github.com/robbert-vdh/nih-plug)
- [nih-plug-template](https://github.com/robbert-vdh/nih-plug-template)
- [Actuate 合成器](https://github.com/ardura/Actuate)
- [rust-audio-plugin](https://github.com/steckes/rust-audio-plugin)
