use build_html::{HtmlContainer, HtmlElement, HtmlTag};
use serde::Serialize;

use super::Page;

/// `GET /`: the home page.
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
        HtmlElement::new(HtmlTag::Main)
            .with_header(1, self.name)
            .with_paragraph(self.description)
    }
}
