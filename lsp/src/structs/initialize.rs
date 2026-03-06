use serde::{Deserialize, Serialize};

use crate::structs::server_capabilities::ServerCapabilities;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#initializeParams
pub struct InitializeParams {
    /// The process Id of the parent process that started the server. Is null if
    /// the process has not been started by another process. If the parent
    /// process is not alive then the server should exit (see exit notification)
    /// its process.
    #[serde(rename = "processId")]
    process_id: Option<i32>,

    // /// The capabilities provided by the client (editor or tool)
    // capabilities: ClientCapabilities,
    /// The workspace folders configured in the client when the server starts.
    /// This property is only available if the client supports workspace folders.
    /// It can be `null` if the client supports workspace folders but none are
    /// configured.
    #[serde(rename = "workspaceFolders")]
    #[serde(default)]
    workspace_folders: Option<Vec<WorkspaceFolder>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    /// The capabilities the language server provides.
    pub capabilities: ServerCapabilities,

    /// Information about the server.
    #[serde(rename = "serverInfo")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_info: Option<ServerInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// The name of the server as defined by the server.
    pub name: String,

    /// The server's version as defined by the server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

impl Default for ServerInfo {
    fn default() -> Self {
        Self {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceFolder {
    /// The associated URI for this workspace folder.
    uri: String,
    /// The name of the workspace folder. Used to refer to this
    /// workspace folder in the user interface.
    name: String,
}
