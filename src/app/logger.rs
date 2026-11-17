//! The logger of the server: the lines of the `log` macros go to stdout and to the log file, as `[<timestamp> <file:line> <level>] <message>`.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use anyhow::Context;
use log::{LevelFilter, Log, Metadata, Record};

/// Sets up logging up to `info`: the `log` macros write to the end of `log_file` (the log of earlier runs is kept) and to stdout.
pub fn init_logging(log_file: &Path) -> anyhow::Result<()> {
    if let Some(log_dir) = log_file.parent() {
        fs::create_dir_all(log_dir)
            .with_context(|| format!("cannot create the log directory {}", log_dir.display()))?;
    }

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file)
        .with_context(|| format!("cannot open the log file {}", log_file.display()))?;

    let logger = Logger { file };
    log::set_boxed_logger(Box::new(logger))?;
    log::set_max_level(LevelFilter::Info);

    Ok(())
}

struct Logger {
    file: File,
}

impl Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let timestamp = jiff::ZonedDateTime::now().strftime("%Y-%m-%dT%H:%M:%S%.3f%:z");

        let location = match (record.file(), record.line()) {
            (Some(file), Some(line_number)) => format!("{file}:{line_number}"),
            _ => record.target().to_owned(),
        };

        let line = format!(
            "[{} {} {}] {}\n",
            timestamp,
            location,
            record.level(),
            record.args()
        );

        // the logger has nowhere to report its own write error
        let _ = (&self.file).write_all(line.as_bytes());
        let _ = io::stdout().write_all(line.as_bytes());
    }

    fn flush(&self) {
        // the logger has nowhere to report its own write error
        let _ = (&self.file).flush();
    }
}
