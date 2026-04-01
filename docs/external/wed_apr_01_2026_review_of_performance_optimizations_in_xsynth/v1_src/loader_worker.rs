use crossbeam::channel::Receiver;
use rayon::ThreadPoolBuilder;

/// 后台 loader loop（伪示例）
pub fn loader_loop(rx: Receiver<LoaderCmd>) {
    // 固定大小线程池用于并行解码
    let pool = ThreadPoolBuilder::new().num_threads(num_cpus::get()).build().unwrap();

    while let Ok(cmd) = rx.recv() {
        match cmd {
            LoaderCmd::OpenAndIndex { path, reply } => {
                // 快速 parse headers -> build index (轻量)
                // 写入侧车索引文件
                // 构建 Library struct (samples[].data = None)
                // reply.send(Ok(lib_handle))
                // 不在此阻塞加载全部 sample
            }
            LoaderCmd::LoadSample { lib, sample_idx } => {
                // 在线程池里 spawn 解码任务：
                // pool.spawn(move || { decode sample -> Arc<Vec<f32>> -> store into lib.samples[sample_idx].data });
            }
            LoaderCmd::PriorityLoadInstrument { lib, instr_idx } => {
                // 找到该 instrument 的代表 samples，优先提交 LoadSample
            }
        }
    }
}