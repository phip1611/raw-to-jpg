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

use crate::io_prefetch::IoPrefetchThread;
use crate::thread_pool::ConverterThreadPool;
use crate::util::ParallelizationInfo;
use minifb::{Key, Window, WindowOptions};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::mpsc::sync_channel;
use std::sync::Mutex;
use std::thread::sleep;
use std::time::Duration;

mod io_prefetch;
mod thread_pool;
mod util;
mod raw_image_util;

const WIDTH: usize = 640;
const HEIGHT: usize = 360;

fn main() {
    let files = load_files();
    if files.is_empty() {
        println!("No files found. Exit.");
        return;
    }


    /*let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for i in buffer.iter_mut() {
            *i = 0; // write something more funny here!
        }

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }*/

    /*let parallelization_info = ParallelizationInfo::new(files.len());
    let (sender, receiver) = io_prefetch::create_channel(&parallelization_info);
    let io_prefetch_thread = IoPrefetchThread::new(files, sender.clone());
    let mut thread_pool = ConverterThreadPool::new(&parallelization_info);
    thread_pool.wait_until_done_and_stop_gracefully(receiver);
    io_prefetch_thread.gracefully_stop();*/
}

/// Loads all file names that apply to the given filter (current working directory, name filter,
/// etc.). Only performs a flat search.
fn load_files() -> Vec<PathBuf> {
    vec!["src/main.rs".into()]
}
