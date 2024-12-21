use crate::staging_buffer::StagingBuffer;
use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};

lazy_static! {
    pub static ref LOGGER: Arc<Logger> = {
        let logger = Arc::new(Logger::new());

        let logger_cloned = logger.clone();
        std::thread::spawn(move || {
            logger_cloned.compress_thread_main();
        });

        logger
    };
}

thread_local! {
    pub static STAGING_BUFFER: Arc<StagingBuffer> = {
        let staging_buffer = Arc::new(StagingBuffer::new(std::thread::current().id()));
        LOGGER.append_log_buffer(staging_buffer.clone());
        staging_buffer
    };
}

pub struct Logger {
    thread_buffer: Mutex<Vec<Arc<StagingBuffer>>>,
}

impl Logger {
    pub fn new() -> Self {
        Logger {
            thread_buffer: Mutex::new(vec![]),
        }
    }

    pub fn append_log_buffer(&self, buffer: Arc<StagingBuffer>) {
        self.thread_buffer.lock().unwrap().push(buffer);
    }

    pub fn get_storage(&self) -> *mut u8 {
        STAGING_BUFFER.with(|staging_buffer| staging_buffer.get_storage_ptr())
    }

    pub fn reserve_alloc(&self, n_bytes: usize) -> usize {
        STAGING_BUFFER.with(|staging_buffer| staging_buffer.reserve_producer_space(n_bytes))
    }

    pub fn finish_alloc(&self, n_bytes: usize) {
        STAGING_BUFFER.with(|staging_buffer| {
            staging_buffer.finish_reservation(n_bytes);
        });
    }

    fn compress_thread_main(&self) {
        let mut last_staging_buffer_checked: usize = 0;

        loop {
            let mut i = last_staging_buffer_checked;
            let mut thread_buffer = self.thread_buffer.lock().unwrap();
            while !thread_buffer.is_empty() {
                let staging_buffer = thread_buffer.get(i).unwrap().clone();
                let (peek_pos, peek_bytes) = staging_buffer.peek();

                if peek_bytes > 0 {
                    drop(thread_buffer);

                    unsafe {
                        let raw_ptr = staging_buffer.get_storage_ptr().add(peek_pos);
                        let slice = std::slice::from_raw_parts(raw_ptr, peek_bytes);
                        match std::str::from_utf8(slice) {
                            Ok(s) => println!("{}", s),
                            Err(_) => println!("Invalid UTF-8 data"),
                        }
                    }
                    staging_buffer.consume(peek_bytes);

                    thread_buffer = self.thread_buffer.lock().unwrap();
                } else {
                    //TODO: check whether need delete
                }

                i = (i + 1) % thread_buffer.len();

                if i == last_staging_buffer_checked {
                    break;
                }
            }

            // TODO: Log File IO
        }
    }
}
