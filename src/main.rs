#![doc = include_str!("../README.md")]

mod app;
mod files;
mod library;
mod pages;
mod reply;
mod router;

use std::process::ExitCode;

use app::Arguments;

fn main() -> ExitCode {
    let arguments = Arguments::from_env();

    // an error before the logger is set can only go to stderr
    if let Err(error) = app::init_logging(&arguments.log_file) {
        eprintln!("{error:#}");
        return ExitCode::FAILURE;
    }

    match app::run(&arguments) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            log::error!("{error:#}");
            ExitCode::FAILURE
        }
    }
}
