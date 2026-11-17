use anyhow::Context;
use tiny_http_server::Server;

use super::Arguments;
use crate::library;
use crate::router::Router;

/// Indexes the `--library` directory, then starts the server on the `--listen` address and serves until the process exits.
pub fn run(arguments: &Arguments) -> anyhow::Result<()> {
    log::info!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    let database = library::index(&arguments.library, &arguments.database)?;

    let server = Server::bind(arguments.listen, Router::new(database))
        .with_context(|| format!("cannot listen on {}", arguments.listen))?;

    log::info!("server address: http://{}", arguments.listen);

    server.run().context("the server stopped")
}
