//! The pages of the site: every page is a data struct, serde gives its JSON form and [`Page`] its HTML form, the `Accept` header of the request chooses between them.

mod health;
mod home;
mod internal_error;
mod not_found;

use build_html::{Html, HtmlContainer, HtmlElement, HtmlPage, HtmlTag};
use http::header::{ACCEPT, CONTENT_TYPE, VARY};
use http::{HeaderMap, Response, StatusCode};
use serde::Serialize;
use tiny_http_server::Body;

use crate::router::{HEALTH_PATH, HOME_PATH, STYLE_PATH};

pub use health::health;
pub use home::home;
pub use internal_error::internal_error;
pub use not_found::not_found;

const HTML_CONTENT_TYPE: &str = "text/html; charset=utf-8";
const JSON_CONTENT_TYPE: &str = "application/json";

/// The name of the site in the header of every HTML page.
const SITE_NAME: &str = env!("CARGO_PKG_NAME");
/// The running version of the site, in the footer of every HTML page and in the health check.
const SITE_VERSION: &str = env!("CARGO_PKG_VERSION");
/// The name of the author of the site, without the email address, which a public page must not show.
const SITE_AUTHOR: &str = author_name(env!("CARGO_PKG_AUTHORS"));
/// The license of the site, in the footer of every HTML page.
const SITE_LICENSE: &str = env!("CARGO_PKG_LICENSE");
/// The source code repository of the site, linked from the footer of every HTML page.
const SITE_REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

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
        .header(VARY, ACCEPT.as_str()) // the same URL answers in two forms, a cache must keep them apart
        .body(Body::from(body))
        .unwrap() // cannot fail, the status and the headers are valid constants
}

/// The response of a page that could not be built because of a server error: the error is logged, the client gets the internal error page.
pub fn render_or_error(page: anyhow::Result<impl Page>, format: Format) -> Response<Body> {
    match page {
        Ok(page) => render(&page, format),
        Err(error) => {
            log::error!("{error:#}");
            render(&internal_error(), format)
        }
    }
}

/// The whole HTML document of `page`.
fn html_page(page: &impl Page) -> Vec<u8> {
    HtmlPage::new()
        .with_meta(CHARSET_META)
        .with_meta(VIEWPORT_META)
        .with_title(build_html::escape_html(page.title()))
        .with_stylesheet(STYLE_PATH)
        .with_html(site_header())
        .with_html(page.content())
        .with_html(site_footer())
        .to_html_string()
        .into_bytes()
}

/// The header of every HTML page: the name of the site, a link to the home page.
fn site_header() -> HtmlElement {
    let site_name = HtmlElement::new(HtmlTag::Link)
        .with_attribute("href", HOME_PATH)
        .with_raw(SITE_NAME);

    HtmlElement::new(HtmlTag::Header).with_header(1, site_name)
}

/// The footer of every HTML page:
///
/// 1. the running version, a link to the source code and to the health check
/// 2. the author and the license
fn site_footer() -> HtmlElement {
    let source_link = HtmlElement::new(HtmlTag::Link)
        .with_attribute("href", SITE_REPOSITORY)
        .with_raw("Source");

    let health_link = HtmlElement::new(HtmlTag::Link)
        .with_attribute("href", HEALTH_PATH)
        .with_raw("Health");

    let about_line = format!("{SITE_NAME} {SITE_VERSION} | {source_link} | {health_link}");
    let credits_line = format!("by {SITE_AUTHOR} | {SITE_LICENSE}");

    HtmlElement::new(HtmlTag::Footer)
        .with_paragraph(about_line)
        .with_paragraph(credits_line)
}

/// The name part of a Cargo author, `Name <email>` -> `Name`; more than one author is a compile error.
const fn author_name(author: &str) -> &str {
    let bytes = author.as_bytes();

    // Cargo separates the authors with `:`, a second one would be dropped silently otherwise
    let mut index = 0;
    while index < bytes.len() {
        assert!(bytes[index] != b':', "more than one author in Cargo.toml");
        index += 1;
    }

    let mut name_end = 0;
    while name_end < bytes.len() && bytes[name_end] != b'<' {
        name_end += 1;
    }

    author.split_at(name_end).0.trim_ascii_end()
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
