use anyhow::Context;
use build_html::{Html, HtmlContainer, HtmlElement, HtmlTag, Table};
use serde::Serialize;
use tiny_database::Database;

use super::Page;
use crate::library::Book;
use crate::router::{BOOK_PATH, COVER_PATH};

/// The width of a cover in the list of books, in pixels.
const COVER_WIDTH: &str = "60";

/// `GET /`: the home page with every book of the library.
///
/// # Errors
///
/// Returns an error if the books cannot be read from the database.
pub fn home(database: &Database) -> anyhow::Result<Home> {
    let books = database.all::<Book>().context("cannot read the books")?;

    Ok(Home {
        name: "repub",
        description: "Search and download EPUB books.",
        books,
    })
}

/// The home page, in JSON `{"name":"repub","description":"...","books":[...]}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Home {
    name: &'static str,
    description: &'static str,
    books: Vec<Book>,
}

impl Page for Home {
    fn title(&self) -> &str {
        self.name
    }

    fn content(&self) -> HtmlElement {
        // the name of the site is already in the header of the page
        let main = HtmlElement::new(HtmlTag::Main).with_paragraph(self.description);

        if self.books.is_empty() {
            return main.with_paragraph(
                "No books yet: put EPUB files into the library directory and restart the server.",
            );
        }

        main.with_table(book_table(&self.books))
    }
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
