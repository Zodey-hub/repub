use build_html::{HtmlContainer, HtmlElement, HtmlTag};
use http::StatusCode;
use serde::Serialize;

use super::Page;
use crate::router::HOME_PATH;

/// The page of a request without a page: 404 with a link back to the home page.
pub fn not_found() -> NotFound {
    NotFound { error: "not_found" }
}

/// The not found page, in JSON `{"error":"not_found"}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NotFound {
    error: &'static str,
}

impl Page for NotFound {
    fn title(&self) -> &'static str {
        "Page not found"
    }

    fn content(&self) -> HtmlElement {
        HtmlElement::new(HtmlTag::Main)
            .with_header(2, "Page not found")
            .with_link(HOME_PATH, "Back to the home page")
    }

    fn status(&self) -> StatusCode {
        StatusCode::NOT_FOUND
    }
}
