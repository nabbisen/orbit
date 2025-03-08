use std::thread::available_parallelism;

pub fn _logical_cpu_num() -> usize {
    let cores = match available_parallelism() {
        Ok(x) => x.get(),
        // fallback
        _ => 1,
    };
    cores
}
