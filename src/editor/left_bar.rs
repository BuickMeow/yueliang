use nih_plug_egui::egui;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub const LEFT_BAR_WIDTH: f32 = 48.0;
const BUTTON_SIZE: f32 = 48.0;

/// 左侧面板标签页
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LeftTab {
    Transport = 0,   // 走带控制（播放/暂停/进度）
    Soundfonts,      // 音色库管理（16端口x多音色）
    Channels,        // 通道矩阵（256通道开关+鼓通道配置）
}

impl LeftTab {
    pub fn icon(&self) -> &'static str {
        match self {
            LeftTab::Transport => "▶",
            LeftTab::Soundfonts => "🏦",
            LeftTab::Channels => "🔀",
        }
    }
    
    pub fn tooltip(&self) -> &'static str {
        match self {
            LeftTab::Transport => "Transport / Playback Control",
            LeftTab::Soundfonts => "SoundFont Manager",
            LeftTab::Channels => "Channel Matrix",
        }
    }
    
    pub fn all() -> [LeftTab; 3] {
        [LeftTab::Transport, LeftTab::Soundfonts, LeftTab::Channels]
    }
}

/// 使用 SidePanel 绘制左侧栏
pub fn show_side_panel(egui_ctx: &egui::Context, selected_tab: &Arc<AtomicUsize>) -> LeftTab {
    let mut current = selected_tab.load(Ordering::Relaxed);
    let tabs = LeftTab::all();
    
    if current >= tabs.len() {
        current = 0;
    }
    
    let panel = egui::SidePanel::left("left_bar_panel")
        .exact_width(LEFT_BAR_WIDTH)
        .resizable(false)
        .frame(egui::Frame::new()
            .fill(egui_ctx.style().visuals.window_fill())
            .inner_margin(0.0)
        );
    
    panel.show(egui_ctx, |ui| {
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
        
        ui.vertical(|ui| {
            ui.set_width(LEFT_BAR_WIDTH);
            
            for (idx, tab) in tabs.iter().enumerate() {
                let is_selected = current == idx;
                
                let response = draw_nav_button(ui, tab.icon(), is_selected);
                if response.clicked() {
                    selected_tab.store(idx, Ordering::Relaxed);
                }
                
                // 添加这行：显示 tooltip
                response.on_hover_text(tab.tooltip());
            }
        });
    });
    
    tabs[current]
}

fn draw_nav_button(ui: &mut egui::Ui, icon: &str, is_selected: bool) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(BUTTON_SIZE, BUTTON_SIZE),
        egui::Sense::click(),
    );
    
    // 选中指示器（左侧竖条）
    if is_selected {
        let indicator_rect = egui::Rect::from_min_size(
            rect.min,
            egui::vec2(2.0, BUTTON_SIZE),  // 2px宽，占满高度
        );
        ui.painter().rect_filled(
            indicator_rect,
            0.0,
            ui.visuals().selection.bg_fill,
        );
    }
    
    // 图标颜色
    let icon_color = if response.hovered() || is_selected {
        ui.visuals().strong_text_color()
    } else {
        ui.visuals().text_color().gamma_multiply(0.5)
    };
    
    // 绘制图标（居中，不可选择）
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        icon,
        egui::FontId::proportional(20.0),
        icon_color,
    );
    
    response
}
