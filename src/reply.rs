//! What an endpoint gives back, turned into the response of the request: a page, a file, not found or a server error.

use http::Response;
use tiny_http_server::Body;

use crate::pages::{self, Format, Page};

/// What an endpoint gives back: a page, a ready response (a file), `None` when what was asked for does not exist, or an error.
pub trait Reply {
    /// The response of the reply in `format`.
    fn into_response(self, format: Format) -> Response<Body>;
}

impl<P: Page> Reply for P {
    fn into_response(self, format: Format) -> Response<Body> {
        pages::render(&self, format)
    }
}

impl Reply for Response<Body> {
    fn into_response(self, _format: Format) -> Response<Body> {
        self
    }
}

impl<T: Reply> Reply for Option<T> {
    fn into_response(self, format: Format) -> Response<Body> {
        match self {
            Some(reply) => reply.into_response(format),
            None => pages::not_found().into_response(format),
        }
    }
}

impl<T: Reply> Reply for anyhow::Result<T> {
    /// Every failed request ends here: the error is logged, the client gets the internal error page.
    fn into_response(self, format: Format) -> Response<Body> {
        match self {
            Ok(reply) => reply.into_response(format),
            Err(error) => {
                log::error!("{error:#}");
                pages::internal_error().into_response(format)
            }
        }
    }
}
