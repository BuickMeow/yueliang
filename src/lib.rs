use nih_plug::prelude::*;
use nih_plug_egui::EguiState;
use std::sync::Arc;
use parking_lot::Mutex;

use crate::engine::NUM_CHANNELS;

mod engine;
mod data;
mod utils;
mod editor;

#[derive(Enum, Clone, Copy, PartialEq, Eq, Debug)]
pub enum InterpolationMode {
    Nearest,
    Linear,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub enum ChannelMatrixMode {
    #[default]
    Mute,
    Drum,
}

impl Default for InterpolationMode {
    fn default() -> Self {
        Self::Linear
    }
}

#[derive(Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct SoundfontEntry {
    pub path: String,
    pub name: String,           // 显示名称（如 "ABCD.sf2"）
    //pub instrument_type: String, // "钢琴"/"鼓"/"多乐器"
    pub enabled: bool,          // 开关状态
}

#[derive(Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct PortSoundfonts {
    pub entries: Vec<SoundfontEntry>, // 替换原来的 paths: Vec<String>
}

#[derive(Params)]
pub struct YueliangParams {
    #[persist = "editor_state"]
    pub editor_state: Arc<EguiState>,

    #[persist = "soundfont_path"]
    pub soundfont_path: Arc<parking_lot::Mutex<String>>,

    #[persist = "midi_path"]
    pub midi_path: Arc<parking_lot::Mutex<String>>,

    #[persist = "port_soundfonts"]
    pub port_soundfonts: Arc<Mutex<[PortSoundfonts; (NUM_CHANNELS / 16) as usize]>>,

    #[persist = "channel_matrix"]
    pub channel_matrix: Arc<Mutex<Vec<bool>>>,

    #[persist = "drum_matrix"]
    pub drum_matrix: Arc<Mutex<Vec<bool>>>,

    #[persist = "channel_matrix_mode"]
    pub channel_matrix_mode: Arc<Mutex<ChannelMatrixMode>>,

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
            soundfont_path: Arc::new(parking_lot::Mutex::new(String::new())),
            midi_path: Arc::new(parking_lot::Mutex::new(String::new())),
            port_soundfonts: Arc::new(Mutex::new(
                std::array::from_fn(|_| PortSoundfonts::default())
            )),
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
            channel_matrix: Arc::new(Mutex::new(vec![true; 256])),
            channel_matrix_mode: Arc::new(Mutex::new(ChannelMatrixMode::Mute)),
            drum_matrix: Arc::new(Mutex::new(vec![false; 256])),
        }
    }
}

pub struct Yueliang {
    params: Arc<YueliangParams>,
    engine: Arc<Mutex<Option<engine::SynthEngine>>>,
    pipeline: engine::Pipeline,
    midi_player: Arc<Mutex<engine::MidiPlayer>>, 
    //last_mutes: [bool; 256], 
}

impl Default for Yueliang {
    fn default() -> Self {
        Self {
            params: Arc::new(YueliangParams::default()),
            engine: Arc::new(Mutex::new(None)),
            pipeline: engine::Pipeline::new(),
            midi_player: Arc::new(Mutex::new(engine::MidiPlayer::new())), 
            //last_mutes: [true; 256],
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

        // ---- SoundFont ----
        let port_configs = self.params.port_soundfonts.lock().clone();
        for (port_idx, port_config) in port_configs.iter().enumerate() {
            if !port_config.entries.is_empty() {
                let paths: Vec<String> = port_config.entries.iter().filter(|e| e.enabled).map(|e| e.path.clone()).collect();
                if let Err(e) = engine.load_soundfonts_to_port(port_idx, &paths) {
                    nih_log!("端口 {} 音色加载失败: {}", port_idx, e);
                } else {
                    nih_log!("端口 {} 加载 {} 个音色", port_idx, paths.len());
                }
            }
        }

        // ---- MIDI ----
        let midi_saved = self.params.midi_path.lock().clone();
        if !midi_saved.is_empty() {
            if let Ok(loaded) = crate::data::midi_loader::load_from_file(&midi_saved) {
                nih_log!("MIDI ready: {} events", loaded.events.len());
                {
                    let mut drum_vec = self.params.drum_matrix.lock();
                    for (i, &v) in loaded.drum_channels.iter().enumerate() {
                        drum_vec[i] = v;
                    }
                }
                self.midi_player.lock().load(loaded.events, loaded.ppqn);
            }
        }

        *self.engine.lock() = Some(engine);

        // 同步 drum matrix 到 XSynth
        if let Some(ref mut engine) = self.engine.lock().as_mut() {
            let drum_vec = self.params.drum_matrix.lock();
            for (i, &v) in drum_vec.iter().enumerate() {
                engine.set_percussion_mode(i as u32, v);
            }
        }

        // Pipeline预分配
        let max_frames = buffer_config.max_buffer_size as usize;
        self.pipeline = engine::Pipeline::with_capacity(max_frames);

        /*// 同步 channel matrix 初始状态
        let vec = self.params.channel_matrix.lock();
        for (i, &v) in vec.iter().enumerate() {
            self.last_mutes[i] = v;
        }*/
        true
    }

    fn reset(&mut self) {
        if let Some(ref mut engine) = self.engine.lock().as_mut() {
            engine.all_notes_killed();
        }
        self.midi_player.lock().reset();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let mut engine_guard = self.engine.lock();
        let mut midi_player = self.midi_player.lock();

        if let Some(ref mut engine) = engine_guard.as_mut() {
            let transport = context.transport();
            let num_frames = buffer.samples();

            let mutes: [bool; 256] = {
                let vec = self.params.channel_matrix.lock();
                let mut arr = [true; 256];
                for (i, &v) in vec.iter().enumerate() {
                    arr[i] = v;
                }
                arr
            };

            let drums: [bool; 256] = {
                let vec = self.params.drum_matrix.lock();
                let mut arr = [false; 256];
                for (i, &v) in vec.iter().enumerate() {
                    arr[i] = v;
                }
                arr
            };

            midi_player.process(transport, engine, &self.params, num_frames, &mutes, &drums);
            self.pipeline.render(buffer, engine, &self.params);

        }

        ProcessStatus::Normal
    }


    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(
            self.params.clone(),
            self.engine.clone(),
            self.midi_player.clone(),
        )
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
