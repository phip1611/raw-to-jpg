//! CLI Utility to Convert Raw Images of Different Formats to JPEG.

#![deny(
    clippy::all,
    clippy::cargo,
    clippy::nursery,
    // clippy::restriction,
    // clippy::pedantic
)]
// now allow a few rules which are denied by the above statement
// --> they are ridiculous and not necessary
#![allow(
    clippy::suboptimal_flops,
    clippy::redundant_pub_crate,
    clippy::fallible_impl_from
)]
// I can't do anything about this; fault of the dependencies
#![allow(clippy::multiple_crate_versions)]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(rustdoc::all)]
#![allow(rustdoc::missing_doc_code_examples)]

use std::path::PathBuf;
use std::sync::mpsc::sync_channel;
use std::sync::Mutex;
use std::thread::sleep;
use std::time::Duration;
use crate::io_prefetch::IoPrefetchThread;
use crate::thread_pool::ConverterThreadPool;
use crate::util::ParallelizationInfo;

mod thread_pool;
mod io_prefetch;
mod util;

fn main() {
    let files = load_files();
    if files.is_empty() {
        println!("No files found. Exit.");
        return;
    }
    let parallelization_info = ParallelizationInfo::new(files.len());
    let (sender, receiver) = io_prefetch::create_channel(&parallelization_info);
    let io_prefetch_thread = IoPrefetchThread::new(files, sender.clone());
    let mut thread_pool = ConverterThreadPool::new(&parallelization_info);
    thread_pool.wait_until_done_and_stop_gracefully(receiver);
    io_prefetch_thread.gracefully_stop();
}

/// Loads all file names that apply to the given filter (current working directory, name filter,
/// etc.). Only performs a flat search.
fn load_files() -> Vec<PathBuf> {
    vec![
        "src/main.rs".into(),
    ]
}
