//! The pages of the site: every page is a data struct, serde gives its JSON form and [`Page`] its HTML form, the `Accept` header of the request chooses between them.

mod health;
mod home;
mod not_found;

use build_html::{Html, HtmlContainer, HtmlElement, HtmlPage};
use http::header::{ACCEPT, CONTENT_TYPE, VARY};
use http::{HeaderMap, Response, StatusCode};
use serde::Serialize;
use tiny_http_server::Body;

pub use health::health;
pub use home::home;
pub use not_found::not_found;

const HTML_CONTENT_TYPE: &str = "text/html; charset=utf-8";
const JSON_CONTENT_TYPE: &str = "application/json";

/// The `<meta>` of the character encoding, the same on every HTML page.
const CHARSET_META: [(&str, &str); 1] = [("charset", "utf-8")];
/// The `<meta>` of the viewport, the same on every HTML page.
const VIEWPORT_META: [(&str, &str); 2] = [
    ("name", "viewport"),
    ("content", "width=device-width, initial-scale=1"),
];

/// The data of a page: serde gives its JSON form, this trait its HTML form.
pub trait Page: Serialize {
    /// The title of the HTML page, escaped by [`render`].
    fn title(&self) -> &str;

    /// The body of the HTML page; a text taken from outside must be escaped with `build_html::escape_html`.
    fn content(&self) -> HtmlElement;

    /// The status code of the response, in both forms.
    fn status(&self) -> StatusCode {
        StatusCode::OK
    }
}

/// The form of a response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// A whole HTML page, for a browser.
    Html,
    /// The JSON of the page data, for curl, scripts and monitoring.
    Json,
}

impl Format {
    /// The form the request asks for: HTML when its `Accept` lists `text/html` (every browser sends it), JSON otherwise.
    pub fn from_accept(headers: &HeaderMap) -> Self {
        let accepts_html = headers
            .get_all(ACCEPT)
            .iter()
            .filter_map(|value| value.to_str().ok())
            .flat_map(|value| value.split(','))
            .map(|media_range| media_range.split(';').next().unwrap_or_default().trim())
            .any(|media_type| media_type.eq_ignore_ascii_case("text/html"));

        if accepts_html { Self::Html } else { Self::Json }
    }
}

/// The response of `page` in `format`.
pub fn render(page: &impl Page, format: Format) -> Response<Body> {
    let (content_type, body) = match format {
        Format::Html => (HTML_CONTENT_TYPE, html_page(page)),
        Format::Json => (JSON_CONTENT_TYPE, serde_json::to_vec(page).unwrap()), // cannot fail, the pages are structs with string keys
    };

    Response::builder()
        .status(page.status())
        .header(CONTENT_TYPE, content_type)
        // the same URL answers in two forms, a cache must keep them apart
        .header(VARY, ACCEPT.as_str())
        .body(Body::from(body))
        .unwrap() // cannot fail, the status and the headers are valid constants
}

/// The whole HTML document of `page`.
fn html_page(page: &impl Page) -> Vec<u8> {
    let title = build_html::escape_html(page.title());

    HtmlPage::new()
        .with_meta(CHARSET_META)
        .with_meta(VIEWPORT_META)
        .with_title(title)
        .with_html(page.content())
        .to_html_string()
        .into_bytes()
}

#[cfg(test)]
mod tests {
    use http::HeaderValue;

    use super::*;

    /// The format of a request with these `Accept` headers.
    fn format_of(accept_headers: &[&'static str]) -> Format {
        let mut headers = HeaderMap::new();
        for accept in accept_headers {
            headers.append(ACCEPT, HeaderValue::from_static(accept));
        }

        Format::from_accept(&headers)
    }

    #[test]
    fn a_browser_gets_html() {
        // the Accept of Firefox and Chrome on a page load, then one with spaces, upper case and a parameter
        for accept in [
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8",
            "application/json , TEXT/HTML;q=0.5",
        ] {
            assert_eq!(format_of(&[accept]), Format::Html, "{accept}");
        }

        // the header can come in several lines too
        assert_eq!(format_of(&["application/json", "text/html"]), Format::Html);
    }

    #[test]
    fn everything_else_gets_json() {
        // no Accept, the Accept of curl, a JSON client, and media types that only contain text/html
        for accept_headers in [
            &[][..],
            &["*/*"],
            &["application/json"],
            &["text/*"],
            &["text/htmlx, application/text/html"],
        ] {
            assert_eq!(
                format_of(accept_headers),
                Format::Json,
                "{accept_headers:?}"
            );
        }
    }
}
