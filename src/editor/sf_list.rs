use nih_plug_egui::egui;

pub fn draw_draggable_list(
    ui: &mut egui::Ui,
    entries: &mut Vec<crate::SoundfontEntry>,
    selected: &mut Vec<usize>,
    is_edit: bool,
) {
    let item_height = 48.0;
    let mut action: Option<ListAction> = None;
    
    for i in 0..entries.len() {
        let entry = &entries[i]; // 不可变借用
        let is_selected = selected.contains(&i);
        
        // 绘制条目背景（选中高亮）
        let rect = ui.available_rect_before_wrap();
        let item_rect = egui::Rect::from_min_size(
            rect.min,
            egui::vec2(rect.width(), item_height),
        );
        
        if is_selected {
            ui.painter().rect_filled(
                item_rect,
                0.0,
                ui.visuals().selection.bg_fill.gamma_multiply(0.3),
            );
        }
        
        // 绘制条目内容
        let response = ui.horizontal(|ui| {
            ui.set_height(item_height);
            
            // 启用/禁用开关（显示当前值，但不直接修改）
            let mut enabled = entry.enabled;
            if ui.checkbox(&mut enabled, "").changed() {
                action = Some(ListAction::ToggleEnabled(i, enabled));
            }
            
            ui.vertical(|ui| {
                ui.label(&entry.name);
                ui.small(&entry.instrument_type);
            });
        });
        
        // 点击选择
        if response.response.clicked() {
            handle_selection(i, selected, ui);
        }
        
        // 右键菜单
        response.response.context_menu(|ui| {
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
    
    // 在循环结束后执行操作（避免借用冲突）
    match action {
        Some(ListAction::ToggleEnabled(i, enabled)) => {
            entries[i].enabled = enabled;
        }
        Some(ListAction::Swap(from, to)) => {
            entries.swap(from, to);
        }
        Some(ListAction::Remove(i)) => {
            entries.remove(i);
            // 更新选中状态
            selected.retain(|&x| x != i);
            // 调整大于 i 的索引
            for x in selected.iter_mut() {
                if *x > i {
                    *x -= 1;
                }
            }
        }
        None => {}
    }
}

enum ListAction {
    ToggleEnabled(usize, bool),
    Swap(usize, usize),
    Remove(usize),
}


fn handle_selection(i: usize, selected: &mut Vec<usize>, ui: &egui::Ui) {
    let modifiers = ui.ctx().input(|i| i.modifiers);
    
    if modifiers.command || modifiers.ctrl {
        // Ctrl/Cmd + 点击：切换选择
        if let Some(pos) = selected.iter().position(|&x| x == i) {
            selected.remove(pos);
        } else {
            selected.push(i);
        }
    } else if modifiers.shift && !selected.is_empty() {
        // Shift + 点击：范围选择
        let last = *selected.last().unwrap();
        let range = if i > last { last..=i } else { i..=last };
        for idx in range {
            if !selected.contains(&idx) {
                selected.push(idx);
            }
        }
    } else {
        // 普通点击：单选
        selected.clear();
        selected.push(i);
    }
}
