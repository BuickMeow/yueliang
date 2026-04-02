# 调试技巧与踩坑记录

最后更新：2026-04-02  
当前阶段：v0.0.1 - 阶段 3 完成

---

## 1. VST3 插件无声音排查流程

### 1.1 音频通路验证

**问题**：DAW 没有电平表跳动，完全无声。

**排查步骤**：
1. 先用 `TestToneGenerator` 正弦波验证音频通路是否正常
2. 正弦波有声 → 问题在 XSynth 引擎
3. 正弦波无声 → 检查 DAW 播放状态和插件加载

### 1.2 XSynth 引擎排查

**关键发现**：
```
process() 日志输出：
- num_frames: 512
- Sample sum after render: 0  ← 关键指标！
```

如果 `Sample sum` 为 0，说明 XSynth 引擎没有输出音频，需要检查：

1. **SoundFont 是否加载成功**
   - 使用文件日志记录加载结果
   - 检查文件路径（VST3 中相对路径可能失效）

2. **MIDI 事件是否发送**
   - 使用 `active_voices()` 监控活跃 voice 数
   - 刚开始为 1 然后变 0 → NoteOn 已触发但播放完毕

3. **音色库是否正确关联**
   - 通过 `SetSoundfonts` 事件发送
   - 需要使用 `Arc<dyn SoundfontBase>` 包装

---

## 2. macOS 代码签名问题

### 2.1 症状

加载插件时 DAW 闪退，错误日志显示：
```
Exception Type: EXC_BAD_ACCESS (SIGKILL (Code Signature Invalid))
Termination Reason: Namespace CODESIGNING, Code 2, Invalid Page
```

### 2.2 解决方案

对 VST3 插件进行临时签名：

```bash
codesign --force --sign - --deep ~/Library/Audio/Plug-Ins/VST3/Yueliang.vst3
```

### 2.3 自动化

在 `deploy.sh` 中添加签名步骤：

```bash
echo "🔐 签名插件..."
codesign --force --sign - --deep "$VST3_DIR/$PLUGIN_NAME.vst3"
echo "✅ 签名完成"
```

---

## 3. 日志调试技巧

### 3.1 为什么 nih_log! 看不到？

- `nih_log!` 输出到系统日志，但在某些 DAW 中不可见
- `eprintln!` 输出到 stderr，也可能被 DAW 捕获
- 最可靠的方法：**写入文件**

### 3.2 文件日志实现

```rust
use std::fs::OpenOptions;
use std::io::Write;

fn initialize(...) -> bool {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/yueliang_debug.log")
        .unwrap();
    
    writeln!(file, "=== initialize START ===").unwrap();
    
    // ... 代码 ...
    
    writeln!(file, "SoundFont loaded: {:?}", result).unwrap();
    writeln!(file, "=== initialize END ===").unwrap();
    
    true
}
```

查看日志：
```bash
tail -f /tmp/yueliang_debug.log
```

### 3.3 关键监控指标

| 指标 | 正常值 | 异常值 | 含义 |
|------|--------|--------|------|
| `Sample sum after render` | > 0 | 0 | XSynth 是否输出音频 |
| `active_voices()` | > 0 (播放中) | 0 | 是否有活跃音符 |
| `is_soundfont_loaded()` | true | false | 音色库是否加载 |

---

## 4. 常见 Bug 与修复

### 4.1 self.engine 赋值顺序

**错误代码**：
```rust
if let Some(ref mut engine) = self.engine {  // ❌ 此时还是 None
    engine.send_test_note();
}
self.engine = Some(engine);  // 赋值在后面
```

**正确代码**：
```rust
engine.send_test_note();  // 使用局部变量
self.engine = Some(engine);  // 最后才赋值
```

### 4.2 SampleSoundfont 参数顺序

**错误**：
```rust
SampleSoundfont::new(path, Default::default(), stream_params)  // ❌
```

**正确**：
```rust
SampleSoundfont::new(path, stream_params, SoundfontInitOptions::default())  // ✅
```

### 4.3 相对路径在 VST3 中失效

VST3 插件的工作目录不是项目根目录，必须使用**绝对路径**：

```rust
// ❌ 错误：相对路径
let path = "assets/GeneralUser-GS.sf2";

// ✅ 正确：绝对路径
let path = "/Users/jieneng/Documents/GitHub/yueliang/assets/GeneralUser-GS.sf2";
```

---

## 5. 阶段 3 关键里程碑

### 5.1 成功验证清单

- [x] SoundFont 加载成功（GeneralUser-GS.sf2）
- [x] MIDI NoteOn 事件发送成功
- [x] XSynth 引擎输出音频（Sample sum > 0）
- [x] Active Voices 从 1 变 0（音符正常播放并衰减）
- [x] DAW 中可听到声音

### 5.2 技术要点

1. **SoundFont 加载**：通过 `ChannelConfigEvent::SetSoundfonts` 发送
2. **MIDI 事件发送**：使用 `SynthEvent::Channel(channel, ChannelEvent::Audio(...))`
3. **音频渲染**：`ChannelGroup::read_samples()` 输出交错采样 [L, R, L, R, ...]
4. **实时安全**：所有操作都在 `initialize()` 完成，`process()` 只读取

---

## 6. 下一步（阶段 4）

- [ ] 内部走带同步（读取 DAW 播放位置）
- [ ] MIDI 文件加载（midly）
- [ ] 力度过滤与预处理
- [ ] 预过滤与批量发送优化
