#[derive(Clone, Debug)]
pub struct MidiEvent {
    pub sample_offset: usize,
    pub channel: u8,
    pub message: MidiMessage,
}

#[derive(Clone, Debug)]
pub enum MidiMessage {
    NoteOn { key: u8, velocity: u8 },
    NoteOff { key: u8 },
    ControlChange { cc: u8, value: u8 },
    ProgramChange { pc: u8 },
    PitchBend { value: i16 },
}