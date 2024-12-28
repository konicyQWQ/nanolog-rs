use std::cell::{Cell, UnsafeCell};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::ThreadId;

pub const STAGING_BUFFER_MAX_SIZE: usize = 1 << 20;
const CACHE_LINE_SIZE: usize = 64;

pub struct StagingBuffer {
    storage: UnsafeCell<Box<[u8; STAGING_BUFFER_MAX_SIZE]>>,

    // alignas buffer
    _cache_line_buffer_consumer: [u8; 2 * CACHE_LINE_SIZE],

    // consumer use
    consumer_pos: AtomicUsize,

    // alignas buffer
    _cache_line_buffer_producer: [u8; 2 * CACHE_LINE_SIZE],

    // producer use
    producer_pos: AtomicUsize,
    end_of_recorded_space: AtomicUsize,
    min_free_space: Cell<usize>,

    // other
    buffer_id: ThreadId,
}

unsafe impl Sync for StagingBuffer {}

impl StagingBuffer {
    pub fn new(id: ThreadId) -> StagingBuffer {
        StagingBuffer {
            storage: UnsafeCell::new(Box::new([0; STAGING_BUFFER_MAX_SIZE])),

            _cache_line_buffer_consumer: [0; 2 * CACHE_LINE_SIZE],

            consumer_pos: AtomicUsize::new(0),

            _cache_line_buffer_producer: [0; 2 * CACHE_LINE_SIZE],

            producer_pos: AtomicUsize::new(0),
            end_of_recorded_space: AtomicUsize::new(STAGING_BUFFER_MAX_SIZE),
            min_free_space: Cell::new(0),

            buffer_id: id,
        }
    }

    pub fn get_storage_ptr(&self) -> *mut u8 {
        self.storage.get().cast()
    }

    pub fn get_id(&self) -> ThreadId {
        self.buffer_id
    }

    #[inline]
    pub fn reserve_producer_space(&self, n_bytes: usize) -> usize {
        if n_bytes < self.min_free_space.get() {
            self.producer_pos.load(Ordering::Relaxed)
        } else {
            self.reserve_space_internal(n_bytes)
        }
    }

    fn reserve_space_internal(&self, n_bytes: usize) -> usize {
        while self.min_free_space.get() <= n_bytes {
            let cached_consumer_pos = self.consumer_pos.load(Ordering::Relaxed);

            if cached_consumer_pos <= self.producer_pos.load(Ordering::Relaxed) {
                self.min_free_space
                    .set(STAGING_BUFFER_MAX_SIZE - self.producer_pos.load(Ordering::Relaxed));

                if self.min_free_space.get() > n_bytes {
                    break;
                }

                self.end_of_recorded_space
                    .store(self.producer_pos.load(Ordering::Relaxed), Ordering::Relaxed);

                if cached_consumer_pos != 0 {
                    //TODO: sfence?
                    self.producer_pos.store(0, Ordering::Relaxed);
                    self.min_free_space.set(cached_consumer_pos);
                }
            } else {
                self.min_free_space
                    .set(cached_consumer_pos - self.producer_pos.load(Ordering::Relaxed));
            }
        }

        self.producer_pos.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn finish_reservation(&self, n_bytes: usize) {
        //TODO: sfence
        self.min_free_space.set(self.min_free_space.get() - n_bytes);
        self.producer_pos.fetch_add(n_bytes, Ordering::Release);
    }

    #[inline]
    pub fn consume(&self, nbytes: usize) {
        //TODO: lfence?
        self.consumer_pos.fetch_add(nbytes, Ordering::Relaxed);
    }

    pub fn peek(&self) -> (usize, usize) {
        let cached_producer_pos = self.producer_pos.load(Ordering::Acquire);

        if cached_producer_pos < self.consumer_pos.load(Ordering::Relaxed) {
            //TODO: lfence
            let byte_available = self.end_of_recorded_space.load(Ordering::Relaxed) as i64
                - self.consumer_pos.load(Ordering::Relaxed) as i64;

            if byte_available > 0 {
                return (self.consumer_pos.load(Ordering::Relaxed), byte_available as usize);
            }

            self.consumer_pos.store(0, Ordering::Relaxed);
        }

        (
            self.consumer_pos.load(Ordering::Relaxed),
            cached_producer_pos - self.consumer_pos.load(Ordering::Relaxed),
        )
    }
}
