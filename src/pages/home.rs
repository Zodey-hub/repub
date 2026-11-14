use build_html::{HtmlContainer, HtmlElement, HtmlTag};
use http::{Response, StatusCode};
use tiny_http_server::Body;

use super::layout;

/// `GET /`: the home page.
pub fn home() -> Response<Body> {
    let content = HtmlElement::new(HtmlTag::Main)
        .with_header(1, "repub")
        .with_paragraph("Search and download EPUB books.");

    layout::page_response(StatusCode::OK, "repub", content)
}
