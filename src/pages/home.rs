use build_html::{HtmlContainer, HtmlElement, HtmlTag};
use serde::Serialize;

use super::Page;
use crate::router::{SEARCH_PARAMETER, SEARCH_PATH};

/// `GET /`: the home page, the search form.
pub fn home() -> Home {
    Home {
        name: "repub",
        description: "Search and download EPUB books.",
    }
}

/// The home page, in JSON `{"name":"repub","description":"..."}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Home {
    name: &'static str,
    description: &'static str,
}

impl Page for Home {
    fn title(&self) -> &str {
        self.name
    }

    fn content(&self) -> HtmlElement {
        // the name of the site is already in the header of the page
        HtmlElement::new(HtmlTag::Main)
            .with_paragraph(self.description)
            .with_html(search_form())
    }
}

/// The search form; a plain `GET` form, so the results have a URL like any other page.
fn search_form() -> HtmlElement {
    let input = HtmlElement::new(HtmlTag::Input)
        .with_attribute("type", "search")
        .with_attribute("name", SEARCH_PARAMETER)
        .with_attribute("placeholder", "Title or author")
        .with_attribute("aria-label", "Search")
        .with_attribute("autofocus", "");

    HtmlElement::new(HtmlTag::Form)
        .with_attribute("action", SEARCH_PATH)
        .with_html(input)
        .with_raw(" ")
        .with_html(HtmlElement::new(HtmlTag::Button).with_raw("Search"))
}
