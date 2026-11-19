//! The library: the EPUB files of a directory, indexed into the database on every start.

mod book;
mod cover;
mod index;
pub mod text;

pub use book::{Book, BookIndex};
pub use cover::Cover;
pub use index::index;
