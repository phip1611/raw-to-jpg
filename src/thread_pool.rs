use crate::io_prefetch::ImageData;
use crate::ParallelizationInfo;
use std::cmp::min;
use std::sync::mpsc::{channel, sync_channel, Receiver, Sender, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::{available_parallelism, sleep, JoinHandle};
use std::time::Duration;

/// Information transferable to threads that either contains the next workload or indicates a stop.
pub enum NextWorkloadOrStop<T: Send> {
    /// Next workload.
    Workload(T),
    /// End of work. This will lead to the termination of worker threads.
    Stop,
}

struct ConvertWorkerThread {
    id: usize,
    handle: JoinHandle<()>,
}

impl ConvertWorkerThread {
    /// Creates a new thread.
    fn new(id: usize, task_receiver: Arc<Mutex<Receiver<NextWorkloadOrStop<ImageData>>>>) -> Self {
        let handle = thread::spawn(move || {
            eprintln!("Thread {} started", { id });
            loop {
                let task = task_receiver.lock().unwrap().recv().unwrap();
                if matches!(task, NextWorkloadOrStop::Stop) {
                    eprintln!("Thread {} stopped", { id });
                    break;
                } else {
                    // Decode Raw
                    sleep(Duration::from_secs(1));
                    // Encode JPEG
                }
            }
        });
        Self { id, handle }
    }
}

pub struct ConverterThreadPool {
    workers: Vec<ConvertWorkerThread>,
    /// The thread pool has a dedicated channel to its workers.
    sender: SyncSender<NextWorkloadOrStop<ImageData>>,
}

impl ConverterThreadPool {
    pub fn new(info: &ParallelizationInfo) -> Self {
        let (sender, receiver) = sync_channel(info.worker_count());
        let receiver = Arc::new(Mutex::new(receiver));
        let workers = (0..info.worker_count())
            .map(move |id| ConvertWorkerThread::new(id, receiver.clone()))
            .collect::<Vec<_>>();

        Self { workers, sender }
    }

    pub fn wait_until_done_and_stop_gracefully(
        self,
        io_dispatch_receiver: Arc<Mutex<Receiver<NextWorkloadOrStop<ImageData>>>>,
    ) {
        loop {
            let task = io_dispatch_receiver.lock().unwrap().recv().unwrap();
            if matches!(task, NextWorkloadOrStop::Stop) {
                break;
            } else {
                // IT IS IMPORTANT that this channel is also a synchronized channel.
                // Otherwise, the advantages of the synchronized channel from the IO Prefetch Thread
                // to the thread of the Thread pool go away.
                self.sender.send(task).unwrap();
            }
        }
        for worker in &self.workers {
            eprintln!("Stopping Thread {}", worker.id);
            self.sender.send(NextWorkloadOrStop::Stop).unwrap();
        }
        for thread in self.workers {
            thread.handle.join().unwrap();
        }
    }
}
