use crate::diagnostics;
use clerr::{Code, Properties, Report};
use lsp_server::{Connection, IoThreads, Message, Notification, Response};
use lsp_types::notification::{
    DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument,
    Notification as NotificationType, PublishDiagnostics,
};
use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    PublishDiagnosticsParams, ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind,
    Uri,
};
use std::fmt::Display;

/// The ProtoPacket language server.
///
/// # Note
/// - It reports the errors of each open schema file, as the file is edited.
pub struct LanguageServer {
    connection: Connection,
}

impl LanguageServer {
    //! Constants

    /// The code of the response to a request for a method the server doesn't have.
    const METHOD_NOT_FOUND: i32 = -32601;
}

impl LanguageServer {
    //! Construction

    /// Creates the [LanguageServer].
    const fn new(connection: Connection) -> Self {
        Self { connection }
    }
}

impl LanguageServer {
    //! Serve

    /// Serves the client on the standard input & output, until the client ends the session.
    pub fn serve_stdio() -> Result<(), Report> {
        let (connection, io_threads): (Connection, IoThreads) = Connection::stdio();
        Self::new(connection).serve()?;
        io_threads
            .join()
            .map_err(|error| report("L_IO", "could not read or write a message", error))
    }

    /// Serves the client of the connection, until the client ends the session.
    ///
    /// # Note
    /// - The client sends the whole text of a file on each change.
    fn serve(self) -> Result<(), Report> {
        let capabilities: ServerCapabilities = ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
            ..ServerCapabilities::default()
        };
        let capabilities: serde_json::Value = serde_json::to_value(capabilities)
            .map_err(|error| report("L_PROTOCOL", "could not write a message", error))?;
        self.connection.initialize(capabilities).map_err(protocol)?;
        for message in &self.connection.receiver {
            match message {
                Message::Request(request) => {
                    if self
                        .connection
                        .handle_shutdown(&request)
                        .map_err(protocol)?
                    {
                        return Ok(());
                    }
                    let response: Response = Response::new_err(
                        request.id,
                        Self::METHOD_NOT_FOUND,
                        format!("unknown method: {}", request.method),
                    );
                    self.send(Message::Response(response))?;
                }
                Message::Notification(notification) => self.notified(notification)?,
                Message::Response(_) => {}
            }
        }
        Ok(())
    }

    /// Handles the `notification` of the client.
    ///
    /// # Note
    /// - A closed file has its diagnostics cleared.
    /// - Any other notification is ignored, as the protocol allows.
    fn notified(&self, notification: Notification) -> Result<(), Report> {
        let params: serde_json::Value = notification.params;
        match notification.method.as_str() {
            DidOpenTextDocument::METHOD => {
                let params: DidOpenTextDocumentParams = parse(params)?;
                self.publish(params.text_document.uri, &params.text_document.text)
            }
            DidChangeTextDocument::METHOD => {
                let params: DidChangeTextDocumentParams = parse(params)?;
                match params.content_changes.last() {
                    Some(change) => self.publish(params.text_document.uri, &change.text),
                    None => Ok(()),
                }
            }
            DidCloseTextDocument::METHOD => {
                let params: DidCloseTextDocumentParams = parse(params)?;
                self.send_diagnostics(params.text_document.uri, Vec::default())
            }
            _ => Ok(()),
        }
    }

    /// Publishes the diagnostics of the schema file at the `uri` with the `source`.
    fn publish(&self, uri: Uri, source: &str) -> Result<(), Report> {
        self.send_diagnostics(uri, diagnostics(source))
    }

    /// Sends the `diagnostics` of the file at the `uri` to the client.
    fn send_diagnostics(
        &self,
        uri: Uri,
        diagnostics: Vec<lsp_types::Diagnostic>,
    ) -> Result<(), Report> {
        let params: PublishDiagnosticsParams =
            PublishDiagnosticsParams::new(uri, diagnostics, None);
        let notification: Notification =
            Notification::new(PublishDiagnostics::METHOD.to_string(), params);
        self.send(Message::Notification(notification))
    }

    /// Sends the `message` to the client.
    fn send(&self, message: Message) -> Result<(), Report> {
        self.connection
            .sender
            .send(message)
            .map_err(|error| report("L_IO", "could not read or write a message", error))
    }
}

/// Parses the `params` of a notification.
fn parse<P>(params: serde_json::Value) -> Result<P, Report>
where
    P: serde::de::DeserializeOwned,
{
    serde_json::from_value(params)
        .map_err(|error| report("L_PROTOCOL", "the client broke the protocol", error))
}

/// Creates the report for the protocol `error`.
fn protocol<E>(error: E) -> Report
where
    E: Display,
{
    report("L_PROTOCOL", "the client broke the protocol", error)
}

/// Creates the report with the `code` & the `message` for the `error`.
fn report<E>(code: &str, message: &str, error: E) -> Report
where
    E: Display,
{
    let properties: Properties = Properties::default().with_property("error", error.to_string());
    Report::from(Code::error(code, message)).with_entry(properties)
}
