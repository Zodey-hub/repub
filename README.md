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

Every page answers in two forms on the same URL: a browser (its `Accept` lists `text/html`) gets an HTML page, anything else (curl, scripts, monitoring) gets JSON.

- `GET /`: the home page, `{"name":"repub","description":"..."}`
- `GET /health`: the state of the server, `{"status":"pass","version":"<version>"}`
- Any other path or method: 404, `{"error":"not_found"}`

The HTML pages are styled with [Simple.css](https://simplecss.org) v2.3.7 (MIT), a classless stylesheet built into the binary from `assets/simple.min.css` and served at `GET /style.css`.

```sh
curl http://127.0.0.1:8080/health                          # JSON
curl -H 'Accept: text/html' http://127.0.0.1:8080/health   # HTML
```

## License

Licensed under either of

- Apache License, Version 2.0
- MIT license

at your option.
