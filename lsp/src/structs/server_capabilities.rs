use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#serverCapabilities
pub struct ServerCapabilities {
    position_encoding: PositionEncodingKind,
    text_document_sync: Option<TextDocumentSyncOptions>,
}

impl Default for ServerCapabilities {
    fn default() -> Self {
        Self {
            position_encoding: PositionEncodingKind::UTF8,
            text_document_sync: Some(TextDocumentSyncOptions {
                open_close: Some(true),
                change: TextDocumentSyncKind::Full,
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#positionEncodingKind
pub enum PositionEncodingKind {
    #[serde(rename = "utf-8")]
    UTF8,
    #[serde(rename = "utf-16")]
    UTF16,
    #[serde(rename = "utf-32")]
    UTF32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#textDocumentSyncOptions
pub struct TextDocumentSyncOptions {
    /// Open and close notifications are sent to the server. If omitted open
    /// close notifications should not be sent.
    open_close: Option<bool>,

    #[serde(default)]
    change: TextDocumentSyncKind,
}

#[derive(Debug, Clone, Serialize_repr, Deserialize_repr, Default)]
#[repr(u8)]
/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#textDocumentSyncKind
pub enum TextDocumentSyncKind {
    /// Documents should not be synced at all.
    #[default]
    None = 0,

    /// Documents are synced by always sending the full content
    /// of the document.
    Full = 1,

    /// Documents are synced by sending the full content on open.
    /// After that only incremental updates to the document are sent.
    Incremental = 2,
}
