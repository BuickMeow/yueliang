use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui};
use std::sync::Arc;
use parking_lot::Mutex;
//use egui_system_fonts::{set_auto, FontStyle};

// 简单的 async block_on，不需要额外依赖
//use std::future::Future;
//use std::pin::Pin;
use std::sync::atomic::{AtomicUsize};
//use std::task::{Context, Poll, Wake, Waker};
//use std::thread;
//use std::time::Duration;

mod left_bar;
mod transport;
mod sf_manager;
mod sf_list;
mod channel_matrix;

pub struct EditorState {
    pub params: Arc<crate::YueliangParams>,
    pub engine: Arc<Mutex<Option<crate::engine::SynthEngine>>>,
    pub midi_player: Arc<Mutex<crate::engine::MidiPlayer>>,
    //pub soundfont_path: Arc<parking_lot::Mutex<String>>,
    pub midi_path: Arc<parking_lot::Mutex<String>>,
    //pub picking_soundfont: Arc<AtomicBool>,
    //pub picking_midi: Arc<AtomicBool>,
    pub selected_left_tab: Arc<AtomicUsize>,
}

pub fn create(
    params: Arc<crate::YueliangParams>,
    engine: Arc<Mutex<Option<crate::engine::SynthEngine>>>,
    midi_player: Arc<Mutex<crate::engine::MidiPlayer>>,
) -> Option<Box<dyn Editor>> {
    let egui_state = params.editor_state.clone();
    let state = EditorState {
        params: params.clone(),  // 需要 clone 一份到 state 里
        engine,
        midi_player,
        //soundfont_path: params.soundfont_path.clone(),
        midi_path: params.midi_path.clone(),
        //pending_soundfont: Arc::new(Mutex::new(None)),  // 如果还有用的话保留
        //picking_soundfont: Arc::new(AtomicBool::new(false)),
        //picking_midi: Arc::new(AtomicBool::new(false)),
        selected_left_tab: Arc::new(AtomicUsize::new(0)),
        // 删掉 sf_xxx 和 pending_midi 相关字段
    };


    create_egui_editor(
        egui_state,
        state,
        |_, _| {},
        move |egui_ctx, _setter, state| {
            let selected_tab = left_bar::show_side_panel(egui_ctx, &state.selected_left_tab);

            // 用 thread_local 或 static 缓存子模块状态
            use std::cell::RefCell;
            thread_local! {
                static TRANSPORT: RefCell<Option<transport::TransportState>> = RefCell::new(None);
                static SF_MANAGER: RefCell<Option<sf_manager::SfManagerState>> = RefCell::new(None);
                static CHANNEL_MATRIX: RefCell<Option<channel_matrix::ChannelMatrixState>> = RefCell::new(None); // 新增
            }

            egui::CentralPanel::default().show(egui_ctx, |ui| {
                match selected_tab {
                    left_bar::LeftTab::Transport => {
                        let t = TRANSPORT.with(|c| {
                            c.borrow_mut().get_or_insert_with(|| {
                                transport::TransportState::new(
                                    state.midi_path.clone(),
                                    state.midi_player.clone(),
                                )
                            }).clone()  // 注意：TransportState 里的字段都是 Arc，clone 很便宜
                        });
                        transport::draw(ui, &t);
                        transport::process_pending(&t);
                    }
                    left_bar::LeftTab::Soundfonts => {
                        let s = SF_MANAGER.with(|c| {
                            c.borrow_mut().get_or_insert_with(|| {
                                sf_manager::SfManagerState::new(
                                    state.params.clone(),
                                    state.engine.clone(),
                                )
                            }).clone()
                        });
                        sf_manager::draw(ui, &s);
                    }
                    left_bar::LeftTab::Channels => {
                        let cm = CHANNEL_MATRIX.with(|c| {
                            c.borrow_mut().get_or_insert_with(|| {
                                channel_matrix::ChannelMatrixState::new(
                                    state.params.channel_matrix.clone(),
                                    state.params.drum_matrix.clone(),
                                    state.params.channel_matrix_mode.clone(),
                                )
                            }).clone()
                        });
                        channel_matrix::draw(ui, &cm);
                    }
                }
            });
        }
    )
}
