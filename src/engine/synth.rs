pub const NUM_CHANNELS: u32 = 256;
const _: () = assert!(NUM_CHANNELS <= 256);
// 256通道以上还是有点麻烦的，不仅Domino不支持，DAW也不一定能很好支持，改代码也稍微有点费劲，就这样吧

use xsynth_core::{
    AudioPipe, AudioStreamParams, channel::{
        ChannelAudioEvent, ChannelEvent, ChannelConfigEvent, ChannelInitOptions, ControlEvent
    }, channel_group::{
        ChannelGroup, ChannelGroupConfig, ParallelismOptions, SynthEvent, SynthFormat
    }, soundfont::{
        SampleSoundfont, SoundfontInitOptions, SoundfontBase
    }
};

use std::sync::Arc;
use std::collections::HashMap;
//use nih_plug::prelude::nih_log;

pub struct SynthEngine {
    core: ChannelGroup,
    sample_rate: f32,
    //max_voices: usize,
    soundfont_loaded: bool,
    sf_cache: HashMap<String, Arc<dyn SoundfontBase>>, // 新增
}

impl SynthEngine {
    pub fn new(sample_rate: f32, max_voices: usize) -> Self {
        let audio_params = AudioStreamParams {
            sample_rate: sample_rate as u32,
            channels: xsynth_core::ChannelCount::Stereo,
        };

        let config = ChannelGroupConfig {
            channel_init_options: ChannelInitOptions { fade_out_killing: true },
            format: SynthFormat::Custom { channels: NUM_CHANNELS },
            audio_params,
            parallelism: ParallelismOptions::AUTO_PER_CHANNEL,
        };

        let core = ChannelGroup::new(config);

        Self {
            core,
            sample_rate,
            //max_voices,
            soundfont_loaded: false,
            sf_cache: HashMap::new(),
        }
    }

    /*pub fn load_soundfont(&mut self, path: &str) -> Result<(), String> {
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
    }*/

    pub fn load_soundfonts_to_port(&mut self, port: usize, paths: &[String]) -> Result<(), String> {
        let mut soundfonts: Vec<Arc<dyn SoundfontBase>> = Vec::new();
        
        for path in paths {
            if let Some(sf) = self.sf_cache.get(path) {
                soundfonts.push(sf.clone());
            } else {
                match SampleSoundfont::new(
                    path,
                    self.core.stream_params().clone(),
                    SoundfontInitOptions::default(),
                ) {
                    Ok(sf) => {
                        let arc = Arc::new(sf) as Arc<dyn SoundfontBase>;
                        self.sf_cache.insert(path.clone(), arc.clone());
                        soundfonts.push(arc);
                    }
                    Err(e) => return Err(format!("Failed to load {}: {:?}", path, e)),
                }
            }
        }
    
        for ch in (port * 16)..((port + 1) * 16) {
            self.core.send_event(SynthEvent::Channel(
                ch as u32,
                ChannelEvent::Config(ChannelConfigEvent::SetSoundfonts(soundfonts.clone()))
            ));
        }
        
        Ok(())
    }


    /// 直接发送 XSynth 事件（实时安全）
    pub fn send_event(&mut self, event: SynthEvent) {
        self.core.send_event(event);
    }

    fn send_to_all_channels(&mut self, event: ChannelAudioEvent) {
        for ch in 0..NUM_CHANNELS {
            self.core.send_event(SynthEvent::Channel(
                ch,
                ChannelEvent::Audio(event.clone()),
            ));
        }
    }

    pub fn all_notes_off(&mut self) { self.send_to_all_channels(ChannelAudioEvent::AllNotesOff); }
    pub fn all_notes_killed(&mut self) { self.send_to_all_channels(ChannelAudioEvent::AllNotesKilled); }
    pub fn reset_all_controllers(&mut self) { self.send_to_all_channels(ChannelAudioEvent::ResetControl); } // 未来会用，别删
    pub fn sustain_pedal_off(&mut self) { self.send_to_all_channels(ChannelAudioEvent::Control(ControlEvent::Raw(64, 0))); }
    
    pub fn active_voices(&self) -> u64 {
        self.core.voice_count()
    }

    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    pub fn read_samples(&mut self, buffer: &mut [f32]) {
        self.core.read_samples(buffer);
    }

    pub fn system_reset(&mut self) {
        // Reset controllers + kill all notes to ensure clean state
        self.core.send_event(SynthEvent::AllChannels(
            ChannelEvent::Audio(ChannelAudioEvent::ResetControl),
        ));
        self.all_notes_killed();
    }

    pub fn set_percussion_mode(&mut self, channel: u32, percussion: bool) {
        self.core.send_event(SynthEvent::Channel(
            channel,
            ChannelEvent::Config(ChannelConfigEvent::SetPercussionMode(percussion)),
        ));
    }
}