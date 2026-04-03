# nih-plug + egui 集成要点

最后更新：2026-04-03-11-04-11

---

## 1. `create_egui_editor` 基本签名

```rust
pub fn create_egui_editor<T, B, U>(
    egui_state: Arc<EguiState>,
    user_state: T,
    build: B,
    update: U,
) -> Option<Box<dyn Editor>>
where
    T: 'static + Send + Sync,
    B: Fn(&Context, &mut T) + 'static + Send + Sync,
    U: Fn(&Context, &ParamSetter, &mut T) + 'static + Send + Sync,
```

- `egui_state`：窗口大小、打开状态，通常来自 `params.editor_state`。
- `user_state`：自定义状态，会在 `Editor` 生命周期内持久保存。
- `build`：仅在窗口创建时调用一次，用于初始化 egui 资源。
- `update`：每帧调用，负责绘制 UI。注意它**只接收 `&mut T`，不接收 `&mut Plugin`**。

---

## 2. `update` 闭包无法直接访问 Plugin 实例

`update` 的签名决定了你无法在 UI 绘制时直接修改 `Yueliang` / `Plugin` 本身的字段。

**解决方案**：把需要共享的状态（如 `engine`、`midi_player`）用 `Arc<Mutex<...>>` 包装，在 `editor()` 中 clone 一份放进 `user_state`。

```rust
fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
    let engine = self.engine.clone();           // Arc<Mutex<...>>
    let midi_player = self.midi_player.clone(); // Arc<Mutex<...>>
    create_egui_editor(
        self.params.editor_state.clone(),
        EditorState { engine, midi_player },
        |_, _| {},
        move |egui_ctx, _setter, state| {
            // 在这里通过 state.engine.lock() 访问
        },
    )
}
```

---

## 3. `AsyncExecutor` 与 `BackgroundTask` 的限制

`AsyncExecutor` 提供了 `execute_background(task)` 方法，可以把任务扔到后台线程执行。

```rust
pub struct AsyncExecutor<P: Plugin> {
    execute_background: Arc<dyn Fn(P::BackgroundTask) + Send + Sync>,
    execute_gui: Arc<dyn Fn(P::BackgroundTask) + Send + Sync>,
}
```

`Plugin::task_executor()` 返回的闭包签名是：

```rust
pub type TaskExecutor<P> = Box<dyn Fn(<P as Plugin>::BackgroundTask) + Send>;
```

**关键限制**：`task_executor` 的闭包**不接收 `&mut self`**，因此无法直接在闭包里修改 `Plugin` 的字段。如果一定要通过 `BackgroundTask` 做加载，需要先借助 `Arc<Mutex<...>>` 共享状态。

---

## 4. 在 egui 中使用 `rfd` 做文件选择

`rfd` 提供了同步和异步两种 API。在 egui 的 `update` 闭包中最简单的做法是**直接调用同步版** `rfd::FileDialog::new().pick_file()`：

```rust
if ui.button("选择文件").clicked() {
    if let Some(path) = rfd::FileDialog::new()
        .add_filter("SoundFont", &["sf2"])
        .pick_file()
    {
        let path_str = path.to_string_lossy().to_string();
        // 直接加载（在 UI 线程执行）
        state.engine.lock().as_mut().unwrap().load_soundfont(&path_str);
    }
}
```

虽然理论上阻塞 UI 线程不太好，但对于系统原生的文件选择器来说，这是 VST 插件中的常见做法，DAW 通常也能正常挂起窗口。

---

## 5. 音频线程与 UI 线程共享状态

由于 `process()` 和 `update()` 都需要访问 `engine` 和 `midi_player`，最实用的方案是用 `Arc<parking_lot::Mutex<T>>` 共享：

```rust
pub struct Yueliang {
    engine: Arc<Mutex<Option<engine::SynthEngine>>>,
    midi_player: Arc<Mutex<engine::MidiPlayer>>,
}
```

**为什么可以用 `Mutex`？**
- `parking_lot::Mutex` 的 lock/unlock 开销极低（几纳秒级别）。
- UI 加载的频率极低（只在用户手动点击时发生），`process()` 每次 lock 只持有一瞬间。
- 对于原型阶段和中等负载，这种竞争几乎不可感知。

如果追求极致零锁，未来可以改为 `crossbeam` channel + `BackgroundTask` 的复杂架构。

---

## 6. 典型的 `editor.rs` 结构

```rust
use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, EguiState};
use std::sync::Arc;
use parking_lot::Mutex;

pub struct EditorState {
    pub engine: Arc<Mutex<Option<crate::engine::SynthEngine>>>,
    pub midi_player: Arc<Mutex<crate::engine::MidiPlayer>>,
}

pub fn create(
    params: Arc<crate::YueliangParams>,
    engine: Arc<Mutex<Option<crate::engine::SynthEngine>>>,
    midi_player: Arc<Mutex<crate::engine::MidiPlayer>>,
) -> Option<Box<dyn Editor>> {
    create_egui_editor(
        params.editor_state.clone(),
        EditorState { engine, midi_player },
        |_, _| {},
        move |egui_ctx, _setter, state| {
            egui::CentralPanel::default().show(egui_ctx, |ui| {
                ui.heading("Plugin UI");
                // 绘制控件...
            });
        },
    )
}
```

---

## 参考来源

- `nih-plug` git 仓库：`plugins/examples/gain_gui_egui/src/lib.rs`
- `nih_plug_egui` 源码：`nih_plug_egui/src/lib.rs`、`nih_plug_egui/src/editor.rs`
- `nih-plug` 源码：`src/context/gui.rs`、`src/plugin.rs`
