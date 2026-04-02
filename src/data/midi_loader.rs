use midly::{Smf, TrackEventKind, MidiMessage as MidlyMidiMessage, Timing};
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
        for event in track {
            track_tick += event.delta.as_int() as u64;
            if let TrackEventKind::Midi { channel, message } = event.kind {
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
                    _ => continue,
                };
                events.push(MidiEvent {
                    tick: track_tick,
                    channel: channel.as_int(),
                    message: msg,
                });
            }
            // 故意忽略 MetaMessage::Tempo，完全使用 DAW BPM
        }
    }

    events.sort_by_key(|e| e.tick);

    nih_log!("MIDI loaded: {} events, PPQN = {}", events.len(), ppqn);
    Ok(LoadedMidi { events, ppqn })
}
