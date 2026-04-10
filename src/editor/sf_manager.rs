use nih_plug::prelude::*;
use nih_plug_egui::egui;
use std::sync::Arc;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};

pub struct SfManagerState {
    pub params: Arc<crate::YueliangParams>,
    pub engine: Arc<Mutex<Option<crate::engine::SynthEngine>>>,
    pub selected_port: Arc<AtomicUsize>,        // 当前选中的端口 0-15
    pub edit_mode: Arc<AtomicBool>,             // 是否处于编辑状态
    pub selected_entries: Arc<Mutex<Vec<usize>>>, // 选中的条目索引
    pub pending_import: Arc<Mutex<Option<String>>>,
    pub pending_export: Arc<Mutex<Option<String>>>,
    pub show_menu: Arc<AtomicBool>,             // 菜单是否展开
}

pub fn draw(ui: &mut egui::Ui, state: &SfManagerState) {
    ui.heading("SoundFont Manager");
    ui.separator();
    
    // === 顶部工具栏 ===
    draw_toolbar(ui, state);
    
    ui.separator();
    
    // === 音色库列表 ===
    draw_sf_list(ui, state);
}

fn draw_toolbar(ui: &mut egui::Ui, state: &SfManagerState) {
    ui.horizontal(|ui| {
        // (1) 端口下拉框 Port A - Port P
        let current_port = state.selected_port.load(Ordering::Relaxed);
        let port_label = format!("Port {}", (b'A' + current_port as u8) as char);
        
        egui::ComboBox::from_id_salt("port_selector")  // 使用 from_id_salt 代替 from_label
            .width(80.0)  // 设置固定宽度
            .selected_text(port_label)
            .show_ui(ui, |ui| {
                for i in 0..16 {
                    let label = format!("Port {}", (b'A' + i as u8) as char);
                    // 关键修复：点击时直接修改 state
                    if ui.selectable_label(current_port == i, label).clicked() {
                        state.selected_port.store(i, Ordering::Relaxed);
                    }
                }
            });
        
        ui.separator();

        // (+) 添加按钮（放在编辑按钮前面）
        let add_btn = egui::Button::new("+");
        if ui.add(add_btn).on_hover_text("Add SoundFont").clicked() {
            spawn_add_soundfont_dialog(state);
        }
        
        // (2) 编辑按钮 🖊️
        let is_edit = state.edit_mode.load(Ordering::Relaxed);
        let edit_btn = egui::Button::new("🖊️").selected(is_edit);
        if ui.add(edit_btn).on_hover_text("Edit Mode").clicked() {
            let new_state = !state.edit_mode.load(Ordering::Relaxed);
            state.edit_mode.store(new_state, Ordering::Relaxed);
            if !new_state {
                state.selected_entries.lock().clear();
            }
        }
        
        // (3) 全选按钮 📦（仅在编辑模式可用）
        let select_all_btn = egui::Button::new("📦");
        let select_all_response = ui.add_enabled(is_edit, select_all_btn);
        if select_all_response.on_hover_text("Select All").clicked() {
            let port_idx = state.selected_port.load(Ordering::Relaxed);
            let entries = &state.params.port_soundfonts.lock()[port_idx].entries;
            let mut selected = state.selected_entries.lock();
            selected.clear();
            for i in 0..entries.len() {
                selected.push(i);
            }
        }
        
        // (4) 移除按钮 🗑️（需要至少选中一个）
        let has_selection = !state.selected_entries.lock().is_empty();
        let remove_btn = egui::Button::new("🗑️");
        let remove_response = ui.add_enabled(is_edit && has_selection, remove_btn);
        if remove_response.on_hover_text("Remove Selected").clicked() {
            remove_selected_entries(state);
        }
        
        ui.separator();
        
        // (5) 菜单按钮 💬
        let menu_btn = egui::Button::new("💬");
        let menu_response = ui.add(menu_btn).on_hover_text("Menu");
        
        if menu_response.clicked() {
            let current = state.show_menu.load(Ordering::Relaxed);
            state.show_menu.store(!current, Ordering::Relaxed);
        }
        
        // 下拉菜单
        if state.show_menu.load(Ordering::Relaxed) {
            egui::Window::new("Menu")
                .fixed_pos(menu_response.rect.left_bottom())
                .collapsible(false)
                .resizable(false)
                .title_bar(false)
                .show(ui.ctx(), |ui| {
                    if ui.button("📥 Import Config").clicked() {
                        spawn_import_dialog(state);
                        state.show_menu.store(false, Ordering::Relaxed);
                    }
                    if ui.button("📤 Export Config").clicked() {
                        spawn_export_dialog(state);
                        state.show_menu.store(false, Ordering::Relaxed);
                    }
                });
        }
    });
}

// 添加音色库对话框
fn spawn_add_soundfont_dialog(state: &SfManagerState) {
    let port_idx = state.selected_port.load(Ordering::Relaxed);
    let params = state.params.clone();
    let engine = state.engine.clone();  // 克隆 engine Arc
    
    std::thread::spawn(move || {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("SoundFont", &["sf2", "sfz"])
            .pick_file() 
        {
            let path_str = path.to_string_lossy().to_string();
            let name = path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Unknown".to_string());
            
            let entry = crate::SoundfontEntry {
                path: path_str.clone(),
                name,
                instrument_type: "Multi".to_string(),
                enabled: true,
            };
            
            // 添加到 params
            params.port_soundfonts.lock()[port_idx].entries.push(entry);
            
            nih_log!("Added soundfont to port {}: {}", port_idx, path_str);
            
            // 关键：通知引擎重新加载该端口的音色
            if let Some(ref mut engine) = engine.lock().as_mut() {
                let paths: Vec<String> = params.port_soundfonts.lock()[port_idx].entries
                    .iter()
                    .filter(|e| e.enabled)
                    .map(|e| e.path.clone())
                    .collect();
                
                if let Err(e) = engine.load_soundfonts_to_port(port_idx, &paths) {
                    nih_log!("Failed to reload soundfonts for port {}: {}", port_idx, e);
                } else {
                    nih_log!("Port {} reloaded with {} soundfonts", port_idx, paths.len());
                }
            }
        }
    });
}



fn draw_sf_list(ui: &mut egui::Ui, state: &SfManagerState) {
    let port_idx = state.selected_port.load(Ordering::Relaxed);
    let is_edit = state.edit_mode.load(Ordering::Relaxed);
    
    // 获取当前端口的音色库列表
    let mut port_soundfonts = state.params.port_soundfonts.lock();
    let entries = &mut port_soundfonts[port_idx].entries;
    let mut selected = state.selected_entries.lock();
    
    egui::ScrollArea::vertical().show(ui, |ui| {
        if entries.is_empty() {
            ui.label("No soundfonts loaded for this port");
            return;
        }
        
        // 调用 sf_list 模块的函数
        crate::editor::sf_list::draw_draggable_list(
            ui,
            entries,
            &mut selected,
            is_edit,
        );
    });
}


fn remove_selected_entries(state: &SfManagerState) {
    let port_idx = state.selected_port.load(Ordering::Relaxed);
    let selected = state.selected_entries.lock().clone();
    
    // 从后往前删除，避免索引错乱
    let mut entries = std::mem::take(&mut state.params.port_soundfonts.lock()[port_idx].entries);
    let mut new_entries = Vec::new();
    
    for (i, entry) in entries.into_iter().enumerate() {
        if !selected.contains(&i) {
            new_entries.push(entry);
        }
    }
    
    state.params.port_soundfonts.lock()[port_idx].entries = new_entries;
    state.selected_entries.lock().clear();
    
    // 通知引擎重新加载
    reload_port_soundfonts(state, port_idx);
}

fn spawn_import_dialog(state: &SfManagerState) {
    // TODO: 文件选择 + JSON 解析
}

fn spawn_export_dialog(state: &SfManagerState) {
    // TODO: 文件选择 + JSON 导出
}

fn reload_port_soundfonts(state: &SfManagerState, port_idx: usize) {
    if let Some(ref mut engine) = state.engine.lock().as_mut() {
        let paths: Vec<String> = state.params.port_soundfonts.lock()[port_idx].entries
            .iter()
            .filter(|e| e.enabled)
            .map(|e| e.path.clone())
            .collect();
        
        if let Err(e) = engine.load_soundfonts_to_port(port_idx, &paths) {
            nih_log!("Failed to reload soundfonts for port {}: {}", port_idx, e);
        } else {
            nih_log!("Port {} reloaded with {} soundfonts", port_idx, paths.len());
        }
    }
}

