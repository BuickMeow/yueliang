use nih_plug::prelude::*;
use crate::engine::{SynthEngine, midi_mapper, midi_filter};
use crate::data::event::{MidiEvent, MidiMessage};
use crate::YueliangParams;

const CC_BANK_SELECT_MSB: u8 = 0;
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

// 以下CC不进行追踪
const CC_ALL_SOUND_OFF: u8 = 120;
const CC_ALL_NOTE_OFF: u8 = 123;
const CC_RESET_ALL_CONTROLLERS: u8 = 121;

struct StateTable {
    // [channel][cc_number] -> Vec<(tick, value)>
    cc: Vec<Vec<Vec<(u64, u8)>>>,
    pc: Vec<Vec<(u64, u8)>>,
    pb: Vec<Vec<(u64, i16)>>,
}

impl StateTable {
    fn new() -> Self {
        let channels = crate::engine::NUM_CHANNELS as usize;
        Self {
            cc: vec![vec![Vec::new(); 128]; channels],
            pc: vec![Vec::new(); channels],
            pb: vec![Vec::new(); channels],
        }
    }

    fn build(&mut self, events: &[MidiEvent]) {
        // 清空旧数据
        for ch in &mut self.cc {
            for vec in ch {
                vec.clear();
            }
        }
        for ch in &mut self.pc {
            ch.clear();
        }
        for ch in &mut self.pb {
            ch.clear();
        }

        let max_ch = self.cc.len() as u8;
        for e in events {
            if e.channel >= max_ch {
                continue;
            }
            let ch = e.channel as usize;
            match e.message {
                MidiMessage::ControlChange { cc, value } => {
                    self.cc[ch][cc as usize].push((e.tick, value));
                }
                MidiMessage::ProgramChange { pc } => {
                    self.pc[ch].push((e.tick, pc));
                }
                MidiMessage::PitchBend { value } => {
                    self.pb[ch].push((e.tick, value));
                }
                _ => {}
            }
        }
    }

    fn snapshot_at(&self, tick: u64) -> Vec<MidiEvent> {
        let mut out = Vec::new();

        for ch in 0..self.cc.len() {
            // 1. 先收集所有需要注入的 CC
            let mut cc_events: Vec<(u8, u8)> = Vec::new();
            for cc_num in 0..128 {
                let events = &self.cc[ch][cc_num];
                if events.is_empty() { continue; }
                let idx = events.partition_point(|(t, _)| *t < tick);
                if idx > 0 {
                    if let Some(&(_, value)) = events.get(idx - 1) {
                        cc_events.push((cc_num as u8, value));
                    }
                }
            }

            // 2. 按 RPN 设置顺序排序：101, 100, 其他, 6, 38
            cc_events.sort_by_key(|(cc, _)| match *cc {
                CC_RPN_MSB => 0,
                CC_RPN_LSB => 1,
                // CC_NRPN_MSB => 2,
                // CC_NRPN_LSB => 3,
                CC_DATA_ENTRY_MSB => 4,
                CC_DATA_ENTRY_LSB => 5,
                _ => 100,
            });

            for (cc, value) in cc_events {
                out.push(MidiEvent {
                    tick,
                    channel: ch as u8,
                    message: MidiMessage::ControlChange { cc, value },
                });
            }

            if !self.pc[ch].is_empty() {
                let idx = self.pc[ch].partition_point(|(t, _)| *t < tick);
                if idx > 0 {
                    if let Some(&(_, pc)) = self.pc[ch].get(idx - 1) {
                        out.push(MidiEvent {
                            tick,
                            channel: ch as u8,
                            message: MidiMessage::ProgramChange { pc },
                        });
                    }
                }
            }

            if !self.pb[ch].is_empty() {
                let idx = self.pb[ch].partition_point(|(t, _)| *t < tick);
                if idx > 0 {
                    if let Some(&(_, value)) = self.pb[ch].get(idx - 1) {
                        out.push(MidiEvent {
                            tick,
                            channel: ch as u8,
                            message: MidiMessage::PitchBend { value },
                        });
                    }
                }
            }
        }

        out
    }
}


pub struct MidiPlayer {
    events: Vec<MidiEvent>,
    ppqn: u16,
    event_index: usize,
    was_playing: bool,
    last_tick: f64,
    state_table: StateTable,
}

impl MidiPlayer {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            ppqn: 480,
            event_index: 0,
            was_playing: false,
            last_tick: 0.0,
            state_table: StateTable::new(),
        }
    }

    /// 实时安全：只接收已解析好的数据，不碰文件系统
    pub fn load(&mut self, events: Vec<MidiEvent>, ppqn: u16) {
        self.state_table.build(&events); 
        self.events = events;
        self.ppqn = ppqn;
        self.event_index = 0;
        self.last_tick = 0.0;
    }

    pub fn process(
        &mut self,
        transport: &Transport,
        engine: &mut SynthEngine,
        params: &YueliangParams,
        num_frames: usize,
    ) {
        let is_playing = transport.playing;

        // 1. DAW 暂停：发送 AllNotesOff（让音符进入 release）
        if !is_playing {
            if self.was_playing {
                engine.all_notes_off();
            }
            self.was_playing = false;
            return;
        }

        // 2. DAW 开始播放（从暂停恢复）：先发送 AllNotesKilled 切断残留声音
        if is_playing && !self.was_playing {
            engine.all_notes_killed();
        }

        let bpm = transport.tempo.unwrap_or(120.0);
        let sample_rate = engine.sample_rate() as f64;
        let pos_beats = transport.pos_beats().unwrap_or(0.0);

        // tick 映射：1 beat = ppqn ticks
        let current_tick = pos_beats * self.ppqn as f64;
        let tick_delta = num_frames as f64 * bpm * self.ppqn as f64 / (60.0 * sample_rate);
        let end_tick = current_tick + tick_delta;

        // 播放头跳转检测（scrub / 循环 / 暂停后恢复）
        if (current_tick - self.last_tick).abs() > self.ppqn as f64 * 0.5 {
            engine.all_notes_killed();  //可能会删
            self.event_index = self.find_event_index(current_tick);

            // 注入跳转前的最新状态事件
            for event in self.state_table.snapshot_at(current_tick as u64) {
                if let Some(synth_event) = midi_mapper::map_midi_event(&event) {
                    engine.send_event(synth_event);
                }
            }
        }

        // 分发本 buffer 内的事件
        while self.event_index < self.events.len() {
            let event = &self.events[self.event_index];
            let event_tick = event.tick as f64;

            if event_tick >= end_tick {
                break;
            }

            if event_tick >= current_tick {
                if let Some(filtered) = midi_filter::apply_filter(event, params) {
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
}
