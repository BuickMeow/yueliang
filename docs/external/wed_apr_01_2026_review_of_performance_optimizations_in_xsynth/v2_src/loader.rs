use std::sync::Arc;
use arc_swap::ArcSwapOption;
use crossbeam::channel::{unbounded, Sender, Receiver};
use rayon::ThreadPool;
use std::path::PathBuf;

/// LibraryHandle = 只读、可共享的音色库句柄
pub type LibraryHandle = Arc<Library>;

pub struct Library {
    pub name: String,
    pub instruments: Vec<InstrumentMeta>,
    // samples: metadata + optional data
    pub samples: Vec<SampleEntry>,
}

pub struct SampleEntry {
    pub id: String,
    pub offset: u64,      // 在源文件中的偏移（如果适用）
    pub len: u64,         // 字节长度或样本长度
    pub sample_rate: u32,
    pub loop_start: usize,
    pub loop_end: usize,
    pub data: Option<Arc<Vec<f32>>>, // Some after loaded
}

/// 全局 Channel -> Library 映射（RT 安全读取）
pub struct ChannelMap {
    // 每个通道一个 ArcSwapOption<LibraryHandle>
    pubs: Vec<ArcSwapOption<LibraryHandle>>,
}

impl ChannelMap {
    pub fn new(n: usize) -> Self {
        Self { pubs: (0..n).map(|_| ArcSwapOption::empty()).collect() }
    }
    pub fn assign(&self, channel: usize, lib: Option<LibraryHandle>) {
        self.pubs[channel].store(lib);
    }
    pub fn get_rt(&self, channel: usize) -> Option<LibraryHandle> {
        self.pubs[channel].load_full()
    }
}

/// Loader 命令
pub enum LoaderCmd {
    OpenAndIndex { path: PathBuf, reply: Sender<Result<LibraryHandle,String>> },
    LoadSample { lib: LibraryHandle, sample_idx: usize },
    PriorityLoadInstrument { lib: LibraryHandle, instr_idx: usize },
}