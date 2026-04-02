use nih_plug::prelude::*;
use nih_plug_egui::EguiState;
use std::sync::Arc;

mod engine;
mod data;
mod utils;

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

#[derive(Params)]
pub struct YueliangParams {
    #[persist = "editor_state"]
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
                1.0,
                FloatRange::Linear { min: 0.0, max: 2.0 },
            )
            .with_smoother(SmoothingStyle::None)
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            max_voices: IntParam::new("Max Voices", 1000, IntRange::Linear { min: 1, max: 100000 }),
            velocity_threshold: IntParam::new("Velocity Threshold", 1, IntRange::Linear { min: 0, max: 127 }),
            force_max_velocity: BoolParam::new("Force Max Velocity", false),
            interpolation: EnumParam::new("Interpolation", InterpolationMode::Linear),
            enable_limiter: BoolParam::new("Enable Limiter", true),
        }
    }
}

pub struct Yueliang {
    params: Arc<YueliangParams>,
    engine: Option<engine::SynthEngine>,
    pipeline: engine::Pipeline,
    midi_player: engine::MidiPlayer,
}

impl Default for Yueliang {
    fn default() -> Self {
        Self {
            params: Arc::new(YueliangParams::default()),
            engine: None,
            pipeline: engine::Pipeline::new(),
            midi_player: engine::MidiPlayer::new(),
        }
    }
}

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

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        let max_voices = self.params.max_voices.value() as usize;
        let sample_rate = buffer_config.sample_rate;

        let mut engine = engine::SynthEngine::new(sample_rate, max_voices);

        let soundfont_path = "/Users/jieneng/Documents/GitHub/yueliang/assets/GeneralUser-GS.sf2";
        match engine.load_soundfont(soundfont_path) {
            Ok(()) => nih_log!("SoundFont loaded successfully: {}", soundfont_path),
            Err(e) => nih_log!("Warning: Failed to load SoundFont: {}", e),
        }

        let midi_path = "/Users/jieneng/Documents/GitHub/yueliang/assets/Act Beloved.mid";
        match crate::data::midi_loader::load_from_file(midi_path) {
            Ok(loaded) => {
                nih_log!("MIDI ready: {} events", loaded.events.len());
                self.midi_player.load(loaded.events, loaded.ppqn);
            }
            Err(e) => nih_log!("Warning: Failed to load MIDI: {}", e),
        }

        self.engine = Some(engine);
        true
    }

    fn reset(&mut self) {
        if let Some(ref mut engine) = self.engine {
            engine.reset();
        }
        self.midi_player.reset();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        if let Some(ref mut engine) = self.engine {
            let transport = context.transport();
            let num_frames = buffer.samples();
            self.midi_player.process(transport, engine, &self.params, num_frames);
            self.pipeline.render(buffer, engine, &self.params);
        }

        ProcessStatus::Normal
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        None
    }
}

impl Vst3Plugin for Yueliang {
    const VST3_CLASS_ID: [u8; 16] = *b"YueliangVSTi0000";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Instrument,
        Vst3SubCategory::Synth,
    ];
}

nih_export_vst3!(Yueliang);
