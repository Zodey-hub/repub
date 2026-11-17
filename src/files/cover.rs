use http::Response;
use http::header::{CONTENT_TYPE, X_CONTENT_TYPE_OPTIONS};
use tiny_database::Database;
use tiny_http_server::Body;

use crate::library::Cover;
use crate::pages::{self, Format};

/// `GET /cover?id=<id>`: the cover image of the book `id`; the not found page if there is no such cover.
pub fn cover(database: &Database, query: Option<&str>, format: Format) -> Response<Body> {
    let Some(id) = book_id(query) else {
        return pages::render(&pages::not_found(), format);
    };

    let cover = database.get::<Cover>(id);

    match cover {
        Ok(Some(cover)) => Response::builder()
            .header(CONTENT_TYPE, cover.mime_type())
            .header(X_CONTENT_TYPE_OPTIONS, "nosniff")
            .body(Body::from(cover.into_image()))
            .unwrap(), // cannot fail, the type is one of the allowed cover types
        Ok(None) => pages::render(&pages::not_found(), format),
        Err(error) => {
            log::error!("cannot read the cover of the book {id}: {error:#}");
            pages::render(&pages::internal_error(), format)
        }
    }
}

/// The value of the `id` parameter in `query`, or `None` if it is missing or not a number.
fn book_id(query: Option<&str>) -> Option<u64> {
    query?
        .split('&')
        .find_map(|parameter| parameter.strip_prefix("id="))?
        .parse()
        .ok()
}
