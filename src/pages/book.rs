use anyhow::Context;
use build_html::{HtmlContainer, HtmlElement, HtmlTag};
use serde::Serialize;
use tiny_database::Database;

use super::{Page, WithEach, WithOptional};
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
        let details = HtmlElement::new(HtmlTag::DescriptionList)
            .with_optional(detail("Authors", authors.as_slice()))
            .with_optional(detail("Translators", translators.as_slice()))
            .with_optional(detail("Publisher", book.publisher().as_slice()))
            .with_optional(detail("Year", year.as_slice()))
            .with_optional(detail("Language", book.language().as_slice()))
            .with_optional(detail("Subjects", book.subjects()));

        HtmlElement::new(HtmlTag::Main)
            .with_header(2, title)
            .with_optional(cover_image(book))
            .with_html(details)
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

/// A detail of the book: a `<div>` of the `<dt>` of `label` and a `<dd>` for each of `values`; `None` if it has no values.
fn detail<S: AsRef<str>>(label: &str, values: &[S]) -> Option<HtmlElement> {
    if values.is_empty() {
        return None;
    }

    // every value comes from an EPUB file, it must not be able to inject markup
    let descriptions = values.iter().map(|value| {
        HtmlElement::new(HtmlTag::DescriptionListDescription)
            .with_raw(build_html::escape_html(value.as_ref()))
    });

    let term = HtmlElement::new(HtmlTag::DescriptionListTerm).with_raw(label);
    let group = HtmlElement::new(HtmlTag::Div)
        .with_html(term)
        .with_each(descriptions);

    Some(group)
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

    #[test]
    fn a_detail_gets_a_description_for_each_value_and_none_without_values() {
        let subjects = detail("Subjects", &["<b>Poetry</b>", "Novel"]);
        let publisher = detail::<&str>("Publisher", &[]);

        assert_eq!(
            subjects
                .map(|subjects| subjects.to_html_string())
                .as_deref(),
            Some("<div><dt>Subjects</dt><dd>&lt;b&gt;Poetry&lt;/b&gt;</dd><dd>Novel</dd></div>")
        );
        assert!(publisher.is_none());
    }
}
