use anyhow::Context;
use tiny_http_server::Server;

use super::Arguments;
use crate::router::Router;

/// Starts the server on the `--listen` address and serves until the process exits.
pub fn run(arguments: &Arguments) -> anyhow::Result<()> {
    log::info!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    let server = Server::bind(arguments.listen, Router)
        .with_context(|| format!("cannot listen on {}", arguments.listen))?;

    log::info!("server address: http://{}", arguments.listen);

    server.run().context("the server stopped")
}
