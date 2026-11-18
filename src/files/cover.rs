use anyhow::Context;
use http::Response;
use http::header::{CONTENT_TYPE, X_CONTENT_TYPE_OPTIONS};
use tiny_database::Database;
use tiny_http_server::Body;

use crate::library::Cover;

/// `GET /cover?id=<id>`: the cover image of the book `id`, or `None` if there is no such book or it has no cover.
///
/// # Errors
///
/// Returns an error if the cover cannot be read from the database.
pub fn cover(database: &Database, id: u64) -> anyhow::Result<Option<Response<Body>>> {
    let cover = database
        .get::<Cover>(id)
        .with_context(|| format!("cannot read the cover of the book {id}"))?;

    let response = cover.map(|cover| {
        Response::builder()
            .header(CONTENT_TYPE, cover.mime_type())
            .header(X_CONTENT_TYPE_OPTIONS, "nosniff") // the type comes from the EPUB file, the browser must not guess another one from the bytes
            .body(Body::from(cover.into_image()))
            .unwrap() // cannot fail, the type is one of the allowed cover types
    });

    Ok(response)
}
