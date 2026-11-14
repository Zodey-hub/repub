use http::header::CONTENT_TYPE;
use http::{HeaderValue, Response};
use tiny_http_server::Body;

const TEXT_CONTENT_TYPE: &str = "text/plain; charset=utf-8";

/// `GET /health`: whether the server is alive and which version runs, the body is the version as plain text.
pub fn health() -> Response<Body> {
    let version = Body::from_static(env!("CARGO_PKG_VERSION").as_bytes());

    let mut response = Response::new(version);
    response
        .headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static(TEXT_CONTENT_TYPE));

    response
}
