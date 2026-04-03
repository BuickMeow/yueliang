use crate::data::event::{MidiEvent, MidiMessage};
use xsynth_core::channel::{ChannelAudioEvent, ChannelEvent, ControlEvent};
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
            ChannelEvent::Audio(ChannelAudioEvent::Control(
                ControlEvent::Raw(*cc, *value)
            ))
        }
        MidiMessage::PitchBend { value } => {
            let normalized = *value as f32 / 8192.0;
            ChannelEvent::Audio(ChannelAudioEvent::Control(
                ControlEvent::PitchBendValue(normalized.clamp(-1.0, 1.0))
            ))
        }
        // 有关RPN的内容，可通过XSynth直接自行处理
    };
    
    Some(SynthEvent::Channel(event.channel as u32, channel_event))
}