use lsp_server::{Connection, ErrorCode, Message, Response};
use lsp_types::{ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // TODO: logging setup

    let (connection, io_threads) = Connection::stdio();
    let capabilities = ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        ..ServerCapabilities::default()
    };

    connection.initialize(serde_json::to_value(capabilities)?)?;
    main_loop(&connection)?;
    drop(connection);
    io_threads.join()?;
    Ok(())
}

fn main_loop(connection: &Connection) -> Result<(), Box<dyn Error + Send + Sync>> {
    for message in &connection.receiver {
        match message {
            Message::Request(request) => {
                if connection.handle_shutdown(&request)? {
                    break;
                }
                //TODO request handling
                let response = Response::new_err(
                    request.id,
                    ErrorCode::MethodNotFound as i32,
                    format!("unsupported request `{}`", request.method),
                );
                connection.sender.send(Message::Response(response))?;
            }
            // TODO: notification handling
            Message::Notification(_) | Message::Response(_) => {}
        }
    }
    Ok(())
}
