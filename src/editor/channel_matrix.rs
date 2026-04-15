use nih_plug_egui::egui;
use std::sync::Arc;
use parking_lot::Mutex;

#[derive(Clone)]
pub struct ChannelMatrixState {
    pub matrix: Arc<Mutex<Vec<bool>>>,
}

impl ChannelMatrixState {
    pub fn new(matrix: Arc<Mutex<Vec<bool>>>) -> Self {
        Self { matrix }
    }
}

pub fn draw(ui: &mut egui::Ui, state: &ChannelMatrixState) {
    ui.heading("Channel Matrix");
    ui.separator();

    let mut matrix = state.matrix.lock();
    let cell_size = egui::vec2(24.0, 24.0);

    egui::Grid::new("channel_matrix_grid")
        .spacing([2.0, 2.0])
        .min_col_width(0.0)
        .show(ui, |ui| {
            // 左上角留空
            ui.label("");
            
            // 列表头：1-16
            for ch in 1..=16 {
                let header_btn = egui::Button::new(format!("{}", ch))
                    .min_size(cell_size);
                let response = ui.add(header_btn);
                if response.clicked() {
                    toggle_column(&mut *matrix, ch - 1);
                }
                if response.secondary_clicked() {
                    solo_column(&mut *matrix, ch - 1);
                }
            }
            ui.end_row();

            // 16 行：A-P
            for port in 0..16 {
                let port_label = format!("{}", (b'A' + port as u8) as char);
                let row_btn = egui::Button::new(port_label)
                    .min_size(cell_size);
                let response = ui.add(row_btn);
                if response.clicked() {
                    toggle_row(&mut *matrix, port);
                }
                if response.secondary_clicked() {
                    solo_row(&mut *matrix, port);
                }

                for ch in 0..16 {
                    let idx = port * 16 + ch;
                    let active = matrix[idx];
                    
                    let fill = if active {
                        ui.visuals().selection.bg_fill
                    } else {
                        ui.visuals().widgets.inactive.weak_bg_fill
                    };
                    
                    let btn = egui::Button::new("")
                        .fill(fill)
                        .min_size(cell_size);
                    
                    let response = ui.add(btn);
                    if response.clicked() {
                        matrix[idx] = !active;
                    }
                    if response.secondary_clicked() {
                        solo_cell(&mut *matrix, idx);
                    }
                }
                ui.end_row();
            }
        });
}

fn solo_cell(matrix: &mut Vec<bool>, idx: usize) {
    for (i, v) in matrix.iter_mut().enumerate() {
        *v = i == idx;
    }
}

fn solo_row(matrix: &mut Vec<bool>, port: usize) {
    let start = port * 16;
    for i in 0..256 {
        matrix[i] = (start..start + 16).contains(&i);
    }
}

fn solo_column(matrix: &mut Vec<bool>, ch: usize) {
    for port in 0..16 {
        for c in 0..16 {
            matrix[port * 16 + c] = c == ch;
        }
    }
}

fn toggle_row(matrix: &mut Vec<bool>, port: usize) {
    let start = port * 16;
    let all_on = matrix[start..start + 16].iter().all(|&b| b);
    for i in 0..16 {
        matrix[start + i] = !all_on;
    }
}

fn toggle_column(matrix: &mut Vec<bool>, ch: usize) {
    let all_on = (0..16).all(|port| matrix[port * 16 + ch]);
    for port in 0..16 {
        matrix[port * 16 + ch] = !all_on;
    }
}
