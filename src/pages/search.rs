use anyhow::Context;
use build_html::{Html, HtmlContainer, HtmlElement, HtmlTag, Table};
use serde::Serialize;
use tiny_database::Database;
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

use super::Page;
use crate::library::Book;
use crate::router::{BOOK_PATH, COVER_PATH};

/// The width of a cover in the list of books, in pixels.
const COVER_WIDTH: &str = "60";

/// `GET /search?q=<query>`: the books whose title or authors contain every word of `query`, ignoring case and accents; no book for an empty `query`.
///
/// # Errors
///
/// Returns an error if the books cannot be read from the database.
pub fn search(database: &Database, query: &str) -> anyhow::Result<Search> {
    let folded_query = folded(query);
    let words: Vec<&str> = folded_query.split_whitespace().collect();

    // an empty search lists nothing, a large library would be one huge page
    let books = if words.is_empty() {
        Vec::new()
    } else {
        let books = database.all::<Book>().context("cannot read the books")?;
        books
            .into_iter()
            .filter(|book| matches(book, &words))
            .collect()
    };

    Ok(Search {
        query: query.to_owned(),
        books,
    })
}

/// The search results, in JSON `{"query":"...","books":[...]}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Search {
    query: String,
    books: Vec<Book>,
}

impl Page for Search {
    fn title(&self) -> &'static str {
        "Search"
    }

    fn content(&self) -> HtmlElement {
        let main = HtmlElement::new(HtmlTag::Main);

        if self.query.trim().is_empty() {
            return main;
        }

        if self.books.is_empty() {
            return main.with_paragraph("No book matches the search.");
        }

        main.with_table(book_table(&self.books))
    }
}

/// The `text` in the form it is searched in: lower case, without accents.
fn folded(text: &str) -> String {
    // lower case first, it can give a letter with an accent (`İ` -> `i̇`)
    text.to_lowercase()
        .nfd()
        .filter(|character| !is_combining_mark(*character))
        .collect()
}

/// Whether the title or the authors of `book` contain each of the folded `words`.
fn matches(book: &Book, words: &[&str]) -> bool {
    // the fields are separated by a line break, which no word contains, so a word cannot match across two of them
    let text = folded(&format!("{}\n{}", book.title(), book.authors().join("\n")));
    words.iter().all(|word| text.contains(word))
}

/// The table of `books`: the cover, the title, the authors and the language of each.
fn book_table(books: &[Book]) -> Table {
    let mut table = Table::new().with_header_row(["", "Title", "Authors", "Language"]);

    for book in books {
        // every text comes from an EPUB file, it must not be able to inject markup
        let title = build_html::escape_html(book.title());
        let title_link = HtmlElement::new(HtmlTag::Link)
            .with_attribute("href", format!("{BOOK_PATH}?id={}", book.id()))
            .with_raw(title)
            .to_html_string();
        let authors = build_html::escape_html(&book.authors().join(", "));
        let language = build_html::escape_html(book.language().unwrap_or_default());

        table.add_body_row([cover_image(book), title_link, authors, language]);
    }

    table
}

/// The `<img>` of the cover of `book`, or nothing if it has no cover.
fn cover_image(book: &Book) -> String {
    if !book.has_cover() {
        return String::new();
    }

    let cover_path = format!("{COVER_PATH}?id={}", book.id());

    HtmlElement::new(HtmlTag::Image)
        .with_attribute("src", cover_path)
        .with_attribute("alt", "")
        .with_attribute("width", COVER_WIDTH)
        .with_attribute("loading", "lazy")
        .to_html_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_search_ignores_case_and_accents() {
        assert_eq!(
            folded("Jókai MÓR: A KŐSZÍVŰ ember fiai, Çà"),
            "jokai mor: a koszivu ember fiai, ca"
        );
    }
}
