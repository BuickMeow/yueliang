use std::sync::Arc;
use crossbeam::channel::{Sender, Receiver};
use arc_swap::ArcSwap;

pub type SampleBuffer = Arc<Vec<f32>>;

#[derive(Clone)]
pub struct SampleEntry {
    pub id: String,
    pub data: Option<SampleBuffer>, // None 表示未加载（on-demand）
    pub sample_rate: u32,
    pub loop_start: usize,
    pub loop_end: usize,
}

pub struct Library {
    pub name: String,
    pub instruments: Vec<InstrumentMeta>,
    pub samples: Vec<SampleEntry>,
    // ... 其他元数据
}

pub type LibraryHandle = Arc<Library>;

/// 全局通过 ArcSwap 存储当前 channel -> library 映射
pub struct ChannelLibraryMap {
    // channel index -> ArcSwap<LibraryHandle> 或 Option<LibraryHandle>
    // audio thread 读取 load() 非阻塞，loader 后台通过 store() 替换
    pub map: Vec<ArcSwap<Option<LibraryHandle>>>,
}

impl ChannelLibraryMap {
    pub fn assign_library_to_channel(&self, channel: usize, lib: Option<LibraryHandle>) {
        self.map[channel].store(Arc::new(lib));
    }
    pub fn get_for_channel_rt(&self, channel: usize) -> Option<LibraryHandle> {
        self.map[channel].load_full()
    }
}

/// Loader 请求与事件
pub enum LoaderCommand {
    LoadLibrary { path: String, reply: Sender<Result<LibraryHandle, String>> },
    LoadSample { sample_id: String, lib: LibraryHandle }, // 后台加载某个 sample
    CancelLoad { /* ... */ },
}

/// 后台 loader 的核心循环（伪）
pub fn loader_thread(rx: Receiver<LoaderCommand>) {
    while let Ok(cmd) = rx.recv() {
        match cmd {
            LoaderCommand::LoadLibrary { path, reply } => {
                // 1) 快速解析元数据（instrument table）
                // 2) 生成 Library struct（samples 的 data = None）
                // 3) 立即 reply 已解析的 LibraryHandle（可用做 preview）
                // 4) 在后台遍历 sample 列表并逐个加载/解码/重采样/放入 cache
            }
            LoaderCommand::LoadSample { sample_id, lib } => {
                // 加载 sample 数据并写入 lib.samples[].data = Some(Arc<Vec<f32>>)
                // 可采用 mmap 或预处理二进制，或者解码到 float32
            }
            _ => {}
        }
    }
}