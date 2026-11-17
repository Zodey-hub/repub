use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::path::PathBuf;
use std::process;

/// The text of `--help`.
const HELP: &str = "\
repub

USAGE:
  repub [OPTIONS]

HELP:
  -h, --help            Prints this help information.

OPTIONAL FLAGS:
  --listen <ADDR>       Sets the address the server listens on (default: 127.0.0.1:8080).
  --library <DIR>       Sets the directory of the EPUB files, indexed on every start (default: library in the working directory).
  --database <FILE>     Sets the database file, rebuilt on every start (default: repub.redb in the working directory).
  --log-file <FILE>     Sets the log file, appended on every run (default: repub_<version>.log in the working directory).
";

/// The default address: reachable only from this machine.
const DEFAULT_LISTEN: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8080));
/// The default directory of the EPUB files, in the working directory.
const DEFAULT_LIBRARY: &str = "library";
/// The default database file, in the working directory.
const DEFAULT_DATABASE: &str = concat!(env!("CARGO_PKG_NAME"), ".redb");
/// The default log file in the working directory: `<package name>_<version>.log`.
const DEFAULT_LOG_FILE: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "_",
    env!("CARGO_PKG_VERSION"),
    ".log"
);

/// The command line: only how the server runs.
pub struct Arguments {
    pub listen: SocketAddr,
    pub library: PathBuf,
    pub database: PathBuf,
    pub log_file: PathBuf,
}

impl Arguments {
    /// Reads the command line; on `--help` prints the help, on an invalid flag prints the error and the help, and exits.
    pub fn from_env() -> Self {
        let mut args = pico_args::Arguments::from_env();

        if args.contains(["-h", "--help"]) {
            print!("{HELP}");
            process::exit(0);
        }

        match Self::parse_from(&mut args) {
            Ok(arguments) => arguments,
            Err(error) => {
                eprintln!("{error}\n{HELP}");
                process::exit(1);
            }
        }
    }

    fn parse_from(args: &mut pico_args::Arguments) -> Result<Self, pico_args::Error> {
        let listen: Option<SocketAddr> = args.opt_value_from_str("--listen")?;
        let library: Option<PathBuf> = args.opt_value_from_str("--library")?;
        let database: Option<PathBuf> = args.opt_value_from_str("--database")?;
        let log_file: Option<PathBuf> = args.opt_value_from_str("--log-file")?;

        Ok(Self {
            listen: listen.unwrap_or(DEFAULT_LISTEN),
            library: library.unwrap_or_else(|| PathBuf::from(DEFAULT_LIBRARY)),
            database: database.unwrap_or_else(|| PathBuf::from(DEFAULT_DATABASE)),
            log_file: log_file.unwrap_or_else(|| PathBuf::from(DEFAULT_LOG_FILE)),
        })
    }
}
