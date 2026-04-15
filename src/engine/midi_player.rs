use nih_plug::prelude::*;
use crate::engine::{SynthEngine, midi_mapper, midi_filter};
use crate::data::event::{MidiEvent, MidiMessage};
use crate::YueliangParams;

use xsynth_core::channel::{ChannelAudioEvent, ChannelEvent, ControlEvent};
use xsynth_core::channel_group::SynthEvent;

const CC_BANK_SELECT_MSB: u8 = 0;
const CC_BANK_SELECT_LSB: u8 = 32;
const CC_RPN_MSB: u8 = 101;
const CC_RPN_LSB: u8 = 100;
const CC_DATA_ENTRY_MSB: u8 = 6;
const CC_DATA_ENTRY_LSB: u8 = 38;

const CC_VOLUME: u8 = 7;
const CC_PANPOT: u8 = 10;
const CC_EXPRESSION: u8 = 11;
const CC_SUSTAIN: u8 = 64;
const CC_RESONANCE: u8 = 71;
const CC_CUTOFF: u8 = 74;
const CC_RELEASE: u8 = 72;
const CC_ATTACK: u8 = 73;

// 以下CC不进行追踪，仅进行参考留档
//const CC_ALL_SOUND_OFF: u8 = 120;
//const CC_ALL_NOTE_OFF: u8 = 123;
//const CC_RESET_ALL_CONTROLLERS: u8 = 121;

/// 需要 Chase 的 CC 列表（按发送顺序排列）
const CHASE_CC_LIST: &[u8] = &[
    CC_RPN_MSB,      // 101 - 必须先设置 RPN
    CC_RPN_LSB,      // 100
    CC_DATA_ENTRY_MSB, // 6
    CC_DATA_ENTRY_LSB, // 38
    CC_BANK_SELECT_MSB, // 0 - Bank Select
    CC_BANK_SELECT_LSB, // 32
    CC_VOLUME,       // 7
    CC_PANPOT,       // 10
    CC_EXPRESSION,   // 11
    CC_SUSTAIN,      // 64
    CC_ATTACK,       // 73
    CC_RELEASE,      // 72
    CC_CUTOFF,       // 74
    CC_RESONANCE,    // 71
];

pub struct MidiPlayer {
    events: Vec<MidiEvent>,
    ppqn: u16,
    event_index: usize,
    was_playing: bool,
    last_tick: f64,
    last_mutes: [bool; 256],
}

impl MidiPlayer {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            ppqn: 480,
            event_index: 0,
            was_playing: false,
            last_tick: 0.0,
            last_mutes: [true; 256],
        }
    }

    /// 实时安全：只接收已解析好的数据，不碰文件系统
    pub fn load(&mut self, events: Vec<MidiEvent>, ppqn: u16) {
        //self.state_table.build(&events); 
        self.events = events;
        self.ppqn = ppqn;
        self.event_index = 0;
        self.last_tick = 0.0;
        self.last_mutes = [true; 256];
    }

    pub fn process(
        &mut self,
        transport: &Transport,
        engine: &mut SynthEngine,
        params: &YueliangParams,
        num_frames: usize,
        mutes: &[bool; 256],
    ) {
        let is_playing = transport.playing;

        // 1. DAW 暂停
        if !is_playing {
            if self.was_playing {
                engine.all_notes_off();
                engine.sustain_pedal_off();
            }
            self.was_playing = false;
            return;
        }

        // 2. DAW 开始播放（从暂停恢复）
        if is_playing && !self.was_playing {
            engine.system_reset();
        }

        let bpm = transport.tempo.unwrap_or(120.0);
        let sample_rate = engine.sample_rate() as f64;
        let pos_beats = transport.pos_beats().unwrap_or(0.0);

        let current_tick = pos_beats * self.ppqn as f64;
        let tick_delta = num_frames as f64 * bpm * self.ppqn as f64 / (60.0 * sample_rate);
        let end_tick = current_tick + tick_delta;

        // 3. 播放头跳转检测 + Chase
        if (current_tick - self.last_tick).abs() > self.ppqn as f64 * 0.5 {
            engine.system_reset();
            self.event_index = self.find_event_index(current_tick);

            for event in self.chase_events(current_tick as u64) {
                if let Some(synth_event) = midi_mapper::map_midi_event(&event) {
                    engine.send_event(synth_event);
                }
            }
        }

        // === 通道静音/恢复状态变化处理 ===
        for ch in 0..256 {
            // 1. 从发声变为静音：立即切断该通道所有音符并松开踏板
            if self.last_mutes[ch] && !mutes[ch] {
                engine.send_event(SynthEvent::Channel(
                    ch as u32,
                    ChannelEvent::Audio(ChannelAudioEvent::AllNotesOff),
                ));
                engine.send_event(SynthEvent::Channel(
                    ch as u32,
                    ChannelEvent::Audio(ChannelAudioEvent::Control(ControlEvent::Raw(64, 0))),
                ));
            }
            
            // 2. 从静音恢复：Chase 该通道的最新控制器状态
            if !self.last_mutes[ch] && mutes[ch] {
                for event in self.chase_single_channel(current_tick as u64, ch as u8) {
                    if let Some(synth_event) = midi_mapper::map_midi_event(&event) {
                        engine.send_event(synth_event);
                    }
                }
            }
        }
        self.last_mutes = *mutes;


        // 4. 分发本 buffer 内的事件
        while self.event_index < self.events.len() {
            let event = &self.events[self.event_index];
            let event_tick = event.tick as f64;

            if event_tick >= end_tick {
                break;
            }

            if event_tick >= current_tick {
                if let Some(filtered) = midi_filter::apply_filter(event, params, mutes) {
                    if let Some(synth_event) = midi_mapper::map_midi_event(&filtered) {
                        engine.send_event(synth_event);
                    }
                }
            }

            self.event_index += 1;
        }

        self.last_tick = end_tick;
        self.was_playing = true;
    }

    fn find_event_index(&self, target_tick: f64) -> usize {
        self.events.partition_point(|e| (e.tick as f64) < target_tick)
    }

    pub fn reset(&mut self) {
        self.event_index = 0;
        self.last_tick = 0.0;
        self.was_playing = false;
    }

    /// 向前搜索指定 tick 之前的最新 CC/PC/PB 状态
    fn chase_events(&self, target_tick: u64) -> Vec<MidiEvent> {
        let mut result = Vec::new();
        
        if self.events.is_empty() {
            return result;
        }

        // 通道状态缓存
        let num_channels = crate::engine::NUM_CHANNELS as usize;
        let mut cc_state: Vec<[Option<u8>; 128]> = vec![[None; 128]; num_channels];
        let mut pc_state: Vec<Option<u8>> = vec![None; num_channels];
        let mut pb_state: Vec<Option<i16>> = vec![None; num_channels];

        // 线性扫描到 target_tick 之前（不限制范围）
        for event in &self.events {
            if event.tick >= target_tick {
                break;
            }

            let ch = event.channel as usize;
            if ch >= num_channels {
                continue;
            }

            match event.message {
                MidiMessage::ControlChange { cc, value } => {
                    cc_state[ch][cc as usize] = Some(value);
                }
                MidiMessage::ProgramChange { pc } => {
                    pc_state[ch] = Some(pc);
                }
                MidiMessage::PitchBend { value } => {
                    pb_state[ch] = Some(value);
                }
                _ => {}
            }
        }

        // 生成 Chase 事件
        for ch in 0..num_channels {
            let ch_u8 = ch as u8;

            // 1. 按顺序 Chase CC
            for &cc_num in CHASE_CC_LIST {
                if let Some(value) = cc_state[ch][cc_num as usize] {
                    result.push(MidiEvent {
                        tick: target_tick,
                        channel: ch_u8,
                        message: MidiMessage::ControlChange { cc: cc_num, value },
                    });
                }
            }

            // 2. Chase PC
            if let Some(pc) = pc_state[ch] {
                result.push(MidiEvent {
                    tick: target_tick,
                    channel: ch_u8,
                    message: MidiMessage::ProgramChange { pc },
                });
            }

            // 3. Chase Pitch Bend
            if let Some(value) = pb_state[ch] {
                result.push(MidiEvent {
                    tick: target_tick,
                    channel: ch_u8,
                    message: MidiMessage::PitchBend { value },
                });
            }
        }

        result
    }

    /*pub fn ppqn(&self) -> u16 {
        self.ppqn
    }*/

        pub fn chase_single_channel(&self, target_tick: u64, channel: u8) -> Vec<MidiEvent> {
        let mut result = Vec::new();
        if self.events.is_empty() {
            return result;
        }

        let mut cc_state: [Option<u8>; 128] = [None; 128];
        let mut pc_state: Option<u8> = None;
        let mut pb_state: Option<i16> = None;

        for event in &self.events {
            if event.tick >= target_tick {
                break;
            }
            if event.channel != channel {
                continue;
            }
            match event.message {
                MidiMessage::ControlChange { cc, value } => {
                    cc_state[cc as usize] = Some(value);
                }
                MidiMessage::ProgramChange { pc } => {
                    pc_state = Some(pc);
                }
                MidiMessage::PitchBend { value } => {
                    pb_state = Some(value);
                }
                _ => {}
            }
        }

        for &cc_num in CHASE_CC_LIST {
            if let Some(value) = cc_state[cc_num as usize] {
                result.push(MidiEvent {
                    tick: target_tick,
                    channel,
                    message: MidiMessage::ControlChange { cc: cc_num, value },
                });
            }
        }
        if let Some(pc) = pc_state {
            result.push(MidiEvent {
                tick: target_tick,
                channel,
                message: MidiMessage::ProgramChange { pc },
            });
        }
        if let Some(value) = pb_state {
            result.push(MidiEvent {
                tick: target_tick,
                channel,
                message: MidiMessage::PitchBend { value },
            });
        }
        result
    }
}
