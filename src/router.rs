//! The single routes file: every path of the site is paired with its page here.

use std::future;
use std::path::PathBuf;

use http::{Method, Request, Response, Uri};
use percent_encoding::percent_decode_str;
use tiny_database::Database;
use tiny_http_server::{Body, Handler};

use crate::files;
use crate::pages::{self, Format};
use crate::reply::Reply;

/// The path of the home page.
pub const HOME_PATH: &str = "/";
/// The path of the search results: `/search?q=<query>`.
pub const SEARCH_PATH: &str = "/search";
/// The name of the query parameter of the search.
pub const SEARCH_PARAMETER: &str = "q";
/// The path of the page of a book: `/book?id=<id>`.
pub const BOOK_PATH: &str = "/book";
/// The path of the EPUB file of a book: `/download?id=<id>`.
pub const DOWNLOAD_PATH: &str = "/download";
/// The path of the health check.
pub const HEALTH_PATH: &str = "/health";
/// The path of the stylesheet of every HTML page.
pub const STYLE_PATH: &str = "/style.css";
/// The path of the cover of a book: `/cover?id=<id>`.
pub const COVER_PATH: &str = "/cover";

/// The handler of the server: answers every request with the page of its method and path.
pub struct Router {
    database: Database,
    library_dir: PathBuf,
}

impl Router {
    /// The router of the pages that read the books from `database`, and their files from `library_dir`.
    pub fn new(database: Database, library_dir: PathBuf) -> Self {
        Self {
            database,
            library_dir,
        }
    }

    /// The page of a method and a URI in `format`; an unknown path, a method other than `GET`, or a book path without a valid `id` gets the not found page.
    fn respond(&self, method: &Method, uri: &Uri, format: Format) -> Response<Body> {
        let database = &self.database;
        let library = &self.library_dir;
        let id = query_parameter(uri.query(), "id").and_then(|id| id.parse().ok());
        let query = query_parameter(uri.query(), SEARCH_PARAMETER).unwrap_or_default();

        match (method, uri.path(), id) {
            (&Method::GET, HOME_PATH, _) => pages::home().into_response(format),
            (&Method::GET, SEARCH_PATH, _) => pages::search(database, &query).into_response(format),
            (&Method::GET, BOOK_PATH, Some(id)) => pages::book(database, id).into_response(format),
            (&Method::GET, HEALTH_PATH, _) => pages::health().into_response(format),
            (&Method::GET, COVER_PATH, Some(id)) => {
                files::cover(database, id).into_response(format)
            }
            (&Method::GET, DOWNLOAD_PATH, Some(id)) => {
                files::download(database, library, id).into_response(format)
            }
            (&Method::GET, STYLE_PATH, _) => files::style(),
            _ => pages::not_found().into_response(format),
        }
    }
}

impl Handler for Router {
    fn handle(&self, request: Request<Body>) -> impl Future<Output = Response<Body>> + Send {
        let format = Format::from_accept(request.headers());
        future::ready(self.respond(request.method(), request.uri(), format))
    }
}

/// The decoded value of the parameter `name` in `query`, or `None` if it is missing.
fn query_parameter(query: Option<&str>, name: &str) -> Option<String> {
    let value = query?
        .split('&')
        .find_map(|parameter| parameter.strip_prefix(name)?.strip_prefix('='))?;

    // a form sends a space as `+`, a real `+` comes percent-encoded
    let value = value.replace('+', " ");

    Some(percent_decode_str(&value).decode_utf8_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_parameter_is_decoded_like_a_form_sends_it() {
        let query = Some("qq=x&q=J%C3%B3kai+M%C3%B3r%2B&id=7");

        assert_eq!(query_parameter(query, "q").as_deref(), Some("Jókai Mór+"));
        assert_eq!(query_parameter(query, "id").as_deref(), Some("7"));
        assert_eq!(query_parameter(query, "i"), None);
        assert_eq!(query_parameter(None, "q"), None);
    }
}
