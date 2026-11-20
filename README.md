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
cargo run --release                                                   # 127.0.0.1:8080, books: ./library, log: repub_<version>.log
cargo run --release -- --library ~/books --database data/repub.redb    # its own book directory and database file
cargo run --release -- --listen 0.0.0.0:8080 --log-file logs/repub.log  # on every interface, with its own log file
cargo run --release -- --help
```

- `--listen <ADDR>`: the address the server listens on; `127.0.0.1:8080` by default, reachable only from this machine
- `--library <DIR>`: the directory of the EPUB files; `library` in the working directory by default
- `--database <FILE>`: the database file (missing directories are created); `repub.redb` in the working directory by default
- `--log-file <FILE>`: the log file, appended on every run (missing directories are created); `repub_<version>.log` in the working directory by default; the log also goes to stdout
- An address in use, a missing library directory, a database or log file that cannot be written or an invalid flag stops it with exit code 1

## Library

On every start the database is rebuilt from the EPUB files directly in the library directory (subdirectories are not read), before the server starts listening:

- The id of a book is the 64-bit FNV-1a hash of its file name cut to 53 bits (exact in JavaScript), so a link to a book stays valid across restarts and new files; renaming a file gives its book a new id, and a file whose id another file already has is skipped with a warning
- Each book is indexed by the words of its title, authors, translators, subjects and publisher, its primary language and its year, so a search reads only the books of one index range instead of the whole library
- Each book stores its title (the file name if it has none), authors, language, description (as plain text) and its cover image as it is in the EPUB
- A file that is not a readable EPUB is skipped with a warning; a cover that is not a JPEG, PNG, GIF or WebP (an SVG could run a script) leaves the book without a cover
- The log shows the number of files found, a `[x/y]` line for each file and a summary at the end

## Pages

Every page answers in two forms on the same URL: a browser (its `Accept` lists `text/html`) gets an HTML page, anything else (curl, scripts, monitoring) gets JSON.

- `GET /`: the home page with the search form (a field for each search parameter), `{"name":"repub","description":"..."}`
- `GET /search?title=&author=&translator=&subject=&publisher=&language=&year_from=&year_to=`: the books that match every given field; no book without one. A text field matches ignoring case and accents when each of its words starts a word of that field in the book (any author, translator or subject; `jok` finds `Jókai`, `okai` does not), the language matches its tag or primary subtag (`en` finds `en-US`), the years are inclusive and a book without a year matches no year range; the books are listed by file name, ignoring case; a blank field or a year that is not a number is ignored. The page also shows how many books the index read (`candidates`, before the other fields filtered them) and how long the search took (`elapsed_us`, in microseconds); `stats` is `null` without a field. `{"criteria":{"title":"...","author":null,...,"year_from":1900,"year_to":null},"books":[{"id":1,"file_name":"...","title":"...","authors":["..."],...}],"stats":{"candidates":37,"elapsed_us":412}}`
- `GET /book?id=<id>`: the page of a book with its cover, authors, language, description and a download button; in JSON the book itself; 404 if there is no such book
- `GET /download?id=<id>`: the EPUB file of a book as an attachment under its own file name (`filename*` keeps the accents); 404 if there is no such book
- `GET /cover?id=<id>`: the cover image of a book, cached by the browser for a day; 404 if the book has no cover
- `GET /health`: the state of the server, `{"status":"pass","version":"<version>","books":9}`; 503 with `{"status":"fail","version":"<version>","books":null}` if the database cannot be read, the cause is in the log
- Any other path or method: 404, `{"error":"not_found"}`
- A database error: 500, `{"error":"internal_error"}`, the cause is in the log

The HTML pages are styled with a small stylesheet of their own (light and dark, following the system setting), built into the binary from `assets/style.css` and served at `GET /style.css`.

```sh
curl http://127.0.0.1:8080/health                          # JSON
curl -H 'Accept: text/html' http://127.0.0.1:8080/health   # HTML
```

## License

Licensed under either of

- Apache License, Version 2.0
- MIT license

at your option.
