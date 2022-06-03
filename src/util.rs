use std::cmp::min;
use std::thread::available_parallelism;

#[derive(Debug)]
pub struct ParallelizationInfo  {
    workers: usize
}

impl ParallelizationInfo {
    pub fn new(file_count: usize) -> Self {
        let workers = calculate_worker_thread_count(file_count);
        Self {
            workers
        }
    }

    pub fn worker_count(&self) -> usize {
        self.workers
    }
}

/// Returns how many worker threads will be available for the CPU-intensive
/// JPEG encoding workload.
fn calculate_worker_thread_count(file_count: usize) -> usize {
    let cpus = available_parallelism().map(|x| x.into()).unwrap_or(1);
    // we do not need more CPUs than files
    min(cpus, file_count)
}
