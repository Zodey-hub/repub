//! The frame shared by every page: the HTML document around its content, and the response that carries it.

use build_html::{Html, HtmlContainer, HtmlPage};
use http::header::CONTENT_TYPE;
use http::{HeaderValue, Response, StatusCode};
use tiny_http_server::Body;

const HTML_CONTENT_TYPE: &str = "text/html; charset=utf-8";

/// A response with `status` and a whole HTML page with `title` and `content` in its body; the title is escaped, the content is not.
pub fn page_response(status: StatusCode, title: &str, content: impl Html) -> Response<Body> {
    let title = build_html::escape_html(title);

    let page = HtmlPage::new()
        .with_meta([("charset", "utf-8")])
        .with_meta([
            ("name", "viewport"),
            ("content", "width=device-width, initial-scale=1"),
        ])
        .with_title(title)
        .with_html(content)
        .to_html_string();

    let mut response = Response::new(Body::from(page));
    *response.status_mut() = status;
    response
        .headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static(HTML_CONTENT_TYPE));

    response
}
