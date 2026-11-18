use anyhow::Context;
use build_html::{HtmlContainer, HtmlElement, HtmlTag};
use serde::Serialize;
use tiny_database::Database;

use super::{Page, WithOptional};
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

        let authors = joined(book.authors());
        let translators = joined(book.translators());
        let year = book.year().map(|year| year.to_string());

        // every text comes from an EPUB file, it must not be able to inject markup
        let title = build_html::escape_html(book.title());
        let description = book.description().map(text_paragraph);

        let download_button = HtmlElement::new(HtmlTag::ParagraphText).with_link_attr(
            format!("{DOWNLOAD_PATH}?id={}", book.id()),
            "Download EPUB",
            [("class", "button")],
        );

        // a detail the EPUB file does not have gets no line
        HtmlElement::new(HtmlTag::Main)
            .with_header(2, title)
            .with_optional(cover_image(book))
            .with_optional(detail_line("Authors", authors.as_deref()))
            .with_optional(detail_line("Translators", translators.as_deref()))
            .with_optional(detail_line("Publisher", book.publisher()))
            .with_optional(detail_line("Year", year.as_deref()))
            .with_optional(detail_line("Language", book.language()))
            .with_optional(subject_list(book))
            .with_optional(description)
            .with_html(download_button)
    }
}

/// The `<img>` of the cover of `book`, or `None` if it has no cover.
fn cover_image(book: &Book) -> Option<HtmlElement> {
    if !book.has_cover() {
        return None;
    }

    let image = HtmlElement::new(HtmlTag::Image)
        .with_attribute("src", format!("{COVER_PATH}?id={}", book.id()))
        .with_attribute("alt", "")
        .with_attribute("width", COVER_WIDTH);

    Some(image)
}

/// The subjects of `book` one under the other below their label, as a description list; `None` if it has none.
fn subject_list(book: &Book) -> Option<HtmlElement> {
    if book.subjects().is_empty() {
        return None;
    }

    // Simple.css paints a term lighter, the label keeps the color of the other details
    let label = HtmlElement::new(HtmlTag::DescriptionListTerm)
        .with_attribute("style", "color: inherit")
        .with_raw("Subjects:");

    // every subject comes from an EPUB file, it must not be able to inject markup
    let list = book
        .subjects()
        .iter()
        .map(|subject| {
            HtmlElement::new(HtmlTag::DescriptionListDescription)
                .with_raw(build_html::escape_html(subject))
        })
        .fold(
            HtmlElement::new(HtmlTag::DescriptionList).with_html(label),
            HtmlContainer::with_html,
        );

    Some(list)
}

/// The `label: value` line of a detail of the book, `None` if the book does not have it.
fn detail_line(label: &str, value: Option<&str>) -> Option<HtmlElement> {
    Some(text_paragraph(&format!("{label}: {}", value?)))
}

/// A `<p>` of `text` with its markup escaped.
fn text_paragraph(text: &str) -> HtmlElement {
    HtmlElement::new(HtmlTag::ParagraphText).with_raw(build_html::escape_html(text))
}

/// The `names` separated by commas, `None` if there are none.
fn joined(names: &[String]) -> Option<String> {
    (!names.is_empty()).then(|| names.join(", "))
}

#[cfg(test)]
mod tests {
    use build_html::Html;

    use super::*;

    #[test]
    fn a_text_from_an_epub_file_cannot_inject_markup() {
        let paragraph = text_paragraph("<script>alert(1)</script> & co");

        assert_eq!(
            paragraph.to_html_string(),
            "<p>&lt;script&gt;alert(1)&lt;/script&gt; &amp; co</p>"
        );
    }
}
