//! The static files of the site, built into the binary.

use http::Response;
use http::header::{CACHE_CONTROL, CONTENT_TYPE};
use tiny_http_server::Body;

const CSS_CONTENT_TYPE: &str = "text/css; charset=utf-8";
/// A browser reuses the stylesheet for a day without asking again.
const STYLE_CACHE_CONTROL: &str = "public, max-age=86400";

/// The stylesheet of every HTML page: Simple.css, a classless CSS that styles the plain HTML elements.
const STYLE: &[u8] = include_bytes!("../assets/simple.min.css");

/// `GET /style.css`: the stylesheet of every HTML page.
pub fn style() -> Response<Body> {
    Response::builder()
        .header(CONTENT_TYPE, CSS_CONTENT_TYPE)
        .header(CACHE_CONTROL, STYLE_CACHE_CONTROL)
        .body(Body::from_static(STYLE))
        .unwrap() // cannot fail, the headers are constants
}
