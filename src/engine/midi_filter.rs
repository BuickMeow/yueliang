use crate::data::event::{MidiEvent, MidiMessage};
use crate::YueliangParams;

pub fn apply_filter(event: &MidiEvent, params: &YueliangParams) -> Option<MidiEvent> {
    match &event.message {
        MidiMessage::NoteOn { key, velocity } => {
            let thresh = params.velocity_threshold.value() as u8;
            let vel = if params.force_max_velocity.value() { 127 } else { *velocity };
            if vel < thresh {
                return None;
            }
            Some(MidiEvent {
                tick: event.tick,
                channel: event.channel,
                message: MidiMessage::NoteOn { key: *key, velocity: vel },
            })
        }
        _ => Some(event.clone()),
    }
}
