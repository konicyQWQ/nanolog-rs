# NanoLog-rs

Learn and implement the extremely performant nanosecond scale logging system [NanoLog](https://github.com/PlatformLab/NanoLog?tab=License-1-ov-file#readme) for Rust.

## Learn

1. Every thread has its own `staging_buffer` (Thread-Local SPSC lock-free circular buffer).
2. Log message will write into its thread-local `staging_buffer`.
3. A consumer thread consumes all `staging_buffer`'s log message and output.
4. Compress and Preprocessor?

## WIP

- [x] `staging_buffer` and `thread_local` management
- [ ] Log interface
- [ ] Compress and Preprocessor
- [ ] Write into file?
- [ ] Test and Benchmark
