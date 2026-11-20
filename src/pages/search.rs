use std::time::{Duration, Instant};

use anyhow::Context;
use build_html::{Html, HtmlContainer, HtmlElement, HtmlTag, Table};
use serde::Serialize;
use tiny_database::{Database, key};

use super::Page;
use crate::library::{Book, BookIndex, text};
use crate::router::{self, BOOK_PATH, COVER_PATH, SEARCH_PATH};

/// The width of a cover in the list of books, in pixels.
const COVER_WIDTH: &str = "60";

/// The query parameters of the search, also the names of the fields of the search form.
const TITLE_PARAMETER: &str = "title";
const AUTHOR_PARAMETER: &str = "author";
const TRANSLATOR_PARAMETER: &str = "translator";
const SUBJECT_PARAMETER: &str = "subject";
const PUBLISHER_PARAMETER: &str = "publisher";
const LANGUAGE_PARAMETER: &str = "language";
const YEAR_FROM_PARAMETER: &str = "year_from";
const YEAR_TO_PARAMETER: &str = "year_to";

/// `GET /search?title=&author=&translator=&subject=&publisher=&language=&year_from=&year_to=`: the books that match every given criterion; no book without a criterion.
///
/// The texts match ignoring case and accents, each word of a criterion has to start a word of that field (`jok` finds `Jókai`); the language matches its tag or primary subtag (`en` finds `en-US`); the years are inclusive.
///
/// # Errors
///
/// Returns an error if the books cannot be read from the database.
pub fn search(database: &Database, query: Option<&str>) -> anyhow::Result<Search> {
    let criteria = Criteria::from_query(query);

    if criteria.is_empty() {
        return Ok(Search {
            criteria,
            books: Vec::new(),
            stats: None,
        });
    }

    let started = Instant::now();

    let folded_criteria = criteria.folded();
    let mut books = folded_criteria
        .candidates(database)
        .context("cannot search the books")?;

    let candidates = books.len();

    books.retain(|book| folded_criteria.matches(book));
    books.sort_by_cached_key(|book| text::folded(book.title()));

    let elapsed_us = u64::try_from(started.elapsed().as_micros()).unwrap(); // cannot fail, a search does not take 584 000 years
    let stats = SearchStats {
        candidates,
        elapsed_us,
    };

    Ok(Search {
        criteria,
        books,
        stats: Some(stats),
    })
}

/// The search results, in JSON `{"criteria":{"title":"...","author":null,...},"books":[...],"stats":{"candidates":37,"elapsed_us":412}}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Search {
    criteria: Criteria,
    books: Vec<Book>,
    stats: Option<SearchStats>,
}

impl Page for Search {
    fn title(&self) -> &'static str {
        "Search"
    }

    fn content(&self) -> HtmlElement {
        let main = HtmlElement::new(HtmlTag::Main);

        let Some(stats) = &self.stats else {
            return main;
        };

        let main = main.with_paragraph(stats.summary(self.books.len()));

        if self.books.is_empty() {
            return main.with_paragraph("No book matches the search.");
        }

        main.with_table(book_table(&self.books))
    }
}

/// The search form, for the home page: every field is optional, a book has to match each filled one.
pub fn search_form() -> HtmlElement {
    let fields = [
        ("Title", TITLE_PARAMETER, "search"),
        ("Author", AUTHOR_PARAMETER, "search"),
        ("Translator", TRANSLATOR_PARAMETER, "search"),
        ("Subject", SUBJECT_PARAMETER, "search"),
        ("Publisher", PUBLISHER_PARAMETER, "search"),
        ("Language (hu, en, ...)", LANGUAGE_PARAMETER, "search"),
        ("Year from", YEAR_FROM_PARAMETER, "number"),
        ("Year to", YEAR_TO_PARAMETER, "number"),
    ];

    // a plain `GET` form, so the results have a URL like any other page
    let form = HtmlElement::new(HtmlTag::Form).with_attribute("action", SEARCH_PATH);
    let button = HtmlElement::new(HtmlTag::Button).with_raw("Search");

    fields
        .into_iter()
        .map(|(label, name, input_type)| form_field(label, name, input_type))
        .fold(form, HtmlContainer::with_html)
        .with_html(button)
}

/// The criteria of a search, each `None` if it is not given.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
struct Criteria {
    title: Option<String>,
    author: Option<String>,
    translator: Option<String>,
    subject: Option<String>,
    publisher: Option<String>,
    language: Option<String>,
    year_from: Option<u16>,
    year_to: Option<u16>,
}

impl Criteria {
    /// The criteria in the URL `query`; a field without a letter or digit, or a year that is not a number is no criterion.
    fn from_query(query: Option<&str>) -> Self {
        let text = |name| {
            router::query_parameter(query, name)
                .map(|value| value.trim().to_owned())
                .filter(|value| text::words(value).next().is_some())
        };
        let year = |name| text(name)?.parse().ok();

        Self {
            title: text(TITLE_PARAMETER),
            author: text(AUTHOR_PARAMETER),
            translator: text(TRANSLATOR_PARAMETER),
            subject: text(SUBJECT_PARAMETER),
            publisher: text(PUBLISHER_PARAMETER),
            language: text(LANGUAGE_PARAMETER),
            year_from: year(YEAR_FROM_PARAMETER),
            year_to: year(YEAR_TO_PARAMETER),
        }
    }

    fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    /// The criteria with their texts folded, in the form the books are compared in.
    fn folded(&self) -> Self {
        let fold = |value: &Option<String>| value.as_deref().map(text::folded);

        Self {
            title: fold(&self.title),
            author: fold(&self.author),
            translator: fold(&self.translator),
            subject: fold(&self.subject),
            publisher: fold(&self.publisher),
            language: fold(&self.language),
            year_from: self.year_from,
            year_to: self.year_to,
        }
    }

    /// The books that one criterion selects through its index, a superset of the results; the criteria must be [`folded`](Self::folded) and not empty.
    ///
    /// The first word of a text field is the most selective, then comes the language, then the years.
    fn candidates(&self, database: &Database) -> tiny_database::Result<Vec<Book>> {
        let text_criteria = [
            (BookIndex::Title, &self.title),
            (BookIndex::Author, &self.author),
            (BookIndex::Translator, &self.translator),
            (BookIndex::Subject, &self.subject),
            (BookIndex::Publisher, &self.publisher),
        ];

        let first_word = text_criteria
            .into_iter()
            .find_map(|(index, value)| Some((index, text::words(value.as_deref()?).next()?)));

        if let Some((index, word)) = first_word {
            return database.range(index, key::prefix_range(word));
        }

        if let Some(language) = &self.language {
            let language_key = key::enc(text::primary_language(language));
            return database.find(BookIndex::Language, &language_key);
        }

        let from = key::enc(self.year_from.unwrap_or(u16::MIN));
        let to = key::enc(self.year_to.unwrap_or(u16::MAX));
        database.range(BookIndex::Year, from..=to)
    }

    /// Whether `book` matches every criterion; the criteria must be [`folded`](Self::folded).
    fn matches(&self, book: &Book) -> bool {
        contains_words(&[book.title()], self.title.as_deref())
            && contains_words(book.authors(), self.author.as_deref())
            && contains_words(book.translators(), self.translator.as_deref())
            && contains_words(book.subjects(), self.subject.as_deref())
            && contains_words(book.publisher().as_slice(), self.publisher.as_deref())
            && language_matches(book.language(), self.language.as_deref())
            && year_matches(book.year(), self.year_from, self.year_to)
    }
}

/// How the index served a search: how many books it read and how long the whole search took.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct SearchStats {
    /// The books read from the index, before the other criteria filtered them.
    candidates: usize,
    /// The time of the index lookup, the filtering and the sorting, in microseconds.
    elapsed_us: u64,
}

impl SearchStats {
    /// The line of the results page: how many of the candidates matched and in how long.
    fn summary(&self, found: usize) -> String {
        let elapsed_ms = Duration::from_micros(self.elapsed_us).as_secs_f64() * 1000.0;
        let candidates = self.candidates;
        format!("{found} of {candidates} candidates from the index matched in {elapsed_ms:.2} ms.")
    }
}

/// A `<label>` with its `<input>` of `input_type` named `name` below it.
fn form_field(label: &str, name: &str, input_type: &str) -> HtmlElement {
    let input = HtmlElement::new(HtmlTag::Input)
        .with_attribute("type", input_type)
        .with_attribute("name", name);

    HtmlElement::new(HtmlTag::Label)
        .with_raw(label)
        .with_html(input)
}

/// Whether each word of the folded `query` starts a word of the `values` of a field; no `query` is no criterion.
fn contains_words<S: AsRef<str>>(values: &[S], query: Option<&str>) -> bool {
    let Some(query) = query else {
        return true;
    };

    let values: Vec<String> = values
        .iter()
        .map(|value| text::folded(value.as_ref()))
        .collect();

    text::words(query).all(|query_word| {
        values
            .iter()
            .flat_map(|value| text::words(value))
            .any(|word| word.starts_with(query_word))
    })
}

/// Whether the `language` tag of a book is the folded `query` tag or has it as its primary subtag; no `query` is no criterion.
fn language_matches(language: Option<&str>, query: Option<&str>) -> bool {
    let Some(query) = query else {
        return true;
    };

    let Some(language) = language else {
        return false;
    };

    let language = language.to_ascii_lowercase();
    let subtags = language.strip_prefix(query);

    subtags.is_some_and(|subtags| subtags.is_empty() || subtags.starts_with('-'))
}

/// Whether `year` is between `from` and `to`, both inclusive; a book without a year matches only when neither is given.
fn year_matches(year: Option<u16>, from: Option<u16>, to: Option<u16>) -> bool {
    if from.is_none() && to.is_none() {
        return true;
    }

    year.is_some_and(|year| from.is_none_or(|from| year >= from) && to.is_none_or(|to| year <= to))
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
    fn each_word_has_to_start_a_word_of_the_field() {
        let authors = ["Jókai Mór", "Gárdonyi Géza"];

        assert!(contains_words(&authors, Some("mor jok")));
        assert!(
            contains_words(&authors, Some("jokai geza")),
            "the words can be in different authors"
        );
        assert!(
            !contains_words(&authors, Some("okai")),
            "a word matches from its start"
        );
        assert!(!contains_words(&authors, Some("jokai mikszath")));
        assert!(
            !contains_words::<&str>(&[], Some("x")),
            "a missing field matches no word"
        );
        assert!(
            contains_words::<&str>(&[], None),
            "no words is no criterion"
        );
    }

    #[test]
    fn a_language_matches_its_tag_or_primary_subtag() {
        assert!(language_matches(Some("en-US"), Some("en")));
        assert!(language_matches(Some("HU"), Some("hu")));
        assert!(language_matches(Some("en-US"), Some("en-us")));
        assert!(!language_matches(Some("eng"), Some("en")));
        assert!(!language_matches(Some("en"), Some("en-us")));
        assert!(!language_matches(None, Some("hu")));
        assert!(language_matches(None, None));
    }

    #[test]
    fn the_years_are_inclusive_and_a_book_without_a_year_matches_no_range() {
        assert!(year_matches(Some(1900), Some(1900), Some(1900)));
        assert!(year_matches(Some(1872), None, Some(1900)));
        assert!(!year_matches(Some(1901), None, Some(1900)));
        assert!(!year_matches(Some(1899), Some(1900), None));
        assert!(!year_matches(None, Some(1900), None));
        assert!(year_matches(None, None, None));
    }

    #[test]
    fn a_blank_field_or_an_invalid_year_is_no_criterion() {
        let criteria = Criteria::from_query(Some(
            "title=+&author=J%C3%B3kai+&year_from=abc&year_to=1900",
        ));

        assert_eq!(criteria.title, None);
        assert_eq!(criteria.author.as_deref(), Some("Jókai"));
        assert_eq!(criteria.year_from, None);
        assert_eq!(criteria.year_to, Some(1900));
        assert!(Criteria::from_query(Some("title=&year_from=x")).is_empty());
    }
}
