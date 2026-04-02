use crate::data::event::{MidiEvent, MidiMessage};
use xsynth_core::channel::{ChannelAudioEvent, ChannelEvent};
use xsynth_core::channel_group::SynthEvent;

pub fn map_midi_event(event: &MidiEvent) -> Option<SynthEvent> {
    let channel_event = match &event.message {
        MidiMessage::NoteOn { key, velocity } => {
            ChannelEvent::Audio(ChannelAudioEvent::NoteOn { key: *key, vel: *velocity })
        }
        MidiMessage::NoteOff { key } => {
            ChannelEvent::Audio(ChannelAudioEvent::NoteOff { key: *key })
        }
        MidiMessage::ProgramChange { pc } => {
            ChannelEvent::Audio(ChannelAudioEvent::ProgramChange(*pc))
        }
        MidiMessage::ControlChange { cc, value } => {
            // TODO: 映射到 XSynth ControlEvent
            return None;
        }
        MidiMessage::PitchBend { value } => {
            // TODO: 映射到 XSynth PitchBend
            return None;
        }
    };
    
    Some(SynthEvent::Channel(event.channel as u32, channel_event))
}