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
    let cell_size = egui::vec2(20.0, 20.0);

    egui::Grid::new("channel_matrix_grid")
        .spacing([2.0, 2.0])
        .show(ui, |ui| {
            // 左上角留空
            ui.label("");
            
            // 列表头：1-16
            for ch in 1..=16 {
                let header_btn = egui::Button::new(format!("{}", ch))
                    .min_size(cell_size);
                if ui.add(header_btn).clicked() {
                    toggle_column(&mut *matrix, ch - 1);
                }
            }
            ui.end_row();

            // 16 行：A-P
            for port in 0..16 {
                let port_label = format!("{}", (b'A' + port as u8) as char);
                let row_btn = egui::Button::new(port_label)
                    .min_size(cell_size);
                if ui.add(row_btn).clicked() {
                    toggle_row(&mut *matrix, port);
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
                    
                    if ui.add(btn).clicked() {
                        matrix[idx] = !active;
                    }
                }
                ui.end_row();
            }
        });
}

/// 切换整行：如果该端口所有通道都开着，就全关；否则全开
fn toggle_row(matrix: &mut Vec<bool>, port: usize) {
    let start = port * 16;
    let all_on = matrix[start..start + 16].iter().all(|&b| b);
    for i in 0..16 {
        matrix[start + i] = !all_on;
    }
}

/// 切换整列：如果该通道在所有端口都开着，就全关；否则全开
fn toggle_column(matrix: &mut Vec<bool>, ch: usize) {
    let all_on = (0..16).all(|port| matrix[port * 16 + ch]);
    for port in 0..16 {
        matrix[port * 16 + ch] = !all_on;
    }
}
