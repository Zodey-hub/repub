use anyhow::Context;
use build_html::{Html, HtmlContainer, HtmlElement, HtmlTag};
use serde::Serialize;
use tiny_database::Database;

use super::Page;
use crate::library::Book;
use crate::router::{COVER_PATH, DOWNLOAD_PATH};

/// The width of the cover on the book page, in pixels; a narrower screen scales it down.
const COVER_WIDTH: &str = "300";

/// `GET /book?id=<id>`: the page of the book `id`, or `None` if there is no such book.
///
/// # Errors
///
/// Returns an error if the book cannot be read from the database.
pub fn book(database: &Database, id: u64) -> anyhow::Result<Option<BookPage>> {
    let book = database
        .get::<Book>(id)
        .with_context(|| format!("cannot read the book {id}"))?;

    Ok(book.map(|book| BookPage { book }))
}

/// The book page, in JSON the book itself: `{"id":1,"file_name":"...","title":"...",...}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct BookPage {
    book: Book,
}

impl Page for BookPage {
    fn title(&self) -> &str {
        self.book.title()
    }

    fn content(&self) -> HtmlElement {
        let book = &self.book;

        // every text comes from an EPUB file, it must not be able to inject markup
        let title = build_html::escape_html(book.title());
        let authors = build_html::escape_html(&book.authors().join(", "));
        let language = build_html::escape_html(book.language().unwrap_or_default());
        let description = build_html::escape_html(book.description().unwrap_or_default());

        let download_link = HtmlElement::new(HtmlTag::Link)
            .with_attribute("href", format!("{DOWNLOAD_PATH}?id={}", book.id()))
            .with_attribute("class", "button")
            .with_raw("Download EPUB");

        HtmlElement::new(HtmlTag::Main)
            .with_header(2, title)
            .with_raw(cover_image(book))
            .with_paragraph(format!("Authors: {authors}"))
            .with_paragraph(format!("Language: {language}"))
            .with_paragraph(description)
            .with_paragraph(download_link)
    }
}

/// The `<img>` of the cover of `book`, or nothing if it has no cover.
fn cover_image(book: &Book) -> String {
    if !book.has_cover() {
        return String::new();
    }

    HtmlElement::new(HtmlTag::Image)
        .with_attribute("src", format!("{COVER_PATH}?id={}", book.id()))
        .with_attribute("alt", "")
        .with_attribute("width", COVER_WIDTH)
        .to_html_string()
}
