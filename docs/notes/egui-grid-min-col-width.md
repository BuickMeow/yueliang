# egui::Grid 默认列宽陷阱

最后更新：2026-04-15-10-18-04

---

## 现象

使用 `egui::Grid` 布局等大小的按钮矩阵时，发现**竖列之间的水平间距远大于垂直间距**，即使显式设置了 `.spacing([2.0, 2.0])` 也无效。

---

## 原因

`egui::Grid` 有一个隐藏的默认参数 `min_col_width`，其默认值为 `ui.spacing().interact_size.x`。

在 egui 0.31 的默认样式中：

```rust
interact_size: vec2(40.0, 18.0)
```

因此，即使你的 widget 只有 24px 宽，Grid 也会把**每一列强制拉宽到 40px**。

- 水平步进 ≈ 40px + spacing.x
- 垂直步进 ≈ 24px + spacing.y

差距可达 60% 以上。

---

## 修复

在创建 Grid 时显式设置 `min_col_width`：

```rust
egui::Grid::new("my_grid")
    .spacing([2.0, 2.0])
    .min_col_width(0.0)  // 取消默认的 40px 限制
    .show(ui, |ui| {
        // ...
    });
```

如果希望列宽至少和按钮一样大，也可以写：

```rust
    .min_col_width(24.0)
```

---

## 补充

`min_row_height` 默认是 `interact_size.y`（18px）。如果你的按钮高度大于 18px（例如 24px），行高会自然由按钮高度决定，所以垂直方向通常不会遇到同样的问题。
