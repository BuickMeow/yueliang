use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, EguiState};
use std::sync::Arc;
use parking_lot::Mutex;

// 简单的 async block_on，不需要额外依赖
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll, Wake, Waker};
use std::thread;
use std::time::Duration;

struct DummyWaker;
impl Wake for DummyWaker {
    fn wake(self: Arc<Self>) {}
}

fn simple_block_on<F: Future>(mut future: F) -> F::Output {
    let waker = Waker::from(Arc::new(DummyWaker));
    let mut context = Context::from_waker(&waker);
    let mut future = unsafe { Pin::new_unchecked(&mut future) };

    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(val) => return val,
            Poll::Pending => thread::sleep(Duration::from_millis(10)),
        }
    }
}

pub struct EditorState {
    pub engine: Arc<Mutex<Option<crate::engine::SynthEngine>>>,
    pub midi_player: Arc<Mutex<crate::engine::MidiPlayer>>,
    pub soundfont_path: Arc<parking_lot::Mutex<String>>,
    pub midi_path: Arc<parking_lot::Mutex<String>>,
    pub pending_soundfont: Arc<Mutex<Option<String>>>,
    pub pending_midi: Arc<Mutex<Option<String>>>,
    pub picking_soundfont: Arc<AtomicBool>,
    pub picking_midi: Arc<AtomicBool>,
}

pub fn create(
    params: Arc<crate::YueliangParams>,
    engine: Arc<Mutex<Option<crate::engine::SynthEngine>>>,
    midi_player: Arc<Mutex<crate::engine::MidiPlayer>>,
) -> Option<Box<dyn Editor>> {
    let egui_state = params.editor_state.clone();
    let state = EditorState {
        engine,
        midi_player,
        soundfont_path: params.soundfont_path.clone(),
        midi_path: params.midi_path.clone(),
        pending_soundfont: Arc::new(Mutex::new(None)),
        pending_midi: Arc::new(Mutex::new(None)),
        picking_soundfont: Arc::new(AtomicBool::new(false)),
        picking_midi: Arc::new(AtomicBool::new(false)),
    };

    create_egui_editor(
        egui_state,
        state,
        |_, _| {},
        move |egui_ctx, _setter, state| {
            egui::CentralPanel::default().show(egui_ctx, |ui| {
                ui.heading("Yueliang 🌙");
                ui.separator();

                // ---- SoundFont 选择 ----
                let sf_picking = state.picking_soundfont.load(Ordering::Relaxed);
                let sf_button_text = if sf_picking {
                    "Selecting..."
                } else {
                    "Load SoundFont (.sf2 / .sfz)"
                };

                if ui.add_enabled(!sf_picking, egui::Button::new(sf_button_text)).clicked() {
                    state.picking_soundfont.store(true, Ordering::Relaxed);
                    let pending = state.pending_soundfont.clone();
                    let picking = state.picking_soundfont.clone();

                    thread::spawn(move || {
                        let result = simple_block_on(
                            rfd::AsyncFileDialog::new()
                                .add_filter("SoundFont", &["sf2", "sfz"])
                                .pick_file(),
                        );
                        if let Some(file) = result {
                            *pending.lock() = Some(file.path().to_string_lossy().to_string());
                        }
                        picking.store(false, Ordering::Relaxed);
                    });
                }

                let sf_display = state.soundfont_path.lock().clone();
                ui.label(if sf_display.is_empty() {
                    "No soundfonts has been loaded".into()
                } else {
                    format!("Loaded: {}", sf_display)
                });

                // 处理已选好的 SoundFont
                if let Some(path) = state.pending_soundfont.lock().take() {
                    *state.soundfont_path.lock() = path.clone();
                    if let Some(ref mut engine) = state.engine.lock().as_mut() {
                        match engine.load_soundfont(&path) {
                            Ok(()) => nih_log!("UI 加载 SF2 成功: {}", path),
                            Err(e) => nih_log!("UI 加载 SF2 失败: {}", e),
                        }
                    }
                }

                ui.add_space(16.0);

                // ---- MIDI 选择 ----
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
                                .add_filter("MIDI", &["mid"])
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
                    "No MIDI has been loaded".into()
                } else {
                    format!("Loaded: {}", midi_display)
                });

                // 处理已选好的 MIDI
                if let Some(path) = state.pending_midi.lock().take() {
                    *state.midi_path.lock() = path.clone();
                    match crate::data::midi_loader::load_from_file(&path) {
                        Ok(loaded) => {
                            nih_log!("UI 加载 MIDI 成功: {} events", loaded.events.len());
                            state.midi_player.lock().load(loaded.events, loaded.ppqn);
                        }
                        Err(e) => nih_log!("UI 加载 MIDI 失败: {}", e),
                    }
                }
            });
        },
    )
}
