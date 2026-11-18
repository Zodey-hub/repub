use std::path::Path;

use epub::doc::{EpubDoc, MetadataItem};
use serde::Serialize;
use tiny_database::{NoIndex, Record, rkyv};

use super::Cover;

/// The MARC relator role of an author, also the role of a creator without one.
const AUTHOR_ROLE: &str = "aut";
/// The MARC relator role of a translator.
const TRANSLATOR_ROLE: &str = "trl";
/// The year Calibre writes for an unknown date (`0101-01-01`), a time zone can shift it to `0100-12-31`.
const CALIBRE_UNKNOWN_YEAR: u16 = 101;

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
    translators: Vec<String>,
    publisher: Option<String>,
    year: Option<u16>,
    language: Option<String>,
    subjects: Vec<String>,
    description: Option<String>,
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

        let authors = names_with_role(&epub.metadata, AUTHOR_ROLE);
        let translators = names_with_role(&epub.metadata, TRANSLATOR_ROLE);

        let publisher = epub
            .mdata("publisher")
            .map(|item| item.value.trim().to_owned())
            .filter(|publisher| !publisher.is_empty());

        let year = publication_year(&epub.metadata);
        let language = epub.mdata("language").map(|item| item.value.clone());

        let subjects = epub
            .metadata
            .iter()
            .filter(|item| item.property == "subject")
            .map(|item| item.value.trim().to_owned())
            .filter(|subject| !subject.is_empty())
            .collect();

        // the description is often HTML, only its text is kept
        let description = epub
            .mdata("description")
            .map(|item| plain_text(&item.value))
            .filter(|description| !description.is_empty());

        let cover = epub
            .get_cover()
            .and_then(|(img, mime)| Cover::new(id, img, &mime));

        let book = Self {
            id,
            file_name: file_name.into_owned(),
            title,
            authors,
            translators,
            publisher,
            year,
            language,
            subjects,
            description,
            has_cover: cover.is_some(),
        };

        Ok((book, cover))
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn authors(&self) -> &[String] {
        &self.authors
    }

    pub fn translators(&self) -> &[String] {
        &self.translators
    }

    pub fn publisher(&self) -> Option<&str> {
        self.publisher.as_deref()
    }

    pub fn year(&self) -> Option<u16> {
        self.year
    }

    pub fn language(&self) -> Option<&str> {
        self.language.as_deref()
    }

    pub fn subjects(&self) -> &[String] {
        &self.subjects
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
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

/// The names of the `creator` and `contributor` items of `metadata` with the MARC relator `role`.
fn names_with_role(metadata: &[MetadataItem], role: &str) -> Vec<String> {
    metadata
        .iter()
        .filter(|item| relator_role(item) == Some(role))
        .map(|item| item.value.trim().to_owned())
        .collect()
}

/// The MARC relator role of a `creator` or `contributor` item, `None` for any other item; a creator without a role is the author.
fn relator_role(item: &MetadataItem) -> Option<&str> {
    let role = item.refinement("role").map(|role| role.value.trim());

    match item.property.as_str() {
        "creator" => role.or(Some(AUTHOR_ROLE)),
        "contributor" => role,
        _ => None,
    }
}

/// The year of the publication `date` of `metadata`; an EPUB2 file can also hold the dates of other events.
fn publication_year(metadata: &[MetadataItem]) -> Option<u16> {
    let date = metadata
        .iter()
        .filter(|item| item.property == "date")
        .find(|item| {
            item.refinement("event")
                .is_none_or(|event| event.value == "publication")
        })?;

    year_of(&date.value)
}

/// The year an EPUB date starts with (`2015`, `2015-03-01`, `2015-03-01T00:00:00+00:00`), `None` if it has none or it is the unknown date of Calibre.
fn year_of(date: &str) -> Option<u16> {
    let (year, rest) = date.trim().split_at_checked(4)?;
    let has_year_only = rest.is_empty() || rest.starts_with('-');
    let year: u16 = year.parse().ok()?;

    (has_year_only && year > CALIBRE_UNKNOWN_YEAR).then_some(year)
}

/// The text of `html` without its tags and with its whitespace collapsed; its character references stay as they are.
fn plain_text(html: &str) -> String {
    let mut text = String::with_capacity(html.len());
    let mut in_tag = false;

    for character in html.chars() {
        match character {
            // a tag separates words, as `<p>a</p><p>b</p>` shows
            '<' => {
                in_tag = true;
                text.push(' ');
            }
            '>' if in_tag => in_tag = false,
            _ if !in_tag => text.push(character),
            _ => {}
        }
    }

    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_year_is_read_from_every_date_form() {
        for date in [
            "2015",
            "2015-03",
            "2015-03-01",
            "2015-03-01T00:00:00+00:00",
            " 2015 ",
        ] {
            assert_eq!(year_of(date), Some(2015), "{date}");
        }
    }

    #[test]
    fn a_date_without_a_known_year_gives_none() {
        // the unknown date of Calibre, shifted by a time zone too, then dates that do not start with a year
        for date in [
            "0101-01-01T00:00:00+00:00",
            "0100-12-31T23:00:00+00:00",
            "",
            "15",
            "20150301",
            "March 2015",
            "2015.03.01",
        ] {
            assert_eq!(year_of(date), None, "{date}");
        }
    }
}
