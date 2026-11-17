//! The library: the EPUB files of a directory, indexed into the database on every start.

mod book;
mod cover;
mod index;

pub use book::Book;
pub use cover::Cover;
pub use index::index;
