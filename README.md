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
cargo run --release -- --library ~/books
cargo run --release -- --help
```

On every start the database is rebuilt from the EPUB files in the library directory.

## Pages

Every page answers with HTML to a browser and with JSON to anything else (curl, scripts).

- `GET /`: the search form
- `GET /search?title=&author=&translator=&subject=&publisher=&language=&year_from=&year_to=`: the matching books
- `GET /book?id=<id>`: the page of a book
- `GET /cover?id=<id>`: the cover image of a book
- `GET /download?id=<id>`: the EPUB file of a book
- `GET /health`: the state of the server

## License

Licensed under either of

- Apache License, Version 2.0
- MIT license

at your option.
