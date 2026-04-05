use anyhow::Result;
use paste::paste;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{Header, structs::initialize::InitializeParams};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    Int(i64),
    Str(String),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Message {
    Request(RequestMessage),
    Notification(NotificationMessage),
}

#[derive(Debug, Clone, Deserialize)]
pub struct RequestMessage {
    /// The request id.
    pub id: RequestId,

    #[serde(flatten)]
    pub method: RequestMethod,
}

impl RequestMessage {
    pub fn reply_with<T>(&self, response: ResponsePayload<T>) -> ResponseMessage<T> {
        ResponseMessage {
            jsonrpc: "2.0".into(),
            id: self.id.clone(),
            payload: response,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct NotificationMessage {
    pub method: String,
    pub params: Value,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidOpenTextDocumentParams {
    pub text_document: TextDocumentItem,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentItem {
    pub uri: String,
    pub version: i32,
    pub text: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidChangeTextDocumentParams {
    pub text_document: VersionedTextDocumentIdentifier,
    pub content_changes: Vec<TextDocumentContentChangeEvent>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionedTextDocumentIdentifier {
    pub uri: String,
    pub version: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TextDocumentContentChangeEvent {
    pub text: String,
}

macro_rules! lsp_request_methods {
    (
        $( $Variant:ident => $method_lit:literal ( $Params:ty ) ),+ $(,)?
    ) => {
        paste! {
            #[derive(Debug, Clone, Serialize, Deserialize)]
            #[serde(untagged)]
            pub enum RequestMethod {
                $(
                    $Variant {
                        method: [<$Variant Method>],
                        params: $Params,
                    },
                )+
                Unknown {
                    method: String,
                    #[serde(default)]
                    params: Option<serde_json::Value>,
                },
            }

            $(
                #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
                pub enum [<$Variant Method>] {
                    #[serde(rename = $method_lit)]
                    Value,
                }
            )+
        }
    };
}

lsp_request_methods! {
    Initialize => "initialize"(InitializeParams),
    Completion => "textDocument/completion"(CompletionParams),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionParams {
    pub text_document: TextDocumentIdentifier,
    pub position: Position,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentIdentifier {
    pub uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub line: usize,
    pub character: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResponseId {
    Int(i64),
    Str(String),
    /// When the JSON has `"id": null`
    Null,
}

/// `error` object (JSON-RPC/LSP style)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseError {
    pub code: ErrorCodes,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Error Codes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorCodes {
    ParseError = -32700,
    InvalidRequest = -32600,
    MethodNotFound = -32601,
    InvalidParams = -32602,
    InternalError = -32603,
}

/// Exactly one of `result` or `error`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResponsePayload<T> {
    Success { result: T },
    Failure { error: ResponseError },
}

#[allow(unused)]
impl<T> ResponsePayload<T> {
    pub fn success(result: T) -> Self {
        Self::Success { result }
    }

    pub fn failure(error: impl Into<ResponseError>) -> Self {
        Self::Failure {
            error: error.into(),
        }
    }
}

/// Full response message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseMessage<T> {
    pub jsonrpc: String,
    pub id: RequestId,

    /// JSON ends up as `{  "result": ... }` or `{  "error": ... }`
    #[serde(flatten)]
    pub payload: ResponsePayload<T>,
}

impl<T: Serialize> ResponseMessage<T> {
    pub fn as_message(self) -> Result<String> {
        Header::new(self)?.to_string()
    }
}
