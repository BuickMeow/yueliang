# 拖拽排序插入索引计算踩坑

最后更新：2026-04-14-03-49-32

---

## 现象

拖拽卡片到目标位置松手后，列表顺序没有变化。调试显示 `drag_released=true`、`drag_insert_idx` 看起来也对，但条目总是插回原位。

---

## 根因

**重复修正插入索引。**

`sf_list.rs` 在计算 `drag_insert_idx` 时，循环里已经 `continue` 跳过了被拖拽的项，因此 `visible_idx` 本身就是"删除拖拽项后"的索引：

```rust
for i in 0..entries.len() {
    if drag_indices.contains(&i) { continue; }  // 已跳过拖拽项
    if py < item_rects[i].center().y {
        new_insert = visible_idx;
        break;
    }
    visible_idx += 1;
}
```

这意味着 `drag_insert_idx` 已经直接对应**最终列表**的插入位置。

但 `sf_manager.rs` 在释放处理时又做了一次修正：

```rust
let mut insert_at = drag_insert_idx;
for &removed in drag_indices.iter() {
    if removed < insert_at {
        insert_at = insert_at.saturating_sub(1);
    }
}
```

这导致 `insert_at` 被多减了一次，恰好把拖拽项插回了原位。

---

## 修正

`sf_manager.rs` 中应直接使用 `drag_insert_idx`，不再二次修正：

```rust
let mut insert_at = drag_insert_idx.min(entries.len());
```

---

## 结论

如果 UI 层按"删除后可见项"计算插入位置，业务层就不应再基于原始索引做偏移修正。两层必须约定统一的索引语义。
