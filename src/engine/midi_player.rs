use nih_plug::prelude::*;
use crate::engine::{SynthEngine, midi_mapper};
use crate::data::event::{MidiEvent, MidiMessage};
use crate::YueliangParams;

pub struct MidiPlayer {
    events: Vec<MidiEvent>,
    playhead_samples: i64,
    event_index: usize,
}

impl MidiPlayer {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            playhead_samples: 0,
            event_index: 0
        }
    }
    pub fn process(&mut self, transport: &Transport, engine: &mut SynthEngine, params: &YueliangParams) {
        // 1. 获取 DAW 当前 sample 位置
        // 2. 找出本 buffer 内该触发的事件
        // 3. 应用 velocity_threshold / force_max_velocity
        // 4. 通过 midi_mapper 发送给 engine

        let current_pos = transport.pos_samples().unwrap_or(0);

        if (current_pos - self.playhead_samples).abs() > 1 {
            self.event_index = 0;
        }
        self.playhead_samples = current_pos;

        // TODO: 阶段 4 实现
        // 当 self.events 有数据后，遍历 event_index 到末尾，
        // 找出 sample_offset 在当前 buffer 范围内的事件，
        // 经过力度过滤后通过 midi_mapper 发送给 engine。
        //
        // 伪代码：
        // while self.event_index < self.events.len() {
        //     let event = &self.events[self.event_index];
        //     if event.sample_offset as i64 < current_pos { break; }
        //     if let Some(filtered) = Self::apply_filter(event, params) {
        //         if let Some(synth_event) = midi_mapper::map_midi_event(&filtered) {
        //             engine.send_event(synth_event);
        //         }
        //     }
        //     self.event_index += 1;
        // }
    }
    fn apply_filter(event: &MidiEvent, params: &YueliangParams) -> Option<MidiEvent> {
        match &event.message {
            MidiMessage::NoteOn { key, velocity } => {
                let thresh = params.velocity_threshold.value() as u8;
                let vel = if params.force_max_velocity.value() { 127 } else { *velocity };
                if vel < thresh { return None; }
                Some(MidiEvent {
                    sample_offset:event.sample_offset,
                    channel: event.channel,
                    message: MidiMessage::NoteOn { key: *key, velocity: vel },
                })
            }
            _ => Some(event.clone()),
        }
    }

    pub fn load_events(&mut self, events: Vec<MidiEvent>) {
        self.events = events;
        self.event_index = 0;
    }

    pub fn reset(&mut self) {
        self.playhead_samples = 0;
        self.event_index = 0;
    }
}