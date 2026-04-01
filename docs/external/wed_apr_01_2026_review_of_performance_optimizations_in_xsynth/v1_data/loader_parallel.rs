// 伪代码：并行解码样本 -> 存入 cache -> publish library
use rayon::prelude::*;
use std::sync::Arc;
use std::fs::File;
use memmap2::MmapOptions;
use crate::data::sidecar::SidecarIndex;
use crate::data::cache::SampleCache; // 你自己实现的 cache

pub fn load_sf2_with_sidecar(path: &std::path::Path, sidecar: &SidecarIndex, cache: &mut SampleCache) {
    // 并行遍历 sidecar.samples，或者优先级加载少量 samples
    sidecar.samples.par_iter().for_each(|s| {
        // 1. mmap 文件区域或读取 chunk
        let mut f = File::open(&sidecar.file_path).unwrap();
        let mmap = unsafe { MmapOptions::new().map(&f).unwrap() };
        let start = s.offset as usize;
        let end = (s.offset + s.byte_len) as usize;
        let bytes = &mmap[start..end];

        // 2. 解码：根据 bits/channels 做转 float32、重采样等（这里是 CPU-bound）
        let samples_f32 = decode_and_convert(bytes, s.sample_rate, s.bits, s.channels);

        // 3. 放入缓存（Arc）
        let arc_buf = Arc::new(samples_f32);
        cache.insert_sample(s.id, arc_buf);
    });

    // 4. 构造 library/preset metadata（轻量）并 publish（arc-swap）
}