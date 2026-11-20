use std::convert::Infallible;

use anyhow::Context;
use tiny_http_server::Server;

use super::Arguments;
use crate::library;
use crate::router::Router;

/// Indexes the `--library` directory, then starts the server on the `--listen` address and serves until it stops with an error.
pub fn run(arguments: &Arguments) -> anyhow::Result<Infallible> {
    log::info!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    let database = library::index(&arguments.library, &arguments.database)?;
    let handler = Router::new(database, arguments.library.clone());
    let server = Server::bind(arguments.listen, handler)?;

    log::info!("server address: http://{}", arguments.listen);

    server.run().context("the server stopped")
}
