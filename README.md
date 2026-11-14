# repub

A lightweight, server-rendered website for searching and downloading EPUB books.

## Status

Early development. The project is a work in progress.

## Build

```sh
cargo build --release
```

## Run

```sh
cargo run --release                                                   # 127.0.0.1:8080, log: repub_<version>.log
cargo run --release -- --listen 0.0.0.0:8080 --log-file logs/repub.log  # on every interface, with its own log file
cargo run --release -- --help
```

- `--listen <ADDR>`: the address the server listens on; `127.0.0.1:8080` by default, reachable only from this machine
- `--log-file <FILE>`: the log file, appended on every run (missing directories are created); `repub_<version>.log` in the working directory by default; the log also goes to stderr
- An address in use, a log file that cannot be written or an invalid flag stops it with exit code 1

## Pages

- `GET /`: the home page
- `GET /health`: the running version as plain text
- Any other path or method: the not found page (404)

## License

Licensed under either of

- Apache License, Version 2.0
- MIT license

at your option.
