use build_html::{HtmlContainer, HtmlElement, HtmlTag};
use serde::Serialize;

use super::{Page, SITE_VERSION};

/// `GET /health`: whether the server is alive and which version runs.
pub fn health() -> Health {
    Health {
        status: HealthStatus::Pass,
        version: SITE_VERSION,
    }
}

/// The state of the server, in JSON `{"status":"pass","version":"<version>"}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Health {
    status: HealthStatus,
    version: &'static str,
}

impl Page for Health {
    fn title(&self) -> &'static str {
        "Health"
    }

    fn content(&self) -> HtmlElement {
        let status = format!("Status: {}", self.status.as_str());
        let version = format!("Version: {}", self.version);

        HtmlElement::new(HtmlTag::Main)
            .with_header(2, "Health")
            .with_paragraph(status)
            .with_paragraph(version)
    }
}

/// Whether the server can serve requests, named as in the IETF draft "Health Check Response Format for HTTP APIs".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// The server works.
    Pass,
}

impl HealthStatus {
    /// The name of the status, the same as in JSON.
    fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
        }
    }
}
