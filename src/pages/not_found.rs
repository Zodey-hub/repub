use build_html::{HtmlContainer, HtmlElement, HtmlTag};
use http::{Response, StatusCode};
use tiny_http_server::Body;

use super::layout;
use crate::router::HOME_PATH;

/// The page of a request without a page: 404 with a link back to the home page.
pub fn not_found() -> Response<Body> {
    let content = HtmlElement::new(HtmlTag::Main)
        .with_header(1, "Page not found")
        .with_link(HOME_PATH, "Back to the home page");

    layout::page_response(StatusCode::NOT_FOUND, "Page not found", content)
}
