use std::fs;
use std::path::Path;

use anyhow::Context;
use http::Response;
use http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC};
use tiny_database::Database;
use tiny_http_server::Body;

use crate::library::Book;

const EPUB_CONTENT_TYPE: &str = "application/epub+zip";
/// The bytes percent-encoded in an RFC 5987 value: every byte but the `attr-char`s.
const NOT_ATTR_CHAR: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'!')
    .remove(b'#')
    .remove(b'$')
    .remove(b'&')
    .remove(b'+')
    .remove(b'-')
    .remove(b'.')
    .remove(b'^')
    .remove(b'_')
    .remove(b'`')
    .remove(b'|')
    .remove(b'~');

/// `GET /download?id=<id>`: the EPUB file of the book `id` as an attachment under its own file name, or `None` if there is no such book.
///
/// # Errors
///
/// Returns an error if the book cannot be read from the database, or its file from the library directory.
pub fn download(
    database: &Database,
    library_dir: &Path,
    id: u64,
) -> anyhow::Result<Option<Response<Body>>> {
    let book = database
        .get::<Book>(id)
        .with_context(|| format!("cannot read the book {id}"))?;

    let Some(book) = book else {
        return Ok(None);
    };

    let path = library_dir.join(book.file_name());
    let epub = fs::read(&path)
        .with_context(|| format!("cannot read the file of the book {id}: {}", path.display()))?;

    let response = Response::builder()
        .header(CONTENT_TYPE, EPUB_CONTENT_TYPE)
        .header(CONTENT_DISPOSITION, content_disposition(book.file_name()))
        .body(Body::from(epub))
        .unwrap(); // cannot fail, the content disposition is visible ASCII only

    Ok(Some(response))
}

/// The `Content-Disposition` of an attachment named `file_name` (RFC 6266): an ASCII `filename` for old clients, and the exact name UTF-8 encoded in `filename*`.
fn content_disposition(file_name: &str) -> String {
    let ascii_name: String = file_name
        .chars()
        .map(|character| {
            let is_plain = character == ' ' || character.is_ascii_graphic();
            let is_quoting = character == '"' || character == '\\';
            if is_plain && !is_quoting {
                character
            } else {
                '_'
            }
        })
        .collect();

    let encoded_name = percent_encoding::utf8_percent_encode(file_name, NOT_ATTR_CHAR);

    format!("attachment; filename=\"{ascii_name}\"; filename*=UTF-8''{encoded_name}")
}

#[cfg(test)]
mod tests {
    use http::HeaderValue;

    use super::*;

    #[test]
    fn an_accented_name_is_kept_in_utf8_with_an_ascii_fallback() {
        let disposition = content_disposition("Mikszáth Kálmán - Szent Péter esernyője.epub");

        assert_eq!(
            disposition,
            "attachment; filename=\"Miksz_th K_lm_n - Szent P_ter eserny_je.epub\"; \
             filename*=UTF-8''Miksz%C3%A1th%20K%C3%A1lm%C3%A1n%20-%20Szent%20P%C3%A9ter%20eserny%C5%91je.epub"
        );
    }

    #[test]
    fn any_file_name_gives_a_valid_header() {
        // a file name on Linux can hold quotes, backslashes and line breaks, none may break the header
        let disposition = content_disposition("a\"b\\c\r\nSet-Cookie: x\t.epub");

        assert!(HeaderValue::from_str(&disposition).is_ok(), "{disposition}");
        assert!(disposition.starts_with("attachment; filename=\"a_b_c__Set-Cookie: x_.epub\";"));
    }
}
