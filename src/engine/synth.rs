pub const NUM_CHANNELS: u32 = 64;

use xsynth_core::{
    AudioPipe, AudioStreamParams, channel::{
        ChannelAudioEvent, ChannelEvent, ChannelConfigEvent
    }, channel_group::{
        ChannelGroup, ChannelGroupConfig, ParallelismOptions, SynthEvent, SynthFormat
    }, soundfont::{
        SampleSoundfont, SoundfontInitOptions
    }
};

use std::sync::Arc;
use nih_plug::prelude::nih_log;

pub struct SynthEngine {
    core: ChannelGroup,
    sample_rate: f32,
    max_voices: usize,
    soundfont_loaded: bool,
}

impl SynthEngine {
    pub fn new(sample_rate: f32, max_voices: usize) -> Self {
        let audio_params = AudioStreamParams {
            sample_rate: sample_rate as u32,
            channels: xsynth_core::ChannelCount::Stereo,
        };

        let config = ChannelGroupConfig {
            channel_init_options: Default::default(),
            format: SynthFormat::Custom { channels: NUM_CHANNELS },
            audio_params,
            parallelism: ParallelismOptions::default(),
        };

        let core = ChannelGroup::new(config);

        Self {
            core,
            sample_rate,
            max_voices,
            soundfont_loaded: false,
        }
    }

    pub fn load_soundfont(&mut self, path: &str) -> Result<(), String> {
        let soundfont = match SampleSoundfont::new(
            path,
            self.core.stream_params().clone(),
            SoundfontInitOptions::default(),
        ) {
            Ok(sf) => Arc::new(sf) as Arc<dyn xsynth_core::soundfont::SoundfontBase>,
            Err(e) => return Err(format!("Failed to load SoundFont: {:?}", e)),
        };

        self.core.send_event(SynthEvent::AllChannels(
            ChannelEvent::Config(ChannelConfigEvent::SetSoundfonts(vec![soundfont]))
        ));

        self.soundfont_loaded = true;
        Ok(())
    }

    pub fn is_soundfont_loaded(&self) -> bool {
        self.soundfont_loaded
    }

    /// 直接发送 XSynth 事件（实时安全）
    pub fn send_event(&mut self, event: SynthEvent) {
        self.core.send_event(event);
    }

    /// 渲染音频到左右声道（行为与原来完全一致）
    pub fn render(&mut self, left: &mut [f32], right: &mut [f32], num_frames: usize) {
        let mut interleaved = vec![0.0f32; num_frames * 2];
        self.core.read_samples(&mut interleaved);

        for i in 0..num_frames {
            left[i] = interleaved[i * 2];
            right[i] = interleaved[i * 2 + 1];
        }
    }

    pub fn reset(&mut self) {
        for ch in 0..NUM_CHANNELS {
            self.core.send_event(SynthEvent::Channel(
                ch,
                ChannelEvent::Audio(ChannelAudioEvent::AllNotesOff),
            ));
        }
    }

    pub fn all_notes_off(&mut self) {
        for ch in 0..NUM_CHANNELS {
            self.core.send_event(SynthEvent::Channel(
                ch as u32,
                ChannelEvent::Audio(ChannelAudioEvent::AllNotesOff),
            ));
        }
    }

    pub fn all_notes_killed(&mut self) {
        for ch in 0..NUM_CHANNELS {
            self.core.send_event(SynthEvent::Channel(
                ch as u32,
                ChannelEvent::Audio(ChannelAudioEvent::AllNotesKilled),
            ));
        }
    }

    pub fn active_voices(&self) -> u64 {
        self.core.voice_count()
    }

    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    /*pub fn send_test_note(&mut self) {
        self.core.send_event(SynthEvent::Channel(
            0,
            ChannelEvent::Audio(ChannelAudioEvent::ProgramChange(0)),
        ));
        self.core.send_event(SynthEvent::Channel(
            0,
            ChannelEvent::Audio(ChannelAudioEvent::NoteOn { key: 60, vel: 100 }),
        ));
        nih_log!("Test note sent: C4 (key=60, vel=100)");
    }*/

    /*pub fn stop_test_note(&mut self) {
        self.core.send_event(SynthEvent::Channel(
            0,
            ChannelEvent::Audio(ChannelAudioEvent::NoteOff { key: 60 }),
        ));
    }*/

    pub fn read_samples(&mut self, buffer: &mut [f32]) {
        self.core.read_samples(buffer);
    }
}