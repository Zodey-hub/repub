//! The single routes file: every path of the site is paired with its page here.

use std::future;

use http::{Method, Request, Response};
use tiny_http_server::{Body, Handler};

use crate::pages;

/// The path of the home page.
pub const HOME_PATH: &str = "/";
/// The path of the health check.
pub const HEALTH_PATH: &str = "/health";

/// The handler of the server: answers every request with the page of its method and path.
#[derive(Debug, Clone, Copy)]
pub struct Router;

impl Handler for Router {
    fn handle(&self, request: Request<Body>) -> impl Future<Output = Response<Body>> + Send {
        future::ready(respond(request.method(), request.uri().path()))
    }
}

/// The page of a method and a path; an unknown path, or a method other than `GET`, gets the not found page.
fn respond(method: &Method, path: &str) -> Response<Body> {
    match (method, path) {
        (&Method::GET, HOME_PATH) => pages::home(),
        (&Method::GET, HEALTH_PATH) => pages::health(),
        _ => pages::not_found(),
    }
}
