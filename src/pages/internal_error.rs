use build_html::{HtmlContainer, HtmlElement, HtmlTag};
use http::StatusCode;
use serde::Serialize;

use super::Page;

/// The page of a request that failed on the server: 500, the cause is only in the log.
pub fn internal_error() -> InternalError {
    InternalError {
        error: "internal_error",
    }
}

/// The internal error page, in JSON `{"error":"internal_error"}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InternalError {
    error: &'static str,
}

impl Page for InternalError {
    fn title(&self) -> &'static str {
        "Server error"
    }

    fn content(&self) -> HtmlElement {
        HtmlElement::new(HtmlTag::Main)
            .with_header(2, "Server error")
            .with_paragraph("Something went wrong on the server, please try again later.")
    }

    fn status(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}
