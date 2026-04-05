use midly::{Smf, TrackEventKind, MidiMessage as MidlyMidiMessage, Timing, MetaMessage};
use crate::data::event::{MidiEvent, MidiMessage};
use nih_plug::prelude::nih_log;

pub struct LoadedMidi {
    pub events: Vec<MidiEvent>,
    pub ppqn: u16,
}

/// 非实时安全：会分配内存、读取文件、依赖 midly
pub fn load_from_file(path: &str) -> Result<LoadedMidi, String> {
    let data = std::fs::read(path).map_err(|e| format!("Read MIDI failed: {}", e))?;
    let smf = Smf::parse(&data).map_err(|e| format!("Parse MIDI failed: {:?}", e))?;

    let ppqn = match smf.header.timing {
        Timing::Metrical(ppqn) => ppqn.as_int(),
        Timing::Timecode(_, _) => {
            return Err("Timecode-based MIDI timing is not supported".into());
        }
    };

    let mut events = Vec::new();

    for track in &smf.tracks {
        let mut track_tick = 0u64;
        let mut current_port: u8 = 0;

        for event in track {
            track_tick += event.delta.as_int() as u64;
            match event.kind {
                TrackEventKind::Meta(MetaMessage::MidiPort(port)) => {
                    current_port = port.as_int();
                }
                TrackEventKind::Midi { channel, message } => {
                    let mapped_channel = (current_port as u16)
                        .saturating_mul(16)
                        .saturating_add(channel.as_int() as u16);
                    
                    // 逻辑通道数不会超过指定的256通道数
                    if mapped_channel >= crate::engine::NUM_CHANNELS as u16 {
                        continue;
                    }

                    let msg = match message {
                        MidlyMidiMessage::NoteOn { key, vel } => {
                            if vel.as_int() == 0 {
                                MidiMessage::NoteOff { key: key.as_int() }
                            } else {
                                MidiMessage::NoteOn {
                                    key: key.as_int(),
                                    velocity: vel.as_int(),
                                }
                            }
                        }
                        MidlyMidiMessage::NoteOff { key, .. } => {
                            MidiMessage::NoteOff { key: key.as_int() }
                        }
                        MidlyMidiMessage::ProgramChange { program } => {
                            MidiMessage::ProgramChange { pc: program.as_int() }
                        }
                        MidlyMidiMessage::Controller { controller, value } => {
                            MidiMessage::ControlChange { 
                                cc: controller.as_int(), 
                                value: value.as_int() 
                            }
                        }
                        MidlyMidiMessage::PitchBend { bend } => {
                            MidiMessage::PitchBend { 
                                value: bend.as_int() as i16, 
                            }
                        }
                        _ => continue,
                    };

                    events.push(MidiEvent {
                        tick: track_tick,
                        channel: mapped_channel as u8,
                        message: msg,
                    });
                }
                _ => {}
            }
        }
    }

    events.sort_by_key(|e| e.tick);

    nih_log!("MIDI loaded: {} events, PPQN = {}", events.len(), ppqn);
    Ok(LoadedMidi { events, ppqn })
}
