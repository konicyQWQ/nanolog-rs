mod runtime;
mod staging_buffer;

pub static LOGGER: runtime::Logger = runtime::Logger;

#[cfg(test)]
mod tests {
    use crate::LOGGER;
    use log::{info, Log};
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn log_output() {
        log::set_logger(&LOGGER).expect("set logger error");
        log::set_max_level(log::LevelFilter::Info);

        info!("hello world");
    }
}
