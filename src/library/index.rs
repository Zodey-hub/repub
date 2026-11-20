use std::collections::HashSet;
use std::fs;
use std::hash::Hasher;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::Context;
use fnv::FnvHasher;
use tiny_database::Database;

use super::Book;

const EPUB_EXTENSION: &str = "epub";
/// The largest integer a JSON number keeps exactly in JavaScript (`Number.MAX_SAFE_INTEGER`), the mask of a book id.
const MAX_SAFE_INTEGER: u64 = (1 << 53) - 1;

/// Rebuilds the database at `database_file` from the EPUB files directly in `library_dir` (not in its subdirectories), and returns it.
///
/// The id of a book is the hash of its file name, so it stays the same across starts; a file that cannot be read, or whose id another file already has, is skipped with a warning.
///
/// # Errors
///
/// Returns an error if the directory cannot be read, or the database cannot be created or written.
pub fn index(library_dir: &Path, database_file: &Path) -> anyhow::Result<Database> {
    let started = Instant::now();

    let epub_files = epub_files(library_dir).with_context(|| {
        format!(
            "cannot read the library directory {}",
            library_dir.display()
        )
    })?;

    log::info!(
        "found {} EPUB files in {}",
        epub_files.len(),
        library_dir.display()
    );

    // the database is rebuilt from the files on every start, the old one is only a cache
    remove_if_exists(database_file)
        .with_context(|| format!("cannot remove the old database {}", database_file.display()))?;

    if let Some(database_dir) = database_file.parent() {
        fs::create_dir_all(database_dir).with_context(|| {
            format!(
                "cannot create the database directory {}",
                database_dir.display()
            )
        })?;
    }

    let database = Database::open(database_file)
        .with_context(|| format!("cannot create the database {}", database_file.display()))?;

    let total = epub_files.len();
    let mut books = Vec::with_capacity(total);
    let mut covers = Vec::new();
    let mut ids = HashSet::with_capacity(total);

    for (position, path) in (1..).zip(&epub_files) {
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();
        let id = book_id(&file_name);

        // a second book with the same id would replace the first one in the database
        if !ids.insert(id) {
            log::warn!("[{position}/{total}] {file_name} has the id {id} of another file, skipped");
            continue;
        }

        let (book, cover) = match Book::from_epub(id, path) {
            Ok(read) => read,
            Err(error) => {
                log::warn!("[{position}/{total}] cannot read {file_name}, skipped: {error:#}");
                continue;
            }
        };

        log::info!(
            "[{position}/{total}] read \"{}\" from {file_name}",
            book.title()
        );

        books.push(book);
        covers.extend(cover);
    }

    // one transaction for the books and one for the covers, a commit per book would sync the file for each
    database
        .insert_many(&books)
        .context("cannot store the books in the database")?;

    database
        .insert_many(&covers)
        .context("cannot store the covers in the database")?;

    log::info!(
        "indexed {} of {total} books ({} skipped, {} with a cover) in {:.2} s",
        books.len(),
        total - books.len(),
        covers.len(),
        started.elapsed().as_secs_f64()
    );

    Ok(database)
}

/// The id of the book in the file `file_name`: its 64-bit FNV-1a hash cut to 53 bits, so that a JavaScript client reads it exactly from JSON.
fn book_id(file_name: &str) -> u64 {
    let mut hasher = FnvHasher::default();
    hasher.write(file_name.as_bytes());
    hasher.finish() & MAX_SAFE_INTEGER
}

/// The EPUB files directly in `library_dir`, sorted by file name without regard to case.
fn epub_files(library_dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut epub_files = Vec::new();

    for entry in fs::read_dir(library_dir)? {
        let path = entry?.path();

        let is_epub = path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case(EPUB_EXTENSION));

        if is_epub && path.is_file() {
            epub_files.push(path);
        }
    }

    epub_files.sort_by_cached_key(|path| path.to_string_lossy().to_lowercase());

    Ok(epub_files)
}

/// Removes the file at `path`, a missing file is not an error.
fn remove_if_exists(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        result => result,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_id_of_a_file_name_never_changes() {
        // the FNV-1a test vectors cut to 53 bits: a change of the hash would break every saved link
        assert_eq!(book_id(""), 0xcbf2_9ce4_8422_2325 & MAX_SAFE_INTEGER);
        assert_eq!(book_id("a"), 0xaf63_dc4c_8601_ec8c & MAX_SAFE_INTEGER);
        assert_eq!(book_id("foobar"), 0x8594_4171_f739_67e8 & MAX_SAFE_INTEGER);
    }
}
