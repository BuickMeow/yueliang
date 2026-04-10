use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, EguiState};
use std::sync::Arc;
use parking_lot::Mutex;
use egui_system_fonts::{set_auto, FontStyle};

// 简单的 async block_on，不需要额外依赖
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering, AtomicUsize};
use std::task::{Context, Poll, Wake, Waker};
use std::thread;
use std::time::Duration;

mod left_bar;
mod transport;
mod sf_manager;
mod sf_list;

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
    // 新增：当前选中的左侧栏标签页
    pub selected_left_tab: Arc<AtomicUsize>,
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
        selected_left_tab: Arc::new(AtomicUsize::new(0)),
    };

    create_egui_editor(
        egui_state,
        state,
        |_, _| {},
        move |egui_ctx, _setter, state| {
            // 先绘制左侧栏（SidePanel 自动带分隔线）
            let selected_tab = left_bar::show_side_panel(egui_ctx, &state.selected_left_tab);
            
            // 主内容区域
            egui::CentralPanel::default().show(egui_ctx, |ui| {
                match selected_tab {
                    left_bar::LeftTab::Transport => {
                        // 构造 transport state
                        let transport_state = transport::TransportState {
                            midi_path: state.midi_path.clone(),
                            pending_midi: state.pending_midi.clone(),
                            picking_midi: state.picking_midi.clone(),
                            midi_player: state.midi_player.clone(),
                        };
                        transport::draw(ui, &transport_state);
                        transport::process_pending(&transport_state);
                    }
                    left_bar::LeftTab::Soundfonts => {
                        // 构造 sf_manager state
                        let sf_state = sf_manager::SfManagerState {
                            params: params.clone(),
                            engine: state.engine.clone(),
                            selected_port: Arc::new(AtomicUsize::new(0)),
                            edit_mode: Arc::new(AtomicBool::new(false)),
                            selected_entries: Arc::new(Mutex::new(Vec::new())),
                            pending_import: Arc::new(Mutex::new(None)),
                            pending_export: Arc::new(Mutex::new(None)),
                            show_menu: Arc::new(AtomicBool::new(false)),
                        };
                        sf_manager::draw(ui, &sf_state);
                    }
                    left_bar::LeftTab::Channels => {
                        // 通道矩阵（256开关+鼓通道配置）
                        ui.label("Channel Matrix (256ch + Drum Config)");
                    }
                }
            });
        }
    )
}
