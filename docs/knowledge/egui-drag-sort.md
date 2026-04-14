# egui 中实现列表拖拽排序

最后更新：2026-04-14-03-49-32

---

## 核心思路

egui 没有内置的 List Reorder 组件，但可以通过 `Response::drag_started()` / `drag_released()` 配合 `Painter` 手动实现。

---

## 1. 整行交互检测

不能依赖 `ui.horizontal()` 返回的 response（它只覆盖内容宽度），需要先用 `ui.interact()` 在整行 rect 上注册交互。

```rust
let row_id = ui.id().with("sf_row").with(i);
let row_response = ui.interact(item_rect, row_id, egui::Sense::click_and_drag());
```

- `click_and_drag()` 同时支持点击选择和拖拽启动
- 在 `allocate_new_ui()` 绘制内容**之前**调用 `ui.interact()`，可确保内部 checkbox 等子元素后创建、优先级更高，不会被行点击覆盖

---

## 2. 内容绘制与点击覆盖

使用 `allocate_new_ui()` 在固定 `max_rect` 内绘制内容：

```rust
ui.allocate_new_ui(
    egui::UiBuilder::new()
        .max_rect(item_rect)
        .layout(egui::Layout::left_to_right(egui::Align::Center)),
    |ui| {
        ui.checkbox(&mut enabled, "");
        ui.label("...");
    },
);
```

注意：`allocate_ui_at_rect()` 已被弃用，应改用 `allocate_new_ui()`。

---

## 3. 拖拽状态管理

拖拽状态需要跨帧持久化。推荐放在共享的 `Arc<Mutex<...>>` 中：

```rust
pub struct EditorState {
    pub sf_drag_indices: Arc<Mutex<Vec<usize>>>,      // 正在拖拽的原始索引
    pub sf_drag_insert_idx: Arc<AtomicUsize>,         // 目标插入位置
}
```

**拖拽开始**：
```rust
if row_response.drag_started() {
    drag_indices.clear();
    drag_indices.extend_from_slice(selected);
    drag_indices.sort();
}
```

**拖拽中**：计算鼠标 Y 坐标对应的可见项插入位置，绘制插入线和幽灵卡片。

**拖拽释放**：
```rust
if ui.ctx().input(|i| i.pointer.primary_released()) {
    drag_released = true;
}
```

---

## 4. 禁止文字选中

egui 的 `Label` 默认可选中（某些环境下长按会触发）。明确禁用：

```rust
ui.add(egui::Label::new(text).selectable(false));
```

---

## 5. 常见陷阱

| 问题 | 原因 | 修复 |
|------|------|------|
| 复选框点不上 | `ui.interact` 在内容绘制之后调用，覆盖了 checkbox | 先 `ui.interact`，再 `allocate_new_ui` |
| 行间距变大 / rect 重叠 | `allocate_response` + `allocate_new_ui` 两次推进 cursor | 只用 `allocate_new_ui` 一次，配合 `ui.interact` 检测点击 |
| 拖拽检测不到 | `Sense::click()` 不包含 drag | 改用 `Sense::click_and_drag()` |
