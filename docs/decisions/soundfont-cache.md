# 决策：音色库跨端口共享缓存

最后更新：2026-04-14-03-49-32

---

## 结论

在 `SynthEngine` 中引入 `HashMap<String, Arc<dyn SoundfontBase>>` 缓存，同一 SF2/SFZ 文件只加载一次，多端口共享引用。

---

## 原因

### 问题

早期实现中，`load_soundfonts_to_port` 每次调用都会重新 `SampleSoundfont::new()` 从磁盘加载音色库。当使用"复制到所有端口"（📑）功能时，同一个文件被重复加载 16 次，导致：
- 加载时间极长（大型 SF2 可达数秒 × 16）
- 内存占用爆炸（16 倍冗余）

### 优点

- **显著减少内存占用**：16 端口共享同一份音色数据
- **复制操作瞬间完成**：不再需要重复磁盘 I/O 和样本解析
- **实现简单**：只需在 `SynthEngine` 中维护一个 `HashMap`

### 缺点

- 音色库文件被修改后需要重启插件才能生效（目前无热重载需求）
- 缓存生命周期与 `SynthEngine` 绑定，插件销毁时自动释放

---

## 被否决方案

### 1. 每个端口只存路径，不存实例

意味着每次播放时按需加载，延迟不可接受。

### 2. 全局单例缓存

引入跨 `SynthEngine` 实例的全局状态，增加生命周期管理复杂度，且不同实例可能使用不同 `AudioStreamParams`，音色库实例并不完全通用。

---

## 实现要点

```rust
pub struct SynthEngine {
    core: ChannelGroup,
    sf_cache: HashMap<String, Arc<dyn SoundfontBase>>,
}

pub fn load_soundfonts_to_port(&mut self, port: usize, paths: &[String]) -> Result<(), String> {
    let mut soundfonts = Vec::new();
    for path in paths {
        if let Some(sf) = self.sf_cache.get(path) {
            soundfonts.push(sf.clone());
        } else {
            let sf = SampleSoundfont::new(path, ...)?;
            let arc = Arc::new(sf) as Arc<dyn SoundfontBase>;
            self.sf_cache.insert(path.clone(), arc.clone());
            soundfonts.push(arc);
        }
    }
    // 发送到对应端口的 16 个通道...
}
```

---

## 适用阶段/版本

v0.0.2 阶段 5，16 端口音色库管理器功能完善期间。
