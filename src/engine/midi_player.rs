use nih_plug::prelude::*;
use crate::engine::{SynthEngine, midi_mapper, midi_filter};
use crate::data::event::MidiEvent;
use crate::YueliangParams;

pub struct MidiPlayer {
    events: Vec<MidiEvent>,
    ppqn: u16,
    event_index: usize,
    was_playing: bool,
    last_tick: f64,
}

impl MidiPlayer {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            ppqn: 480,
            event_index: 0,
            was_playing: false,
            last_tick: 0.0,
        }
    }

    /// 实时安全：只接收已解析好的数据，不碰文件系统
    pub fn load(&mut self, events: Vec<MidiEvent>, ppqn: u16) {
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

        // 走带停止：立即切断所有声音
        if !is_playing {
            if self.was_playing {
                engine.reset();
            }
            self.was_playing = false;
            return;
        }

        let bpm = transport.tempo.unwrap_or(120.0);
        let sample_rate = engine.sample_rate() as f64;
        let pos_beats = transport.pos_beats().unwrap_or(0.0);

        // tick 映射：1 beat = ppqn ticks
        let current_tick = pos_beats * self.ppqn as f64;
        let tick_delta = num_frames as f64 * bpm * self.ppqn as f64 / (60.0 * sample_rate);
        let end_tick = current_tick + tick_delta;

        // 播放头跳转检测（scrub / 循环 / 暂停后恢复）
        if (current_tick - self.last_tick).abs() > self.ppqn as f64 {
            self.event_index = self.find_event_index(current_tick);
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
