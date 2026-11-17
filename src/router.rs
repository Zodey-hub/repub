//! The single routes file: every path of the site is paired with its page here.

use std::future;

use http::{Method, Request, Response, Uri};
use tiny_database::Database;
use tiny_http_server::{Body, Handler};

use crate::files;
use crate::pages::{self, Format};

/// The path of the home page.
pub const HOME_PATH: &str = "/";
/// The path of the health check.
pub const HEALTH_PATH: &str = "/health";
/// The path of the stylesheet of every HTML page.
pub const STYLE_PATH: &str = "/style.css";
/// The path of the cover of a book: `/cover?id=<id>`.
pub const COVER_PATH: &str = "/cover";

/// The handler of the server: answers every request with the page of its method and path.
pub struct Router {
    database: Database,
}

impl Router {
    /// The router of the pages that read the books from `database`.
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    /// The page of a method and a URI in `format`; an unknown path, or a method other than `GET`, gets the not found page.
    fn respond(&self, method: &Method, uri: &Uri, format: Format) -> Response<Body> {
        let database = &self.database;

        match (method, uri.path()) {
            (&Method::GET, HOME_PATH) => pages::render_or_error(pages::home(database), format),
            (&Method::GET, HEALTH_PATH) => pages::render(&pages::health(), format),
            (&Method::GET, COVER_PATH) => files::cover(database, uri.query(), format),
            (&Method::GET, STYLE_PATH) => files::style(),
            _ => pages::render(&pages::not_found(), format),
        }
    }
}

impl Handler for Router {
    fn handle(&self, request: Request<Body>) -> impl Future<Output = Response<Body>> + Send {
        let format = Format::from_accept(request.headers());
        future::ready(self.respond(request.method(), request.uri(), format))
    }
}
