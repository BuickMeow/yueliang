pub mod synth;
pub mod pipeline;
pub mod midi_player;
pub mod midi_mapper;
pub mod midi_filter;

pub use synth::{SynthEngine, NUM_CHANNELS};
pub use midi_player::MidiPlayer;
pub use pipeline::Pipeline;