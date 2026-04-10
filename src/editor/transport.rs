use nih_plug::prelude::*;
use nih_plug_egui::egui;
use std::sync::Arc;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

// 从 editor.rs 导入 simple_block_on，或者在这里重新定义
use crate::editor::simple_block_on;

pub struct TransportState {
    pub midi_path: Arc<parking_lot::Mutex<String>>,
    pub pending_midi: Arc<Mutex<Option<String>>>,
    pub picking_midi: Arc<AtomicBool>,
    pub midi_player: Arc<Mutex<crate::engine::MidiPlayer>>,
}

pub fn draw(ui: &mut egui::Ui, state: &TransportState) {
    ui.heading("MIDI Transport");
    ui.separator();
    
    // 预留的走带控制区域
    ui.horizontal(|ui| {
        ui.label("Playback: ");
        ui.add_enabled(false, egui::Button::new("⏮"));  // 上一首（预留）
        ui.add_enabled(false, egui::Button::new("⏴"));  // 快退（预留）
        ui.add_enabled(false, egui::Button::new("⏵"));  // 播放（预留）
        ui.add_enabled(false, egui::Button::new("⏹"));  // 停止（预留）
        ui.add_enabled(false, egui::Button::new("⏩"));  // 快进（预留）
        ui.add_enabled(false, egui::Button::new("⏭"));  // 下一首（预留）
    });
    
    ui.add_space(16.0);
    ui.separator();
    
    // MIDI 文件选择
    ui.label("MIDI File:");
    
    let midi_picking = state.picking_midi.load(Ordering::Relaxed);
    let midi_button_text = if midi_picking {
        "Selecting..."
    } else {
        "Load MIDI (.mid)"
    };
    
    if ui.add_enabled(!midi_picking, egui::Button::new(midi_button_text)).clicked() {
        state.picking_midi.store(true, Ordering::Relaxed);
        let pending = state.pending_midi.clone();
        let picking = state.picking_midi.clone();
        
        thread::spawn(move || {
            let result = simple_block_on(
                rfd::AsyncFileDialog::new()
                    .add_filter("MIDI", &["mid", "midi", "kar"])
                    .pick_file(),
            );
            if let Some(file) = result {
                *pending.lock() = Some(file.path().to_string_lossy().to_string());
            }
            picking.store(false, Ordering::Relaxed);
        });
    }
    
    let midi_display = state.midi_path.lock().clone();
    ui.label(if midi_display.is_empty() {
        "No MIDI loaded".into()
    } else {
        format!("Loaded: {}", midi_display)
    });
}

pub fn process_pending(state: &TransportState) {
    if let Some(path) = state.pending_midi.lock().take() {
        *state.midi_path.lock() = path.clone();
        match crate::data::midi_loader::load_from_file(&path) {
            Ok(loaded) => {
                nih_log!("MIDI loaded: {} events", loaded.events.len());
                state.midi_player.lock().load(loaded.events, loaded.ppqn);
            }
            Err(e) => nih_log!("MIDI load failed: {}", e),
        }
    }
}
