use anyhow::Context;
use build_html::{HtmlContainer, HtmlElement, HtmlTag};
use http::StatusCode;
use serde::Serialize;
use tiny_database::Database;

use super::{Page, SITE_VERSION, WithOptional};
use crate::library::Book;

/// `GET /health`: whether the server can read its database, which version runs and how many books it serves.
pub fn health(database: &Database) -> Health {
    let book_count = count_books(database);

    // the failure is the answer of the health check, the cause goes to the log
    let (status, books) = match book_count {
        Ok(count) => (HealthStatus::Pass, Some(count)),
        Err(error) => {
            log::error!("{error:#}");
            (HealthStatus::Fail, None)
        }
    };

    Health {
        status,
        version: SITE_VERSION,
        books,
    }
}

/// The state of the server, in JSON `{"status":"pass","version":"<version>","books":9}`; `{"status":"fail",...,"books":null}` with 503 if the database cannot be read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Health {
    status: HealthStatus,
    version: &'static str,
    books: Option<u64>,
}

impl Page for Health {
    fn title(&self) -> &'static str {
        "Health"
    }

    fn content(&self) -> HtmlElement {
        let status = format!("Status: {}", self.status.as_str());
        let version = format!("Version: {}", self.version);
        let books = self.books.map(|count| {
            HtmlElement::new(HtmlTag::ParagraphText).with_raw(format!("Books: {count}"))
        });

        HtmlElement::new(HtmlTag::Main)
            .with_header(2, "Health")
            .with_paragraph(status)
            .with_paragraph(version)
            .with_optional(books)
    }

    fn status(&self) -> StatusCode {
        match self.status {
            HealthStatus::Pass => StatusCode::OK,
            HealthStatus::Fail => StatusCode::SERVICE_UNAVAILABLE,
        }
    }
}

/// Whether the server can serve requests, named as in the IETF draft "Health Check Response Format for HTTP APIs".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// The server works.
    Pass,
    /// The server runs, but cannot read its database.
    Fail,
}

impl HealthStatus {
    /// The name of the status, the same as in JSON.
    fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
        }
    }
}

/// The number of books in `database`, without reading them.
fn count_books(database: &Database) -> anyhow::Result<u64> {
    database.count::<Book>().context("cannot count the books")
}
