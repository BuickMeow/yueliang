# 模块结构

## 文件组织

```
src/
├── lib.rs              # 插件主入口
├── editor.rs           # UI 界面
├── engine.rs           # 音频引擎声明
├── engine/
│   ├── synth.rs        # xsynth 封装
│   └── midi_player.rs  # MIDI 时间线调度器
├── data.rs             # 数据声明
├── data/
│   ├── state.rs        # 共享状态（路由表等）
│   └── loader.rs       # 后台加载器
└── utils.rs            # 工具声明
    └── ring_buffer.rs  # 无锁通信结构
```

---

## 模块职责

### lib.rs
- 插件主入口
- 实现 `Plugin` trait
- 定义 `Yueliang` 与 `YueliangParams`

### editor
- 基于 `nih_plug_egui` 的 UI 实现
- 功能：
  - 文件选择器（MIDI、SF2）
  - 路由矩阵（MIDI 通道 → VST 输出）
  - 参数显示

### engine
音频处理核心，**绝对不能分配内存**。

**engine/synth.rs**
- 封装 `xsynth-core` 的渲染逻辑
- 参数应用（插值算法、复音数等）

**engine/midi_player.rs**
- 与 DAW Transport 严格同步
- 管理内部 MIDI 事件队列
- 时间戳转换（samples ↔ ticks）

### data
数据管理，与音频线程分离。

**data/state.rs**
- 路由配置表
- 使用 `arc-swap` 实现无锁热替换

**data/loader.rs**
- 后台线程异步加载
- MIDI 文件解析（`midly`）
- SoundFont 加载（`xsynth-soundfonts`）

### utils
通用工具。

**utils/ring_buffer.rs**
- 基于 `crossbeam-queue` 的无锁队列
- 用于 GUI → Audio 线程的单向通信

---

## 线程模型

```
┌──────────────┐         ┌──────────────┐
│   GUI 线程    │◄───────►│  后台线程    │
│  (editor)    │  arc-swap│  (loader)   │
└──────┬───────┘         └──────────────┘
       │
       │ 无锁队列
       ▼
┌──────────────┐
│  音频线程    │ ◄── 绝对不能分配内存
│  (process)   │ ◄── 绝对不能加锁
└──────────────┘
```

---

## 模块依赖关系

```
lib.rs
 ├── editor.rs ───────► data/state (读取路由表)
 │
 ├── engine.rs
 │   ├── synth.rs ────► xsynth-core
 │   └── midi_player ─┐
 │                    │
 ├── data.rs          │
 │   ├── state.rs ◄───┘
 │   └── loader.rs ───► midly, xsynth-soundfonts
 │
 └── utils.rs
     └── ring_buffer.rs ──► crossbeam-queue
```
