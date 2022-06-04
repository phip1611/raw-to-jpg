use std::cmp::min;
use std::io::Read;
use std::thread::available_parallelism;

#[derive(Debug)]
pub struct ParallelizationInfo {
    workers: usize,
}

impl ParallelizationInfo {
    pub fn new(file_count: usize) -> Self {
        let workers = calculate_worker_thread_count(file_count);
        Self { workers }
    }

    pub fn worker_count(&self) -> usize {
        self.workers
    }
}

/// Wrapper around `&[u8]` that implements `Read`.
pub struct ReadableByteSlice<'a> {
    data: &'a [u8],
    read_i: usize,
}

impl<'a> ReadableByteSlice<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, read_i: 0 }
    }
}

impl<'a> Read for ReadableByteSlice<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let bytes_to_read_left = self.data.len() - self.read_i;
        let bytes_to_read = min(bytes_to_read_left, buf.len());

        for (write_i, byte) in self
            .data
            .iter()
            .skip(self.read_i)
            .take(bytes_to_read)
            .enumerate()
        {
            buf[write_i] = *byte;
        }

        self.read_i += bytes_to_read;

        Ok(bytes_to_read)
    }
}

/// Returns how many worker threads will be available for the CPU-intensive
/// JPEG encoding workload.
fn calculate_worker_thread_count(file_count: usize) -> usize {
    let cpus = available_parallelism().map(|x| x.into()).unwrap_or(1);
    // we do not need more CPUs than files
    min(cpus, file_count)
}

#[cfg(test)]
mod tests {
    use crate::util::ReadableByteSlice;
    use std::io::Read;

    #[test]
    fn test_readable_byte_slice() {
        let src = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let mut buf = [0; 3];

        let mut src = ReadableByteSlice::new(&src);

        let bytes = src.read(&mut buf).unwrap();
        assert_eq!(bytes, 3);
        for i in 0..3 {
            assert_eq!(buf[i], i as u8 + 0 * 3);
        }

        let bytes = src.read(&mut buf).unwrap();
        assert_eq!(bytes, 3);
        for i in 0..3 {
            assert_eq!(buf[i], i as u8 + 1 * 3);
        }
        let bytes = src.read(&mut buf).unwrap();
        assert_eq!(bytes, 3);
        for i in 0..3 {
            assert_eq!(buf[i], i as u8 + 2 * 3);
        }
        let bytes = src.read(&mut buf).unwrap();
        assert_eq!(bytes, 1);
        assert_eq!(buf[0], 9);
        let bytes = src.read(&mut buf).unwrap();
        assert_eq!(bytes, 0);
    }
}
