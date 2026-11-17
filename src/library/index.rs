use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::Context;
use tiny_database::Database;

use super::Book;

const EPUB_EXTENSION: &str = "epub";

/// Rebuilds the database at `database_file` from the EPUB files directly in `library_dir` (not in its subdirectories), and returns it.
///
/// The books get the ids `1..` in the order of their file names, a file that cannot be read is skipped with a warning.
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
    let mut indexed = 0;
    let mut covers = 0;

    // the ids start from 1, in the order of the files
    for (id, path) in (1..).zip(&epub_files) {
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();

        let (book, cover) = match Book::from_epub(id, path) {
            Ok(read) => read,
            Err(error) => {
                log::warn!("[{id}/{total}] cannot read {file_name}, skipped: {error:#}");
                continue;
            }
        };

        database
            .insert(&book)
            .with_context(|| format!("cannot store the book {file_name} in the database"))?;

        if let Some(cover) = &cover {
            database.insert(cover).with_context(|| {
                format!("cannot store the cover of {file_name} in the database")
            })?;
            covers += 1;
        }

        indexed += 1;
        log::info!(
            "[{id}/{total}] indexed \"{}\" from {file_name}",
            book.title()
        );
    }

    log::info!(
        "indexed {indexed} of {total} books ({} skipped, {covers} with a cover) in {:.2} s",
        total - indexed,
        started.elapsed().as_secs_f64()
    );

    Ok(database)
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

    // the books are listed in id order, which people expect alphabetical regardless of case
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
