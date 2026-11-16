//! The single routes file: every path of the site is paired with its page here.

use std::future;

use http::{Method, Request, Response};
use tiny_http_server::{Body, Handler};

use crate::pages::{self, Format};

/// The path of the home page.
pub const HOME_PATH: &str = "/";
/// The path of the health check.
pub const HEALTH_PATH: &str = "/health";

/// The handler of the server: answers every request with the page of its method and path.
#[derive(Debug, Clone, Copy)]
pub struct Router;

impl Handler for Router {
    fn handle(&self, request: Request<Body>) -> impl Future<Output = Response<Body>> + Send {
        let format = Format::from_accept(request.headers());
        future::ready(respond(request.method(), request.uri().path(), format))
    }
}

/// The page of a method and a path in `format`; an unknown path, or a method other than `GET`, gets the not found page.
fn respond(method: &Method, path: &str, format: Format) -> Response<Body> {
    match (method, path) {
        (&Method::GET, HOME_PATH) => pages::render(&pages::home(), format),
        (&Method::GET, HEALTH_PATH) => pages::render(&pages::health(), format),
        _ => pages::render(&pages::not_found(), format),
    }
}
