pub mod logging;
pub mod structs;
use std::{
    collections::HashMap,
    io::BufRead,
    sync::{LazyLock, Mutex},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sf_formula_parser::{
    ValidationError, token::FunctionName, validate_expression_detailed_with_source,
};
use tracing::{debug, warn};

use crate::structs::{
    initialize::{InitializeResult, ServerInfo},
    request::*,
    server_capabilities::ServerCapabilities,
};

static OPEN_DOCUMENTS: LazyLock<Mutex<HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Debug)]
pub struct Header<Body> {
    pub body: Body,
}

impl<Body: Serialize> Header<Body> {
    pub fn new(obj: Body) -> Result<Self> {
        Ok(Self { body: obj })
    }

    pub fn to_string(&self) -> Result<String> {
        let s = serde_json::to_string(&self.body)?;
        Ok(format!("Content-Length: {}\r\n\r\n{s}", s.len()))
    }
}

pub fn parse_message(mut inp: impl BufRead) -> Result<String> {
    let mut buf = Vec::new();
    let len_def = inp
        .read_until(b'\n', &mut buf)
        .context("failed len def buf read")?;

    // No need to trim whitespace because the spec specifies `: `
    // `len_def - 2` because the end should have an extra `\r\n`
    // See https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#headerPart
    let s = &buf["Content-Length: ".len()..len_def - 2];

    // `s` should only contain ascii numerals in the buffer.
    let size = str::from_utf8(s)
        .context("failed to convert numerals to utf8")?
        .parse::<usize>()
        .context("failed to parse utf8 to usize")?;
    debug!("Size: {size:?}");

    // Now we need to read the extra `\r\n`
    inp.consume(2);

    // Finally, we can read the json
    buf.clear();
    buf.reserve_exact(size);
    // SAFETY: Buffer has already reserved exactly `size` bytes
    unsafe { buf.set_len(size) };

    inp.read_exact(&mut buf)
        .context("failed to read exact to buf")?;

    // Doesn't reallocate, so this mostly just ensures that it contains valid utf8
    let s = String::from_utf8(buf).context("Failed to parse the buffer as utf8")?;

    Ok(s)
}

pub fn handle_notification(notification: NotificationMessage) -> Result<Vec<String>> {
    match notification.method.as_str() {
        "textDocument/didOpen" => {
            let params = serde_json::from_value::<DidOpenTextDocumentParams>(notification.params)
                .context("failed to parse didOpen params")?;

            cache_document(
                params.text_document.uri.clone(),
                params.text_document.text.clone(),
            );

            let diagnostics =
                build_syntax_diagnostics(&params.text_document.text, &params.text_document.uri);

            Ok(vec![publish_diagnostics_message(
                params.text_document.uri,
                Some(params.text_document.version),
                diagnostics,
            )?])
        }
        "textDocument/didChange" => {
            let params = serde_json::from_value::<DidChangeTextDocumentParams>(notification.params)
                .context("failed to parse didChange params")?;

            let latest_change = params
                .content_changes
                .last()
                .context("didChange had no content changes")?;

            let diagnostics =
                build_syntax_diagnostics(&latest_change.text, &params.text_document.uri);

            cache_document(params.text_document.uri.clone(), latest_change.text.clone());

            Ok(vec![publish_diagnostics_message(
                params.text_document.uri,
                Some(params.text_document.version),
                diagnostics,
            )?])
        }
        _ => Ok(Vec::new()),
    }
}

fn build_syntax_diagnostics(text: &str, source_name: &str) -> Vec<Diagnostic> {
    match validate_expression_detailed_with_source(text, source_name) {
        Ok(()) => Vec::new(),
        Err(err) => {
            let message = if err.details.is_empty() {
                err.message.clone()
            } else {
                format!("{}\n{}", err.message, err.details.join("\n"))
            };

            let suggestions = err
                .suggestions
                .iter()
                .filter_map(|suggestion| {
                    let replacement = suggestion.replacement.clone()?;
                    let span = suggestion.span.clone()?;

                    let (line, character) = offset_to_position(text, span.start);
                    let (end_line, end_character) = offset_to_position(text, span.end);

                    Some(DiagnosticSuggestionData {
                        id: suggestion.id.to_string(),
                        title: suggestion.message.clone(),
                        replacement,
                        range: Range {
                            start: Position { line, character },
                            end: Position {
                                line: end_line,
                                character: end_character,
                            },
                        },
                    })
                })
                .collect::<Vec<_>>();

            vec![Diagnostic {
                range: parser_error_range(&err),
                severity: Some(1),
                code: Some("E0001".to_string()),
                source: Some(env!("CARGO_PKG_NAME").to_string()),
                message,
                data: (!suggestions.is_empty())
                    .then_some(serde_json::to_value(DiagnosticData { suggestions }).unwrap()),
            }]
        }
    }
}

fn offset_to_position(input: &str, offset: usize) -> (usize, usize) {
    let clamped = offset.min(input.len());
    let mut line = 0usize;
    let mut line_start = 0usize;

    for (idx, ch) in input.char_indices() {
        if idx >= clamped {
            break;
        }

        if ch == '\n' {
            line += 1;
            line_start = idx + ch.len_utf8();
        }
    }

    (line, clamped.saturating_sub(line_start))
}

fn parser_error_range(err: &ValidationError) -> Range {
    Range {
        start: Position {
            line: err.line,
            character: err.column,
        },
        end: Position {
            line: err.end_line,
            character: err.end_column,
        },
    }
}

fn publish_diagnostics_message(
    uri: String,
    version: Option<i32>,
    diagnostics: Vec<Diagnostic>,
) -> Result<String> {
    Header::new(NotificationOutput {
        jsonrpc: "2.0",
        method: "textDocument/publishDiagnostics",
        params: PublishDiagnosticsParams {
            uri,
            version,
            diagnostics,
        },
    })?
    .to_string()
}

pub fn handle_request(request: RequestMessage) -> Result<String> {
    let response = match &request.method {
        RequestMethod::Initialize {
            method: _method,
            params: _params,
        } => request.reply_with(ResponsePayload::success(serde_json::to_value(
            InitializeResult {
                capabilities: ServerCapabilities::default(),
                server_info: Some(ServerInfo::default()),
            },
        )?)),

        RequestMethod::Completion {
            method: _method,
            params,
        } => request.reply_with(ResponsePayload::success(serde_json::to_value(
            suggest_function_completions(params),
        )?)),

        RequestMethod::CodeAction {
            method: _method,
            params,
        } => request.reply_with(ResponsePayload::success(serde_json::to_value(
            suggest_code_actions(params),
        )?)),

        RequestMethod::Unknown { method, params } => {
            let message = format!("Got unknown request method: `{method}` with params: {params:?}");
            warn!(message);
            request.reply_with(ResponsePayload::failure(ResponseError {
                code: ErrorCodes::MethodNotFound,
                message: message,
                data: params.to_owned(),
            }))
        }
    };

    let response = response
        .as_message()
        .context("failed to serialize message")?;

    Ok(response)
}

fn cache_document(uri: String, text: String) {
    match OPEN_DOCUMENTS.lock() {
        Ok(mut docs) => {
            docs.insert(uri, text);
        }
        Err(err) => {
            warn!("failed to cache document contents: {err}");
        }
    }
}

fn suggest_function_completions(params: &CompletionParams) -> Vec<CompletionItem> {
    let doc = match OPEN_DOCUMENTS.lock() {
        Ok(docs) => docs.get(&params.text_document.uri).cloned(),
        Err(err) => {
            warn!("failed to access open document cache: {err}");
            None
        }
    };

    let Some(doc) = doc else {
        return Vec::new();
    };

    let Some(offset) = offset_for_position(&doc, params.position.line, params.position.character)
    else {
        return Vec::new();
    };

    let (prefix, prefix_start) = extract_identifier_prefix(&doc, offset);
    if !is_function_context(&doc, prefix_start) {
        return Vec::new();
    }

    let prefix_upper = prefix.to_ascii_uppercase();
    FunctionName::ALL
        .iter()
        .copied()
        .filter(|function_name| function_name.as_str().starts_with(&prefix_upper))
        .map(CompletionItem::function)
        .collect()
}

fn offset_for_position(text: &str, line: usize, character: usize) -> Option<usize> {
    let lines: Vec<&str> = text.split('\n').collect();
    let current_line = *lines.get(line)?;

    let mut offset = 0;
    for previous_line in lines.iter().take(line) {
        offset += previous_line.len() + 1;
    }

    Some(offset + character.min(current_line.len()))
}

fn extract_identifier_prefix(text: &str, cursor_offset: usize) -> (&str, usize) {
    let end = cursor_offset.min(text.len());
    let mut start = end;
    let bytes = text.as_bytes();

    while start > 0 {
        let b = bytes[start - 1];
        if b.is_ascii_alphanumeric() || b == b'_' {
            start -= 1;
            continue;
        }

        break;
    }

    (&text[start..end], start)
}

fn is_function_context(text: &str, ident_start: usize) -> bool {
    let bytes = text.as_bytes();
    let mut idx = ident_start.min(bytes.len());

    while idx > 0 && bytes[idx - 1].is_ascii_whitespace() {
        idx -= 1;
    }

    if idx == 0 {
        return true;
    }

    matches!(
        bytes[idx - 1],
        b'(' | b',' | b'=' | b'+' | b'-' | b'*' | b'/' | b'&' | b'|' | b'!' | b'<' | b'>' | b'^'
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NotificationOutput<T> {
    jsonrpc: &'static str,
    method: &'static str,
    params: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PublishDiagnosticsParams {
    uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<i32>,
    diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Diagnostic {
    range: Range,
    #[serde(skip_serializing_if = "Option::is_none")]
    severity: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DiagnosticSuggestionData {
    id: String,
    title: String,
    replacement: String,
    range: Range,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DiagnosticData {
    suggestions: Vec<DiagnosticSuggestionData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CodeAction {
    title: String,
    kind: String,
    diagnostics: Vec<crate::structs::request::CodeActionDiagnostic>,
    edit: WorkspaceEdit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkspaceEdit {
    changes: HashMap<String, Vec<TextEdit>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TextEdit {
    range: Range,
    new_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Range {
    start: Position,
    end: Position,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Position {
    line: usize,
    character: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CompletionItem {
    label: String,
    kind: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    documentation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    insert_text: Option<String>,
}

impl CompletionItem {
    fn function(function_name: FunctionName) -> Self {
        let name = function_name.as_str();
        let description = function_name.description().map(ToString::to_string);

        Self {
            label: name.to_string(),
            kind: 3,
            detail: None,
            documentation: description,
            insert_text: Some(format!("{name}()")),
        }
    }
}

fn suggest_code_actions(params: &CodeActionParams) -> Vec<CodeAction> {
    let mut actions = Vec::new();

    for diagnostic in &params.context.diagnostics {
        let Some(data) = &diagnostic.data else {
            continue;
        };

        let Ok(data) = serde_json::from_value::<DiagnosticData>(data.clone()) else {
            continue;
        };

        for suggestion in data.suggestions {
            let mut changes = HashMap::new();
            changes.insert(
                params.text_document.uri.clone(),
                vec![TextEdit {
                    range: suggestion.range.clone(),
                    new_text: suggestion.replacement,
                }],
            );

            actions.push(CodeAction {
                title: suggestion.title,
                kind: "quickfix".to_string(),
                diagnostics: vec![diagnostic.clone()],
                edit: WorkspaceEdit { changes },
            });
        }
    }

    actions
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestPayload {
        testing: bool,
    }

    #[test]
    fn test_serialize_header() {
        let payload = TestPayload { testing: true };
        let encoded = Header::<TestPayload>::new(payload).unwrap();
        let content = "Content-Length: 16\r\n\r\n{\"testing\":true}";
        assert_eq!(encoded.to_string().unwrap(), content);
    }
    #[test]
    fn test_parse_header() {
        let content: &[u8] = b"Content-Length: 16\r\n\r\n{\"testing\":true}";

        let body = parse_message(content).unwrap();

        let body = serde_json::from_str::<TestPayload>(&body).unwrap();

        assert_eq!(body, TestPayload { testing: true });
    }

    #[test]
    fn test_completion_suggests_function_names_for_prefix() {
        let uri = "file:///completion-test.sff".to_string();
        cache_document(uri.clone(), "RO".to_string());

        let params = CompletionParams {
            text_document: crate::structs::request::TextDocumentIdentifier { uri },
            position: crate::structs::request::Position {
                line: 0,
                character: 2,
            },
        };

        let completions = suggest_function_completions(&params);
        assert!(completions.iter().any(|item| item.label == "ROUND"));
    }

    #[test]
    fn test_completion_does_not_suggest_after_field_separator() {
        let uri = "file:///completion-field-test.sff".to_string();
        cache_document(uri.clone(), "Account.NA".to_string());

        let params = CompletionParams {
            text_document: crate::structs::request::TextDocumentIdentifier { uri },
            position: crate::structs::request::Position {
                line: 0,
                character: 10,
            },
        };

        let completions = suggest_function_completions(&params);
        assert!(completions.is_empty());
    }

    #[test]
    fn test_completion_includes_salesforce_description() {
        let uri = "file:///completion-description-test.sff".to_string();
        cache_document(uri.clone(), "RO".to_string());

        let params = CompletionParams {
            text_document: crate::structs::request::TextDocumentIdentifier { uri },
            position: crate::structs::request::Position {
                line: 0,
                character: 2,
            },
        };

        let round = suggest_function_completions(&params)
            .into_iter()
            .find(|item| item.label == "ROUND")
            .expect("ROUND completion should be present");

        assert!(
            round
                .documentation
                .as_deref()
                .is_some_and(|documentation| documentation.contains("nearest number"))
        );
    }

    #[test]
    fn test_code_action_returns_quickfix_from_diagnostic_suggestion() {
        let uri = "file:///code-action-test.sff".to_string();
        let diagnostics = build_syntax_diagnostics("5 && (1 | 3)", &uri);
        let diagnostic = diagnostics.first().expect("expected one diagnostic");

        let params = CodeActionParams {
            text_document: crate::structs::request::TextDocumentIdentifier { uri: uri.clone() },
            range: crate::structs::request::Range {
                start: crate::structs::request::Position {
                    line: diagnostic.range.start.line,
                    character: diagnostic.range.start.character,
                },
                end: crate::structs::request::Position {
                    line: diagnostic.range.end.line,
                    character: diagnostic.range.end.character,
                },
            },
            context: crate::structs::request::CodeActionContext {
                diagnostics: vec![crate::structs::request::CodeActionDiagnostic {
                    range: crate::structs::request::Range {
                        start: crate::structs::request::Position {
                            line: diagnostic.range.start.line,
                            character: diagnostic.range.start.character,
                        },
                        end: crate::structs::request::Position {
                            line: diagnostic.range.end.line,
                            character: diagnostic.range.end.character,
                        },
                    },
                    code: diagnostic.code.clone(),
                    data: diagnostic.data.clone(),
                }],
            },
        };

        let actions = suggest_code_actions(&params);
        assert!(!actions.is_empty());
        assert_eq!(actions[0].kind, "quickfix");
        assert!(actions[0].title.contains("did you mean `||`"));
        let edit = actions[0].edit.changes.get(&uri).expect("expected edits");
        assert_eq!(edit[0].new_text, "||");

        let encoded = serde_json::to_value(&actions[0]).expect("code action should serialize");
        let new_text = &encoded["edit"]["changes"][&uri][0]["newText"];
        assert_eq!(new_text, "||");
    }
}
