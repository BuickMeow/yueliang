use nih_plug_egui::egui;

#[derive(Default)]
pub struct ListResult {
    pub need_reload: bool,
    pub edit_changed: bool,
    pub drag_released: bool,
}

pub fn draw_draggable_list(
    ui: &mut egui::Ui,
    entries: &mut Vec<crate::SoundfontEntry>,
    selected: &mut Vec<usize>,
    is_edit: &mut bool,
    drag_indices: &mut Vec<usize>,
    drag_insert_idx: &mut usize,
) -> ListResult {
    let item_height = 36.0;
    let mut action: Option<ListAction> = None;
    let mut edit_changed = false;
    let mut drag_released = false;
    let mut item_rects: Vec<egui::Rect> = Vec::new();
    let is_dragging = !drag_indices.is_empty();
    
    for i in 0..entries.len() {
        let entry = &entries[i];
        let is_selected = selected.contains(&i);
        let is_being_dragged = drag_indices.contains(&i);
        
        let top_left = ui.available_rect_before_wrap().min;
        let item_rect = egui::Rect::from_min_size(
            top_left,
            egui::vec2(ui.available_width(), item_height),
        );
        item_rects.push(item_rect);
        
        if is_being_dragged {
            ui.painter().rect_filled(
                item_rect,
                2.0,
                ui.visuals().window_fill.gamma_multiply(0.5),
            );
        } else if is_selected {
            ui.painter().rect_filled(
                item_rect,
                2.0,
                ui.visuals().selection.bg_fill.gamma_multiply(0.3),
            );
        }
        
        let row_id = ui.id().with("sf_row").with(i);
        let row_response = ui.interact(item_rect, row_id, egui::Sense::click_and_drag());
        
        // 绘制内容
        ui.allocate_new_ui(
            egui::UiBuilder::new()
                .max_rect(item_rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
            |ui| {
                ui.set_height(item_height);
                ui.add_space(4.0);
                
                let mut enabled = entry.enabled;
                if ui.checkbox(&mut enabled, "").changed() {
                    action = Some(ListAction::ToggleEnabled(i, enabled));
                }
                
                ui.vertical(|ui| {
                    ui.add_space(2.0);
                    ui.spacing_mut().item_spacing.y = 0.0;
                    ui.add(egui::Label::new(&entry.name).selectable(false));
                    ui.add(egui::Label::new(egui::RichText::new(&entry.path).small()).selectable(false));
                });
            },
        );
        
        // 点击选择
        if !is_dragging && row_response.clicked() {
            handle_selection(i, selected, is_edit, ui);
            edit_changed = true;
        }
        
        // 拖拽开始时记录起始位置到 memory
        if row_response.drag_started() {
            if let Some(pos) = ui.ctx().input(|i| i.pointer.interact_pos()) {
                ui.memory_mut(|m| m.data.insert_temp(egui::Id::new("sf_drag_start"), pos));
            }
            if !is_selected {
                selected.clear();
                selected.push(i);
                *is_edit = true;
                edit_changed = true;
            }
            drag_indices.clear();
            drag_indices.extend_from_slice(selected);
            drag_indices.sort();
            *drag_insert_idx = i;
        }
        
        row_response.context_menu(|ui| {
            if ui.button("Move Up").clicked() && i > 0 {
                action = Some(ListAction::Swap(i, i - 1));
                ui.close_menu();
            }
            if ui.button("Move Down").clicked() && i < entries.len() - 1 {
                action = Some(ListAction::Swap(i, i + 1));
                ui.close_menu();
            }
            if ui.button("Remove").clicked() {
                action = Some(ListAction::Remove(i));
                ui.close_menu();
            }
        });
    }
    
    if is_dragging {
        let pointer_y = ui.ctx().input(|i| i.pointer.interact_pos().map(|p| p.y));
        
        if let Some(py) = pointer_y {
            let mut visible_idx = 0;
            let mut new_insert = 0;
            for i in 0..entries.len() {
                if drag_indices.contains(&i) { continue; }
                if py < item_rects[i].center().y {
                    new_insert = visible_idx;
                    break;
                }
                visible_idx += 1;
                new_insert = visible_idx;
            }
            *drag_insert_idx = new_insert;
        }
        
        let mut visible_idx = 0;
        let mut line_y = None;
        for i in 0..entries.len() {
            if drag_indices.contains(&i) { continue; }
            if visible_idx == *drag_insert_idx {
                line_y = Some(item_rects[i].top());
                break;
            }
            visible_idx += 1;
        }
        if line_y.is_none() {
            line_y = item_rects.last().map(|r| r.bottom());
        }
        if let Some(y) = line_y {
            let x1 = item_rects[0].left();
            let x2 = item_rects[0].right();
            ui.painter().line_segment(
                [egui::pos2(x1, y), egui::pos2(x2, y)],
                egui::Stroke::new(3.0, ui.visuals().selection.bg_fill),
            );
        }
        
        if let Some(pos) = ui.ctx().input(|i| i.pointer.interact_pos()) {
            let ghost_h = item_height * drag_indices.len() as f32;
            let ghost_rect = egui::Rect::from_min_size(
                pos + egui::vec2(12.0, -ghost_h / 2.0),
                egui::vec2(item_rects[0].width() * 0.9, ghost_h),
            );
            ui.painter().rect_filled(
                ghost_rect,
                4.0,
                ui.visuals().window_fill.gamma_multiply(0.95),
            );
            ui.painter().rect_stroke(
                ghost_rect,
                4.0,
                egui::Stroke::new(1.0, ui.visuals().selection.bg_fill),
                egui::StrokeKind::Inside,
            );
        }
        
        if ui.ctx().input(|i| i.pointer.any_released()) {
            drag_released = true;
        }
    }
    
    let need_reload = match action {
        Some(ListAction::ToggleEnabled(i, enabled)) => {
            entries[i].enabled = enabled;
            true
        }
        Some(ListAction::Swap(from, to)) => {
            entries.swap(from, to);
            true
        }
        Some(ListAction::Remove(i)) => {
            entries.remove(i);
            selected.retain(|&x| x != i);
            for x in selected.iter_mut() {
                if *x > i {
                    *x -= 1;
                }
            }
            true
        }
        None => false,
    };
    
    ListResult { need_reload, edit_changed, drag_released }
}

enum ListAction {
    ToggleEnabled(usize, bool),
    Swap(usize, usize),
    Remove(usize),
}

fn handle_selection(i: usize, selected: &mut Vec<usize>, is_edit: &mut bool, ui: &egui::Ui) {
    let modifiers = ui.ctx().input(|i| i.modifiers);
    
    if !*is_edit {
        *is_edit = true;
        selected.clear();
        selected.push(i);
        return;
    }
    
    if !modifiers.command && !modifiers.ctrl && !modifiers.shift
        && selected.len() == 1 && selected[0] == i
    {
        *is_edit = false;
        selected.clear();
        return;
    }
    
    if modifiers.command || modifiers.ctrl {
        if let Some(pos) = selected.iter().position(|&x| x == i) {
            selected.remove(pos);
        } else {
            selected.push(i);
        }
    } else if modifiers.shift && !selected.is_empty() {
        let last = *selected.last().unwrap();
        let range = if i > last { last..=i } else { i..=last };
        for idx in range {
            if !selected.contains(&idx) {
                selected.push(idx);
            }
        }
    } else {
        selected.clear();
        selected.push(i);
    }
}
