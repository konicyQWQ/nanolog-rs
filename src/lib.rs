pub mod runtime;
mod staging_buffer;

#[cfg(test)]
mod tests {
    use std::ops::Add;
    use std::thread::sleep;
    use std::time::Duration;
    use crate::runtime::LOGGER;

    #[test]
    fn log_output() {
        let string = "1234";

        let pos = LOGGER.reserve_alloc(string.len());
        let storage = LOGGER.get_storage();
        unsafe {
            std::ptr::copy_nonoverlapping(string.as_ptr(), storage.add(pos), string.len());
        }
        LOGGER.finish_alloc(string.len());

        sleep(Duration::from_secs(1));
    }
}
