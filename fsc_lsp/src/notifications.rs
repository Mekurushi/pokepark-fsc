use crate::documents::{ChangeError, Documents};
use lsp_types::notification::{
    DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, Notification as _,
};
use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
};

pub(crate) fn handle_notification(
    notification: lsp_server::Notification,
    documents: &mut Documents,
) -> Result<(), NotificationError> {
    match notification.method.as_str() {
        DidOpenTextDocument::METHOD => {
            let params = serde_json::from_value::<DidOpenTextDocumentParams>(notification.params)?;
            let uri = params.text_document.uri.clone();
            let document = documents.open(params.text_document);
            log::debug!(
                "opened {uri:?} at version {} ({} bytes)",
                document.version(),
                document.text().len(),
            );
        }
        DidChangeTextDocument::METHOD => {
            let params =
                serde_json::from_value::<DidChangeTextDocumentParams>(notification.params)?;
            let document = documents.change(params)?;
            log::debug!(
                "updated document to version {} ({} bytes)",
                document.version(),
                document.text().len(),
            );
        }
        DidCloseTextDocument::METHOD => {
            let params = serde_json::from_value::<DidCloseTextDocumentParams>(notification.params)?;
            if documents.close(&params.text_document.uri) {
                log::debug!("closed {:?}", params.text_document.uri);
            }
        }
        _ => {}
    }

    Ok(())
}

#[derive(Debug)]
pub(crate) enum NotificationError {
    InvalidParams(serde_json::Error),
    DocumentChange(ChangeError),
}

impl std::fmt::Display for NotificationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidParams(error) => write!(formatter, "invalid parameters: {error}"),
            Self::DocumentChange(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for NotificationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidParams(error) => Some(error),
            Self::DocumentChange(error) => Some(error),
        }
    }
}

impl From<serde_json::Error> for NotificationError {
    fn from(error: serde_json::Error) -> Self {
        Self::InvalidParams(error)
    }
}

impl From<ChangeError> for NotificationError {
    fn from(error: ChangeError) -> Self {
        Self::DocumentChange(error)
    }
}
