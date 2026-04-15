use nih_plug_egui::egui;
use std::sync::Arc;
use parking_lot::Mutex;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DragButton {
    Left,
    Right,
}

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

    let primary_down = ui.input(|i| i.pointer.primary_down());
    let secondary_down = ui.input(|i| i.pointer.secondary_down());
    let any_down = primary_down || secondary_down;

    egui::Grid::new("channel_matrix_grid")
        .spacing([2.0, 2.0])
        .min_col_width(0.0)
        .show(ui, |ui| {
            let drag_id = ui.id().with("drag_state");
            let mut drag_state: Option<(DragButton, usize)> = ui.memory_mut(|mem| mem.data.get_temp(drag_id));

            // 左上角空白占位
            let _ = ui.add_sized(cell_size, egui::Button::new(""));
            
            // 列表头：1-16
            for ch in 1..=16 {
                let response = ui.add_sized(
                    cell_size,
                    egui::Button::new(format!("{}", ch)),
                );
                if response.clicked() {
                    toggle_column(&mut *matrix, ch - 1);
                }
                if response.secondary_clicked() {
                    if is_solo_column(&*matrix, ch - 1) {
                        unsolo(&mut *matrix);
                    } else {
                        solo_column(&mut *matrix, ch - 1);
                    }
                }
            }
            ui.end_row();

            // 16 行：A-P
            for port in 0..16 {
                let port_label = format!("{}", (b'A' + port as u8) as char);
                let response = ui.add_sized(
                    cell_size,
                    egui::Button::new(port_label),
                );
                if response.clicked() {
                    toggle_row(&mut *matrix, port);
                }
                if response.secondary_clicked() {
                    if is_solo_row(&*matrix, port) {
                        unsolo(&mut *matrix);
                    } else {
                        solo_row(&mut *matrix, port);
                    }
                }

                for ch in 0..16 {
                    let idx = port * 16 + ch;
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
                    
                    // 左键单击
                    if response.clicked() {
                        matrix[idx] = !active;
                    }
                    
                    // 右键单击：solo / 取消 solo
                    if response.secondary_clicked() {
                        if is_solo_cell(&*matrix, idx) {
                            unsolo(&mut *matrix);
                        } else {
                            solo_cell(&mut *matrix, idx);
                        }
                    }
                }
                ui.end_row();
            }

            // 鼠标松开时清除拖动状态
            if !any_down {
                drag_state = None;
            }
            ui.memory_mut(|mem| mem.data.insert_temp(drag_id, drag_state));
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
