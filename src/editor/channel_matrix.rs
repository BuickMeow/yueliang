use nih_plug_egui::egui;
use std::sync::Arc;
use parking_lot::Mutex;
use crate::ChannelMatrixMode;

#[derive(Clone)]
pub struct ChannelMatrixState {
    pub matrix: Arc<Mutex<Vec<bool>>>,
    pub drum_matrix: Arc<Mutex<Vec<bool>>>,
    pub mode: Arc<Mutex<ChannelMatrixMode>>,
}

impl ChannelMatrixState {
    pub fn new(
        matrix: Arc<Mutex<Vec<bool>>>,
        drum_matrix: Arc<Mutex<Vec<bool>>>,
        mode: Arc<Mutex<ChannelMatrixMode>>,
    ) -> Self {
        Self { matrix, drum_matrix, mode }
    }
}

pub fn draw(ui: &mut egui::Ui, state: &ChannelMatrixState) {
    ui.heading("Channel Matrix");
    ui.separator();

    let mut matrix = state.matrix.lock();
    let mut drum_matrix = state.drum_matrix.lock();
    let mut mode = state.mode.lock();
    let cell_size = egui::vec2(24.0, 24.0);

    egui::Grid::new("channel_matrix_grid")
        .spacing([2.0, 2.0])
        .min_col_width(0.0)
        .show(ui, |ui| {
            // 左上角模式切换按钮
            let mode_active = *mode == ChannelMatrixMode::Drum;
            let mode_fill = if mode_active {
                ui.visuals().selection.bg_fill
            } else {
                ui.visuals().widgets.inactive.weak_bg_fill
            };
            let mode_response = ui.add_sized(
                cell_size,
                egui::Button::new("Dr.").fill(mode_fill),
            );
            if mode_response.clicked() {
                *mode = match *mode {
                    ChannelMatrixMode::Mute => ChannelMatrixMode::Drum,
                    ChannelMatrixMode::Drum => ChannelMatrixMode::Mute,
                };
            }

            // 列表头：1-16
            for ch in 1..=16 {
                let response = ui.add_sized(cell_size, egui::Button::new(format!("{}", ch)));
                if response.clicked() {
                    match *mode {
                        ChannelMatrixMode::Mute => toggle_column(&mut *matrix, ch - 1),
                        ChannelMatrixMode::Drum => toggle_column(&mut *drum_matrix, ch - 1),
                    }
                }
                if response.secondary_clicked() {
                    match *mode {
                        ChannelMatrixMode::Mute => {
                            if is_solo_column(&*matrix, ch - 1) {
                                unsolo(&mut *matrix);
                            } else {
                                solo_column(&mut *matrix, ch - 1);
                            }
                        }
                        ChannelMatrixMode::Drum => {
                            if is_solo_column(&*drum_matrix, ch - 1) {
                                unsolo(&mut *drum_matrix);
                            } else {
                                solo_column(&mut *drum_matrix, ch - 1);
                            }
                        }
                    }
                }
            }
            ui.end_row();

            // 16 行：A-P
            for port in 0..16 {
                let port_label = format!("{}", (b'A' + port as u8) as char);
                let response = ui.add_sized(cell_size, egui::Button::new(port_label));
                if response.clicked() {
                    match *mode {
                        ChannelMatrixMode::Mute => toggle_row(&mut *matrix, port),
                        ChannelMatrixMode::Drum => toggle_row(&mut *drum_matrix, port),
                    }
                }
                if response.secondary_clicked() {
                    match *mode {
                        ChannelMatrixMode::Mute => {
                            if is_solo_row(&*matrix, port) {
                                unsolo(&mut *matrix);
                            } else {
                                solo_row(&mut *matrix, port);
                            }
                        }
                        ChannelMatrixMode::Drum => {
                            if is_solo_row(&*drum_matrix, port) {
                                unsolo(&mut *drum_matrix);
                            } else {
                                solo_row(&mut *drum_matrix, port);
                            }
                        }
                    }
                }

                for ch in 0..16 {
                    let idx = port * 16 + ch;

                    match *mode {
                        ChannelMatrixMode::Mute => {
                            let active = matrix[idx];
                            let fill = if active {
                                ui.visuals().selection.bg_fill
                            } else {
                                ui.visuals().widgets.inactive.weak_bg_fill
                            };
                            let response = ui.add_sized(
                                cell_size,
                                egui::Button::new("").fill(fill),
                            );
                            if response.clicked() {
                                matrix[idx] = !active;
                            }
                            if response.secondary_clicked() {
                                if is_solo_cell(&*matrix, idx) {
                                    unsolo(&mut *matrix);
                                } else {
                                    solo_cell(&mut *matrix, idx);
                                }
                            }
                        }
                        ChannelMatrixMode::Drum => {
                            let active = drum_matrix[idx];
                            let fill = if active {
                                ui.visuals().selection.bg_fill
                            } else {
                                ui.visuals().widgets.inactive.weak_bg_fill
                            };
                            let label = if active { "🔷" } else { "" };
                            let response = ui.add_sized(
                                cell_size,
                                egui::Button::new(label).fill(fill),
                            );
                            if response.clicked() {
                                drum_matrix[idx] = !active;
                            }
                            if response.secondary_clicked() {
                                if is_solo_cell(&*drum_matrix, idx) {
                                    unsolo(&mut *drum_matrix);
                                } else {
                                    solo_cell(&mut *drum_matrix, idx);
                                }
                            }
                        }
                    }
                }
                ui.end_row();
            }
        });
}

// === 独奏状态检测 ===

fn is_solo_cell(matrix: &[bool], idx: usize) -> bool {
    matrix[idx] && matrix.iter().filter(|&&b| b).count() == 1
}

fn is_solo_row(matrix: &[bool], port: usize) -> bool {
    let start = port * 16;
    let row_on = (start..start + 16).all(|i| matrix[i]);
    let total_on = matrix.iter().filter(|&&b| b).count();
    row_on && total_on == 16
}

fn is_solo_column(matrix: &[bool], ch: usize) -> bool {
    let col_on = (0..16).all(|port| matrix[port * 16 + ch]);
    let total_on = matrix.iter().filter(|&&b| b).count();
    col_on && total_on == 16
}

// === 独奏操作 ===

fn unsolo(matrix: &mut [bool]) {
    for v in matrix.iter_mut() {
        *v = true;
    }
}

fn solo_cell(matrix: &mut [bool], idx: usize) {
    for (i, v) in matrix.iter_mut().enumerate() {
        *v = i == idx;
    }
}

fn solo_row(matrix: &mut [bool], port: usize) {
    let start = port * 16;
    for i in 0..256 {
        matrix[i] = (start..start + 16).contains(&i);
    }
}

fn solo_column(matrix: &mut [bool], ch: usize) {
    for port in 0..16 {
        for c in 0..16 {
            matrix[port * 16 + c] = c == ch;
        }
    }
}

// === 普通开关操作 ===

fn toggle_row(matrix: &mut [bool], port: usize) {
    let start = port * 16;
    let all_on = matrix[start..start + 16].iter().all(|&b| b);
    for i in 0..16 {
        matrix[start + i] = !all_on;
    }
}

fn toggle_column(matrix: &mut [bool], ch: usize) {
    let all_on = (0..16).all(|port| matrix[port * 16 + ch]);
    for port in 0..16 {
        matrix[port * 16 + ch] = !all_on;
    }
}
