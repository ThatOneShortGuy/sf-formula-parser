pub mod parse;
pub mod token;

use annotate_snippets::{AnnotationKind, Level, Renderer, Snippet};
use std::fmt;
use winnow::{
    LocatingSlice, Parser,
    error::{ContextError, ParseError},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub message: String,
    pub rendered: String,
    pub offset: usize,
    pub end_offset: usize,
    pub line: usize,
    pub column: usize,
    pub end_line: usize,
    pub end_column: usize,
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

fn render_parse_error<'s>(
    input: &'s str,
    err: ParseError<LocatingSlice<&'s str>, ContextError>,
) -> ValidationError {
    let message = {
        let inner = err.inner().to_string();
        if inner.trim().is_empty() {
            "unexpected token while parsing expression".to_string()
        } else {
            inner
        }
    };
    let offset = err.offset();
    let end_offset = input
        .get(offset..)
        .and_then(|rest| rest.chars().next().map(|ch| offset + ch.len_utf8()))
        .unwrap_or(offset);

    let (line, column) = offset_to_position(input, offset);
    let (end_line, end_column) = offset_to_position(input, end_offset);

    let report = &[Level::ERROR
        .primary_title("invalid formula expression")
        .element(
            Snippet::source(input)
                .path("expression")
                .line_start(1)
                .annotation(
                    AnnotationKind::Primary
                        .span(offset..end_offset)
                        .label("error occurred here"),
                ),
        )
        .element(Level::NOTE.message(message.clone()))];

    ValidationError {
        message,
        rendered: Renderer::styled().render(report).to_string(),
        offset,
        end_offset,
        line,
        column,
        end_line,
        end_column,
    }
}

pub fn validate_expression_detailed(input: &str) -> Result<(), ValidationError> {
    parse::expression::parse_expression
        .parse(LocatingSlice::new(input))
        .map(|_| ())
        .map_err(|err| render_parse_error(input, err))
}

pub fn validate_expression(input: &str) -> Result<(), String> {
    validate_expression_detailed(input).map_err(|err| err.rendered)
}
