use xsynth_core::{
    AudioPipe, AudioStreamParams, channel::{
        ChannelAudioEvent,
        ChannelEvent,
    }, channel_group::{
        ChannelGroup, ChannelGroupConfig, ParallelismOptions, SynthEvent, SynthFormat
    }
};
pub struct SynthEngine {
    core: ChannelGroup,
    sample_rate: f32,
    max_voices: usize,  // 用于监控性能
}

#[derive(Clone, Debug)]
pub struct MidiEvent {
    pub sample_offset: usize,    // 这里不是指time，而是当前 buffer 里的 sample 偏移位置
    pub channel: u8,    // 这里不做多端口，而是在上层做端口映射
    pub message: MidiMessage,
}

#[derive(Clone, Debug)]
pub enum MidiMessage {  // 分别为8n 9n Bn Cn En，n为通道号（0-F，9为鼓）
    NoteOn { key: u8, velocity: u8 },
    NoteOff { key: u8 },
    ControlChange { cc: u8, value: u8 },
    ProgramChange { pc: u8 },    // 这里与LSB MSB分开发送
    PitchBend { value: i16 },
    // XSynth不支持系统消息与触后，能支持的都写上面了
}

impl SynthEngine {
    /// 创建新的合成器引擎
    /// 
    /// # Arguments
    /// * `sample_rate` - 采样率（如 44100.0, 48000.0）
    /// * `max_voices` - 最大复音数（用于监控，不限制 XSynth）
    pub fn new(sample_rate: f32, _max_voices: usize) -> Self {
        let audio_params = AudioStreamParams {
            sample_rate: sample_rate as u32,
            channels: xsynth_core::ChannelCount::Stereo,    // 这里指双声道
        };

        let config = ChannelGroupConfig {
            channel_init_options: Default::default(),
            format: SynthFormat::Custom { channels: 64 },
            audio_params,
            parallelism: ParallelismOptions::default(),
        };

        let core = ChannelGroup::new(config);   //这是啥

        Self{
            core,
            sample_rate,
            max_voices: _max_voices,
        }
    }

    pub fn load_soundfont(&mut self, _path: &str) -> Result<(), String> {
        // 我想写外置的音色库加载器，不想用内置的，初期方案详见/docs/external/copilot-soundfont-loading.md，这是Copilot网页版根据XSynth库生成的
        Ok(())
    }

    /// 发送 MIDI 事件到引擎
    /// 
    /// 这个方法是实时安全的，可以在 audio thread 中调用
    pub fn send_midi(&mut self, event: MidiEvent) {
        let channel_event = match event.message {
            MidiMessage::NoteOn { key, velocity } => {
                ChannelEvent::Audio(ChannelAudioEvent::NoteOn { 
                    key: key, 
                    vel: velocity 
                })
            }
            MidiMessage::NoteOff { key } => {
                ChannelEvent::Audio(ChannelAudioEvent::NoteOff { 
                    key: key,
                })
            }
            MidiMessage::ControlChange { cc, value } => {
                // 映射到XSynth的ControlEvent，还没写
                return;
            }
            MidiMessage::ProgramChange { pc } => {
                ChannelEvent::Audio(ChannelAudioEvent::ProgramChange(pc))
            }
            MidiMessage::PitchBend { value } => {
                let normalized = value as f32 / 8192.0;
                return;
            }
        };
        self.core.send_event(SynthEvent::Channel(
            event.channel as u32,
            channel_event,
        ));
    }

    /// 渲染音频到输出缓冲区
    /// 
    /// # Arguments
    /// * `left` - 左声道输出缓冲区
    /// * `right` - 右声道输出缓冲区
    /// * `num_frames` - 需要渲染的采样帧数
    /// 
    /// 这个方法是实时安全的
    pub fn render(&mut self, left: &mut [f32], right: &mut [f32], num_frames: usize) {
        // XSynth 输出交错采样 [L, R, L, R, ...]
        let mut interleaved = vec![0.0f32; num_frames * 2];

        // 读取音频（实现AudioPipe Trait）
        self.core.read_samples(&mut interleaved);

        // 分离到左右声道
        for i in 0..num_frames {
            left[i] = interleaved[i * 2];
            right[i] = interleaved[i * 2 + 1];
        }
    }

    /// 重置引擎状态（播放停止时调用）
    /// 
    /// 发送 AllNotesOff 到所有通道
    pub fn reset (&mut self) {
        for ch in 0..16 {
            self.core.send_event(SynthEvent::Channel(
                ch,
                ChannelEvent::Audio(ChannelAudioEvent::AllNotesOff),
            ));
        }
    }

    // 监控：获取当前活跃voice数
    pub fn active_voices(&self) -> u64 {
        self.core.voice_count()
    }

    // 获取采样率
    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }
}

/// 简单的测试：生成正弦波（不需要 SoundFont）
/// 
/// 用于验证引擎是否能正常输出音频
/// 警告：因为这与黑乐谱没有关系，所以我可能随时移除！
pub struct TestToneGenerator {
    phase: f32,
    frequency: f32,
    sample_rate: f32,
}

impl TestToneGenerator {
    pub fn new(frequency: f32, sample_rate: f32) -> Self {
        Self {
            phase: 0.0,
            frequency,
            sample_rate,
        }
    }

    pub fn render(&mut self, buffer: &mut [f32]) {
        let phase_increment = 2.0 * std::f32::consts::PI * self.frequency / self.sample_rate;
        
        for sample in buffer.iter_mut() {
            *sample = self.phase.sin() * 0.1; // 0.1 振幅避免爆音
            self.phase += phase_increment;
            if self.phase > 2.0 * std::f32::consts::PI {
                self.phase -= 2.0 * std::f32::consts::PI;
            }
        }
    }

    pub fn render_stereo(&mut self, left: &mut [f32], right: &mut [f32]) {
    let phase_increment = 2.0 * std::f32::consts::PI * self.frequency / self.sample_rate;
    
    for i in 0..left.len() {
        let sample = self.phase.sin() * 0.1;
        left[i] = sample;
        right[i] = sample;  // 同样的信号到右声道
        
        self.phase += phase_increment;
        if self.phase > 2.0 * std::f32::consts::PI {
            self.phase -= 2.0 * std::f32::consts::PI;
        }
    }
}
}