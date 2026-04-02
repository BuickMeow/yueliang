use nih_plug::prelude::*;
use crate::engine::SynthEngine;
use crate::YueliangParams;

pub struct Pipeline {
    // 预分配的交错缓冲区，避免 process() 中 Vec 分配
    interleaved: Vec<f32>,
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            interleaved: Vec::new(),
        }
    }

    pub fn render(&mut self, buffer: &mut Buffer, engine: &mut SynthEngine, params: &YueliangParams) {
        let num_frames = buffer.samples();
        self.interleaved.resize(num_frames * 2, 0.0);

        engine.read_samples(&mut self.interleaved);
        let gain_db = params.gain.smoothed.next();
        let gain = util::db_to_gain(gain_db);

        for (i, mut channel_samples) in buffer.iter_samples().enumerate() {
            let l = self.interleaved[i * 2] * gain;
            let r = self.interleaved[i * 2 + 1] * gain;

            let mut iter = channel_samples.iter_mut();
            *iter.next().unwrap() = l;
            *iter.next().unwrap() = r;
        }
    }
}