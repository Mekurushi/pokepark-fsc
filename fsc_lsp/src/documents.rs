use lsp_types::{DidChangeTextDocumentParams, TextDocumentItem, Uri};
use std::collections::HashMap;
use std::collections::hash_map::Entry;

pub(crate) struct Document {
    text: String,
    version: i32,
}

impl Document {
    pub(crate) fn text(&self) -> &str {
        &self.text
    }

    pub(crate) const fn version(&self) -> i32 {
        self.version
    }
}

#[allow(clippy::mutable_key_type)]
pub(crate) struct Documents {
    entries: HashMap<Uri, Document>,
}

impl Documents {
    pub(crate) fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub(crate) fn open(&mut self, item: TextDocumentItem) -> &Document {
        let document = Document {
            text: item.text,
            version: item.version,
        };
        match self.entries.entry(item.uri) {
            Entry::Occupied(mut entry) => {
                entry.insert(document);
                entry.into_mut()
            }
            Entry::Vacant(entry) => entry.insert(document),
        }
    }

    pub(crate) fn change(
        &mut self,
        params: DidChangeTextDocumentParams,
    ) -> Result<&Document, ChangeError> {
        let identifier = params.text_document;
        let document = self
            .entries
            .get_mut(&identifier.uri)
            .ok_or(ChangeError::DocumentNotOpen)?;

        if identifier.version <= document.version {
            return Err(ChangeError::StaleVersion {
                current: document.version,
                received: identifier.version,
            });
        }

        if params
            .content_changes
            .iter()
            .any(|change| change.range.is_some() || change.range_length.is_some())
        {
            return Err(ChangeError::UnexpectedIncrementalChange);
        }

        // we need the last change while using FullSync
        if let Some(change) = params.content_changes.last() {
            document.text.clone_from(&change.text);
        }
        document.version = identifier.version;
        Ok(document)
    }

    pub(crate) fn close(&mut self, uri: &Uri) -> bool {
        self.entries.remove(uri).is_some()
    }
}

#[derive(Debug)]
pub(crate) enum ChangeError {
    DocumentNotOpen,
    StaleVersion { current: i32, received: i32 },
    UnexpectedIncrementalChange,
}

impl std::fmt::Display for ChangeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DocumentNotOpen => formatter.write_str("document is not open"),
            Self::StaleVersion { current, received } => write!(
                formatter,
                "version {received} is not newer than current version {current}"
            ),
            Self::UnexpectedIncrementalChange => {
                formatter.write_str("received an incremental change while using full sync")
            }
        }
    }
}

impl std::error::Error for ChangeError {}
