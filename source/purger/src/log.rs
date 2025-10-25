use std::sync::OnceLock;

use crate::{config, constants};

pub static FILE_STREAM: OnceLock<
    std::sync::Mutex<file_rotate::FileRotate<file_rotate::suffix::AppendTimestamp>>,
> = OnceLock::new();

#[macro_export]
macro_rules! outputln {
    () => {};
    ($($arg:tt)*) => {{
        use std::io::Write;
        use $crate::log::FILE_STREAM;

        let timestamp = chrono::Local::now().timestamp_millis();
        let timestamp = chrono::DateTime::from_timestamp_millis(timestamp).unwrap();
        let timestamp = timestamp.to_rfc3339();
        let message = format!($($arg)*);
        let log_line = timestamp + " MSG " + file!() + ":" + &line!().to_string() + " :: " + &message + "\n";
        let log_line = log_line.as_bytes();

        let _ = std::io::stdout().write(log_line);

        if let Some(stream) = FILE_STREAM.get() {
            if let Ok(mut stream) = stream.lock() {
                let _ = stream.write(log_line);
            }
        }
    }};
}

#[macro_export]
macro_rules! errorln {
    () => {};
    ($($arg:tt)*) => {{
        use std::io::Write;
        use $crate::log::FILE_STREAM;

        let timestamp = chrono::Local::now().timestamp_millis();
        let timestamp = chrono::DateTime::from_timestamp_millis(timestamp).unwrap();
        let timestamp = timestamp.to_rfc3339();
        let message = format!($($arg)*);
        let log_line = timestamp + " ERR " + file!() + ":" + &line!().to_string() + " :: " + &message + "\n";
        let log_line = log_line.as_bytes();

        let _ = std::io::stdout().write(log_line);

        if let Some(stream) = FILE_STREAM.get() {
            if let Ok(mut stream) = stream.lock() {
                let _ = stream.write(log_line);
            }
        }
    }};
}

fn initialize_app_log() {
    let config = config::object();

    let file = file_rotate::FileRotate::new(
        constants::LOG_FILENAME,
        file_rotate::suffix::AppendTimestamp::default(file_rotate::suffix::FileLimit::MaxFiles(
            config.log_days_retention,
        )),
        file_rotate::ContentLimit::Time(file_rotate::TimeFrequency::Daily),
        file_rotate::compression::Compression::None,
        None,
    );

    if FILE_STREAM.set(std::sync::Mutex::new(file)).is_err() {
        panic!("log has been already initialized");
    }
}

pub fn initialize() {
    initialize_app_log();
}
