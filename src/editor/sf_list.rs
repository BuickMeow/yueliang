use nih_plug_egui::egui;

pub fn draw_draggable_list(
    ui: &mut egui::Ui,
    entries: &mut Vec<crate::SoundfontEntry>,
    selected: &mut Vec<usize>,
    is_edit: &mut bool,
) -> (bool, bool) {
    let item_height = 36.0; // 可以适当调小
    let mut action: Option<ListAction> = None;
    let mut edit_changed = false;
    
    for i in 0..entries.len() {
        let entry = &entries[i];
        let is_selected = selected.contains(&i);
        
        let top_left = ui.available_rect_before_wrap().min;
        let item_rect = egui::Rect::from_min_size(
            top_left,
            egui::vec2(ui.available_width(), item_height),
        );
        
        // 绘制选中背景
        if is_selected {
            ui.painter().rect_filled(
                item_rect,
                2.0,
                ui.visuals().selection.bg_fill.gamma_multiply(0.3),
            );
        }
        
        // 先注册整行点击（checkbox 之后会覆盖它）
        let row_id = ui.id().with("sf_row").with(i);
        let row_response = ui.interact(item_rect, row_id, egui::Sense::click());
        
        // 再绘制内容
        ui.allocate_new_ui(
            egui::UiBuilder::new()
                .max_rect(item_rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
            |ui| {
                ui.set_height(item_height);
                
                ui.add_space(4.0); // ← 复选框往右挪 4 像素
                
                let mut enabled = entry.enabled;
                if ui.checkbox(&mut enabled, "").changed() {
                    action = Some(ListAction::ToggleEnabled(i, enabled));
                }
                
                //ui.add_space(2.0);
                ui.vertical(|ui| {
                    ui.add_space(4.0);
                    ui.spacing_mut().item_spacing.y = 0.0;
                    ui.label(&entry.name);
                    ui.small(&entry.path);
                });
            },
        );
        
        if row_response.clicked() {
            handle_selection(i, selected, is_edit, ui);
            edit_changed = true;
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
    
    (need_reload, edit_changed)
}

// handle_selection 保持不变

enum ListAction {
    ToggleEnabled(usize, bool),
    Swap(usize, usize),
    Remove(usize),
}

fn handle_selection(i: usize, selected: &mut Vec<usize>, is_edit: &mut bool, ui: &egui::Ui) {
    let modifiers = ui.ctx().input(|i| i.modifiers);
    
    if !*is_edit {
        // 不在编辑模式：进入编辑模式并单选该卡片
        *is_edit = true;
        selected.clear();
        selected.push(i);
        return;
    }
    
    // 在编辑模式下，普通点击已选中的唯一卡片 -> 退出编辑模式
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
