pub mod parse;
pub mod token;

mod diagnostics;

use annotate_snippets::{AnnotationKind, Level, Renderer, Snippet};
use std::collections::HashSet;
use std::fmt;
use std::ops::Range;
use winnow::{
    LocatingSlice, Parser,
    error::{ContextError, ParseError, StrContext, StrContextValue},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub message: String,
    pub details: Vec<String>,
    pub suggestion_ids: Vec<&'static str>,
    pub suggestions: Vec<ValidationSuggestion>,
    pub rendered: String,
    pub offset: usize,
    pub end_offset: usize,
    pub line: usize,
    pub column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationSuggestion {
    pub id: &'static str,
    pub message: String,
    pub replacement: Option<String>,
    pub span: Option<Range<usize>>,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.rendered)
    }
}

impl std::error::Error for ValidationError {}

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

fn unexpected_token_message(input: &str, offset: usize) -> String {
    match input.get(offset..).and_then(|rest| rest.chars().next()) {
        Some(ch) => {
            let escaped = ch.escape_default().to_string();
            format!("unexpected \"{escaped}\"")
        }
        None => "unexpected end of input".to_string(),
    }
}

fn render_parse_error<'s>(
    input: &'s str,
    source_name: &str,
    err: ParseError<LocatingSlice<&'s str>, ContextError>,
) -> ValidationError {
    let offset = err.offset();
    let end_offset = input
        .get(offset..)
        .and_then(|rest| rest.chars().next().map(|ch| offset + ch.len_utf8()))
        .unwrap_or(offset);

    let (line, column) = offset_to_position(input, offset);
    let (end_line, end_column) = offset_to_position(input, end_offset);

    let mut seen_labels: HashSet<&'static str> = HashSet::new();
    let labels: Vec<&'static str> = err
        .inner()
        .context()
        .filter_map(|ctx| match ctx {
            StrContext::Label(label) => Some(*label),
            _ => None,
        })
        .filter(|label| seen_labels.insert(*label))
        .collect();
    let mut seen_expected: HashSet<String> = HashSet::new();
    let expected: Vec<String> = err
        .inner()
        .context()
        .filter_map(|ctx| match ctx {
            StrContext::Expected(StrContextValue::Description(desc)) => Some((*desc).to_string()),
            StrContext::Expected(StrContextValue::StringLiteral(s)) => Some(format!("`{s}`")),
            StrContext::Expected(StrContextValue::CharLiteral(c)) => Some(format!("`{c}`")),
            _ => None,
        })
        .filter(|exp| seen_expected.insert(exp.clone()))
        .collect();

    let message = unexpected_token_message(input, offset);
    let mut details = Vec::new();
    let mut suggestion_ids = Vec::new();
    let mut suggestions = Vec::new();

    if let Some(cause) = err.inner().cause() {
        let cause = cause.to_string();
        if cause != message {
            details.push(cause);
        }
    }

    let mut report = Level::ERROR
        .primary_title("invalid formula expression")
        .element(
            Snippet::source(input)
                .path(source_name)
                .line_start(1)
                .annotation(
                    AnnotationKind::Primary
                        .span(offset..end_offset)
                        .label("error occurred here"),
                ),
        )
        .element(Level::NOTE.message(message.clone()));

    let hint_ctx = diagnostics::HintContext {
        input,
        offset,
        end_offset,
        expected: &expected,
        labels: &labels,
    };

    let suggestion = diagnostics::derive_best_suggestion(&hint_ctx);

    if let Some(suggestion) = suggestion {
        suggestion_ids.push(suggestion.id);
        suggestions.push(ValidationSuggestion {
            id: suggestion.id,
            message: suggestion.message.clone(),
            replacement: suggestion.replacement.clone(),
            span: suggestion.span.clone(),
        });

        if let Some(patch) = suggestion.patch {
            report = report.element(
                Snippet::source(input)
                    .path(source_name)
                    .line_start(1)
                    .patch(patch),
            );
        }

        details.push(suggestion.message.clone());
        report = report.element(Level::HELP.message(suggestion.message));
    } else {
        if let Some(label) = labels
            .iter()
            .copied()
            .find(|label| !label.starts_with("expr."))
        {
            let detail = format!("invalid {label}");
            details.push(detail.clone());
            report = report.element(Level::NOTE.message(detail));
        }

        if !expected.is_empty() {
            let detail = format!("expected one of: {}", expected.join(", "));
            details.push(detail.clone());
            report = report.element(Level::NOTE.message(detail));
        }
    }

    ValidationError {
        message,
        details,
        suggestion_ids,
        suggestions,
        rendered: Renderer::styled().render(&[report]).to_string(),
        offset,
        end_offset,
        line,
        column,
        end_line,
        end_column,
    }
}

pub fn validate_expression_detailed(input: &str) -> Result<(), ValidationError> {
    validate_expression_detailed_with_source(input, "expression")
}

pub fn validate_expression_detailed_with_source(
    input: &str,
    source_name: &str,
) -> Result<(), ValidationError> {
    parse::expression::parse_expression
        .parse(LocatingSlice::new(input))
        .map(|_| ())
        .map_err(|err| render_parse_error(input, source_name, err))
}

pub fn validate_expression(input: &str) -> Result<(), String> {
    validate_expression_detailed(input).map_err(|err| err.rendered)
}
