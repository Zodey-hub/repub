use std::path::Path;

use epub::doc::EpubDoc;
use serde::Serialize;
use tiny_database::{NoIndex, Record, rkyv};

use super::Cover;

/// A book of the library: the metadata of one EPUB file.
#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
#[rkyv(crate = tiny_database::rkyv)]
pub struct Book {
    id: u64,
    file_name: String,
    title: String,
    authors: Vec<String>,
    language: Option<String>,
    has_cover: bool,
}

impl Book {
    /// Reads the book `id` from the EPUB file at `path`, with its cover if it has one.
    ///
    /// # Errors
    ///
    /// Returns an error if the file is not a readable EPUB; a cover of another type than JPEG, PNG, GIF or WebP only leaves the book without a cover.
    pub fn from_epub(id: u64, path: &Path) -> anyhow::Result<(Self, Option<Cover>)> {
        let mut epub = EpubDoc::new(path)?;

        let file_name = path.file_name().unwrap_or_default().to_string_lossy();
        let file_stem = path.file_stem().unwrap_or_default().to_string_lossy();

        // a book without a title is still listed, under the name of its file
        let title = epub
            .get_title()
            .map_or_else(|| file_stem.into_owned(), |title| title.trim().to_owned());

        let authors = epub
            .metadata
            .iter()
            .filter(|item| item.property == "creator")
            .map(|item| item.value.trim().to_owned())
            .collect();

        let language = epub.mdata("language").map(|item| item.value.clone());

        let cover = epub
            .get_cover()
            .and_then(|(img, mime)| Cover::new(id, img, &mime));

        let book = Self {
            id,
            file_name: file_name.into_owned(),
            title,
            authors,
            language,
            has_cover: cover.is_some(),
        };

        Ok((book, cover))
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn authors(&self) -> &[String] {
        &self.authors
    }

    pub fn language(&self) -> Option<&str> {
        self.language.as_deref()
    }

    pub fn has_cover(&self) -> bool {
        self.has_cover
    }
}

impl Record for Book {
    const TABLE: &'static str = "books";
    type Index = NoIndex;

    fn id(&self) -> u64 {
        self.id
    }

    fn index_keys(&self) -> Vec<(NoIndex, Vec<u8>)> {
        Vec::new()
    }
}
