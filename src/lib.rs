//use midly::stream::DefaultBuffer;
use nih_plug::prelude::*;
use nih_plug_egui::EguiState;
use std::sync::{Arc, mpsc::channel};

//mod editor;
mod engine;
mod data;
mod utils;

// 1 枚举定义
#[derive(Enum, Clone, Copy, PartialEq, Eq, Debug)]
pub enum InterpolationMode {
    Nearest,
    Linear,
}

impl Default for InterpolationMode {
    fn default() -> Self {
        Self::Linear
    }
}

// 2 参数结构(暴露给DAW的核心参数)
#[derive(Params)]
pub struct YueliangParams {
    #[persist = "editor_state"]// 这里editorstate应该是_还是-？
    pub editor_state: Arc<EguiState>,
    #[id = "gain"]
    pub gain: FloatParam,

    #[id = "max_voices"]
    pub max_voices: IntParam,

    #[id = "vel_thresh"]
    pub velocity_threshold: IntParam,

    #[id = "force_max_vel"]
    pub force_max_velocity: BoolParam,

    #[id = "interp"]
    pub interpolation: EnumParam<InterpolationMode>,

    #[id = "limiter"]
    pub enable_limiter: BoolParam,
}

impl Default for YueliangParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(800, 600),

            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(0.0),
                FloatRange::Linear {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(30.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            max_voices: IntParam::new(
                "Max Voices",
                1000,
                IntRange::Linear { min: 1, max: 100000 },
            ),

            velocity_threshold: IntParam::new(
                "Velocity Threshold",
                0,
                IntRange::Linear { min: 0, max: 127 }
            ),

            force_max_velocity: BoolParam::new("Force Max Velocity", false),

            interpolation: EnumParam::new("Interpolation", InterpolationMode::Linear),

            enable_limiter: BoolParam::new("Enable Limiter", true),
        }
    }
}

// 3 主插件结构
pub struct Yueliang {
    params: Arc<YueliangParams>,
    engine: Option<engine::SynthEngine>,
}

impl Default for Yueliang {
    fn default() -> Self {
        Self {
            params: Arc::new(YueliangParams::default()),
            engine: None,
            //接下来engine和mididata也要new？尚未确定
        }
    }
}

// 4 Plugin Trait 实现
impl Plugin for Yueliang {
    const NAME: &'static str = "Yueliang";
    const VENDOR: &'static str = "Jieneng";
    const URL: &'static str = "https://space.bilibili.com/433246974";
    const EMAIL: &'static str = "3347830431@qq.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: None,
        main_output_channels: NonZeroU32::new(2),
        aux_input_ports: &[],
        aux_output_ports: &[],
        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;    // 不走DAW的MIDI输入
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }
    /*fn accepts_bus_config(&self, config: &BusConfig) -> bool {
        config.num_input_channels == 0 && config.num_output_channels == 2
    }*/

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,       // 音频IO配置
        buffer_config: &BufferConfig,           // 采样率/缓冲区大小
        _context: &mut impl InitContext<Self>,  // DAW初始化上下文
    ) -> bool {
        // 获取参数值
        let max_voices = self.params.max_voices.value() as usize;
        let sample_rate = buffer_config.sample_rate;

        // 创建引擎
        self.engine = Some(engine::SynthEngine::new(sample_rate, max_voices));

        // TODO: 加载默认音色库

        true
        // 因为完成度不高（或者说哪都还没开始），所以现阶段不写TODO
        // 接下来，要创建xsynth合成器，加载音色库，应用参数设置等
    }

    fn reset(&mut self) {
        if let Some(ref mut engine) = self.engine {
            engine.reset();
        }
    }

    fn process(
            &mut self,
            buffer: &mut Buffer,
            _aux: &mut AuxiliaryBuffers,
            _context: &mut impl ProcessContext<Self>,
        ) -> ProcessStatus {
            // 获取gain参数
            let gain = self.params.gain.smoothed.next();

            if let Some(ref mut engine) = self.engine {
                let num_frames = buffer.samples(); 
                // 准备左右声道缓冲区
                let mut left = vec![0.0f32; num_frames];
                let mut right = vec![0.0f32; num_frames];

                // 渲染音频
                engine.render(&mut left, &mut right, num_frames);

                // 写入DAW缓冲区并应用gain
                for (i, mut channel_samples) in buffer.iter_samples().enumerate() {
                    let l = left[i] * gain;
                    let r = right[i] * gain;

                    // buffer是逐采样迭代
                    let mut iter = channel_samples.iter_mut();
                    *iter.next().unwrap() = l;
                    *iter.next().unwrap() = r;
                }
            }
            //let _transport = context.transport();

            // AI给了以下步骤建议，目前尚未确定如何做
            // 1. 获取当前DAW时间 (_transport.pos_samples())
            // 2. 从内部MIDI队列取出当前时间的事件
            // 3. 力度过滤 (velocity_threshold)
            // 4. 强制最大力度 (force_max_velocity)
            // 5. 发给 xsynth 引擎渲染
            // 6. 应用 gain 和 limiter
            // 7. 写入 buffer

            ProcessStatus::Normal
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        //我猜得先create？

        None
    }
}

impl Vst3Plugin for Yueliang {
    const VST3_CLASS_ID: [u8; 16] = *b"YueliangVSTi0000";   // 这个ID必须是16位
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Instrument,
        Vst3SubCategory::Synth,
    ];
}

nih_export_vst3!(Yueliang);
