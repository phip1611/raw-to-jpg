//! Module for the I/O Prefetch Thread.

use crate::thread_pool::{NextWorkloadOrStop};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread::{available_parallelism, spawn, JoinHandle};
use crate::ParallelizationInfo;

/// Raw data of the raw image as it lays in memory. This is expected to be a RAW image
/// format that will be decoded later.
pub type ImageData = Vec<u8>;

/// Creates a synchronized/bounded channel to transport [`ImageData`] from the IO Prefetch Thread
/// to the thread pool dispatch thread. We do not care about the exact order but the sync channel
/// has the advantage that we can prevent that too many items are preloaded into memory which would
/// highly increase memory usage. Instead, at maximum one image per CPU/worker thread will be kept
/// in memory.
///
/// Background: The I/O Prefetch thread will be blocked when the channel is full.
///
/// Thus, we can decouple I/O workloads from CPU-intensive workloads.
pub fn create_channel(
    info: &ParallelizationInfo,
) -> (
    Arc<SyncSender<NextWorkloadOrStop<ImageData>>>,
    Arc<Mutex<Receiver<NextWorkloadOrStop<ImageData>>>>,
) {
    let (sender, receiver) = sync_channel(info.worker_count());
    (Arc::new(sender), Arc::new(Mutex::new(receiver)))
}

pub struct IoPrefetchThread {
    handle: JoinHandle<()>,
}

impl IoPrefetchThread {
    pub fn new(
        files: Vec<PathBuf>,
        sender: Arc<SyncSender<NextWorkloadOrStop<ImageData>>>,
    ) -> Self {
        let handle = spawn(move || {
            files
                .iter()
                .map(|path| read_file(path))
                .map(|data| NextWorkloadOrStop::Workload(data))
                // because the sender is a bounded/sync channel, we can ensure that there
                // are never all images in memory (which would highly increase memory footprint)
                .enumerate()
                .for_each(|(index, data)| {
                    // todo currently this can't be stopped with CTRL+C
                    eprintln!("dispatched file {}", index + 1);
                    sender.send(data).unwrap()
                });

            eprintln!("IO Prefetch Thread Done");
            // This stop will be sent through the channel as last action.
            // It will signalize the thread pool to stop all worker threads
            // because there will be no more workloads.
            sender.send(NextWorkloadOrStop::Stop).unwrap();
        });
        Self { handle }
    }

    /// Gracefully stops the thread.
    pub fn gracefully_stop(self) {
        // this is fine because
        self.handle.join().unwrap()
    }
}

fn read_file(path: &PathBuf) -> Vec<u8> {
    let mut file = File::open(path).unwrap();
    // according to my experience, this is a typical size for a Sony RAW File
    const TWENTY_MB: usize = 1024 * 1024 * 20;
    let mut data = Vec::with_capacity(TWENTY_MB);
    file.read_to_end(&mut data).unwrap();
    data
}
