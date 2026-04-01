// 简单 sidecar metadata 定义（建议放到 yueliang 的 data 目录）
use serde::{Serialize, Deserialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct SidecarIndex {
    pub file_path: PathBuf,
    pub file_size: u64,
    pub file_mtime: u64, // unix timestamp
    pub file_hash: Option<String>, // 可选的 hash 用于更强校验
    pub presets: Vec<IndexPreset>,
    pub samples: Vec<IndexSample>,
}

#[derive(Serialize, Deserialize)]
pub struct IndexPreset {
    pub bank: u16,
    pub preset: u16,
    pub name: String,
    pub regions: Vec<usize>, // 指向 samples 索引列表
}

#[derive(Serialize, Deserialize)]
pub struct IndexSample {
    pub id: usize,
    pub offset: u64,   // 字节偏移（或样本偏移）
    pub byte_len: u64, // 字节数
    pub sample_rate: u32,
    pub bits: u16,
    pub channels: u8,
    pub loop_start: u64,
    pub loop_end: u64,
    pub root_key: i8,
}