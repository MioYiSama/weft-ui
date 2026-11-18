//! Language server for `.weft` files.
//!
//! Features: diagnostics, hover, go to definition, completion and
//! formatting. Every request re-analyzes the file's import closure from the
//! open documents (and the file system for files that are not open); the
//! files are small, so there is no incremental state to keep consistent.

mod completion;
mod line_index;
mod server;

use std::error::Error;

use lsp_server::{Connection, Message};
use lsp_types::{
    CompletionOptions, HoverProviderCapability, InitializeParams, OneOf, ServerCapabilities,
    ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind,
};

use server::Server;

pub type Result<T, E = Box<dyn Error + Send + Sync>> = std::result::Result<T, E>;

/// Serves the protocol over stdin / stdout until the client exits.
///
/// # Errors
///
/// Returns an error if the connection fails or the client violates the
/// protocol.
pub fn run_stdio() -> Result<()> {
    let (connection, io_threads) = Connection::stdio();
    serve(&connection)?;
    drop(connection);
    io_threads.join()?;
    Ok(())
}

/// Serves the protocol on `connection` until the client shuts the server down.
///
/// # Errors
///
/// Returns an error if the connection fails or the client violates the
/// protocol.
pub fn serve(connection: &Connection) -> Result<()> {
    let capabilities = ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        definition_provider: Some(OneOf::Left(true)),
        completion_provider: Some(CompletionOptions {
            trigger_characters: Some(vec![".".to_owned()]),
            ..CompletionOptions::default()
        }),
        document_formatting_provider: Some(OneOf::Left(true)),
        ..ServerCapabilities::default()
    };
    let initialize = serde_json::json!({
        "capabilities": capabilities,
        "serverInfo": ServerInfo {
            name: "weft".to_owned(),
            version: Some(env!("CARGO_PKG_VERSION").to_owned()),
        },
    });
    let (id, params) = connection.initialize_start()?;
    connection.initialize_finish(id, initialize)?;
    let params: InitializeParams = serde_json::from_value(params)?;
    let snippets = params
        .capabilities
        .text_document
        .and_then(|t| t.completion)
        .and_then(|c| c.completion_item)
        .and_then(|i| i.snippet_support)
        .unwrap_or(false);

    let mut server = Server::new(snippets);
    for message in &connection.receiver {
        match message {
            Message::Request(request) => {
                if connection.handle_shutdown(&request)? {
                    return Ok(());
                }
                let response = server.request(request);
                connection.sender.send(Message::Response(response))?;
            }
            Message::Notification(notification) => {
                for out in server.notification(notification) {
                    connection.sender.send(Message::Notification(out))?;
                }
            }
            Message::Response(_) => {}
        }
    }
    Ok(())
}
