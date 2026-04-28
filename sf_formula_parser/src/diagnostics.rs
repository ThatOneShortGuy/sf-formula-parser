use annotate_snippets::Patch;
use std::ops::Range;

pub struct HintContext<'a> {
    pub input: &'a str,
    pub offset: usize,
    pub end_offset: usize,
    pub expected: &'a [String],
    pub labels: &'a [&'static str],
}

pub struct Suggestion {
    pub id: &'static str,
    pub message: String,
    pub replacement: Option<String>,
    pub span: Option<Range<usize>>,
    pub patch: Option<Patch<'static>>,
    pub priority: u8,
}

trait HintRule {
    fn apply(&self, ctx: &HintContext<'_>) -> Option<Suggestion>;
}

struct SinglePipeRule;
struct LoneBangRule;
struct MissingClosingParenRule;
struct TrailingCommaRule;
struct MissingCommaBetweenArgsRule;

impl HintRule for SinglePipeRule {
    fn apply(&self, ctx: &HintContext<'_>) -> Option<Suggestion> {
        let rest = ctx.input.get(ctx.offset..)?;
        let token = rest.chars().next()?;
        let expects_logical_or = ctx.expected.iter().any(|exp| exp == "`||`");
        let in_expression = ctx
            .labels
            .iter()
            .any(|label| *label == "expression" || *label == "expr.operator.logical_or_concat");

        if token == '|' && in_expression && (expects_logical_or || !rest.starts_with("||")) {
            return Some(Suggestion {
                id: "replace-single-pipe-with-double-pipe",
                message: "did you mean `||`?".to_string(),
                replacement: Some("||".to_string()),
                span: Some(ctx.offset..ctx.end_offset),
                patch: Some(Patch::new(ctx.offset..ctx.end_offset, "||")),
                priority: 100,
            });
        }

        None
    }
}

impl HintRule for LoneBangRule {
    fn apply(&self, ctx: &HintContext<'_>) -> Option<Suggestion> {
        let rest = ctx.input.get(ctx.offset..)?;
        let token = rest.chars().next()?;
        let expects_not_equal = ctx.expected.iter().any(|exp| exp == "`!=`");

        if token == '!' && (expects_not_equal || !rest.starts_with("!=")) {
            return Some(Suggestion {
                id: "replace-lone-bang-with-not-equal",
                message: "did you mean `!=`?".to_string(),
                replacement: Some("!=".to_string()),
                span: Some(ctx.offset..ctx.end_offset),
                patch: Some(Patch::new(ctx.offset..ctx.end_offset, "!=")),
                priority: 95,
            });
        }

        None
    }
}

impl HintRule for MissingClosingParenRule {
    fn apply(&self, ctx: &HintContext<'_>) -> Option<Suggestion> {
        let expects_close_paren = ctx.expected.iter().any(|exp| exp == "`)`");
        let in_paren = ctx
            .labels
            .iter()
            .any(|label| *label == "expr.parenthesized.close" || *label == "expr.parenthesized");

        if expects_close_paren && in_paren && ctx.offset >= ctx.input.len() {
            return Some(Suggestion {
                id: "insert-missing-closing-paren",
                message: "try adding a closing `)`".to_string(),
                replacement: Some(")".to_string()),
                span: Some(ctx.offset..ctx.end_offset),
                patch: Some(Patch::new(ctx.offset..ctx.end_offset, ")")),
                priority: 90,
            });
        }

        None
    }
}

impl HintRule for TrailingCommaRule {
    fn apply(&self, ctx: &HintContext<'_>) -> Option<Suggestion> {
        let rest = ctx.input.get(ctx.offset..)?;
        let token = rest.chars().next()?;
        if token != ')' {
            return None;
        }

        let mut previous_non_ws = None;
        for (idx, ch) in ctx.input[..ctx.offset].char_indices().rev() {
            if !ch.is_whitespace() {
                previous_non_ws = Some((idx, ch));
                break;
            }
        }

        let (comma_idx, prev_char) = previous_non_ws?;
        if prev_char != ',' {
            return None;
        }

        Some(Suggestion {
            id: "remove-trailing-comma",
            message: "remove trailing comma".to_string(),
            replacement: Some(String::new()),
            span: Some(comma_idx..comma_idx + prev_char.len_utf8()),
            patch: Some(Patch::new(
                comma_idx..comma_idx + prev_char.len_utf8(),
                "",
            )),
            priority: 97,
        })
    }
}

impl HintRule for MissingCommaBetweenArgsRule {
    fn apply(&self, ctx: &HintContext<'_>) -> Option<Suggestion> {
        let rest = ctx.input.get(ctx.offset..)?;
        let token = rest.chars().next()?;
        let expects_close_paren = ctx.expected.iter().any(|exp| exp == "`)`");

        if !expects_close_paren || !starts_argument(token) {
            return None;
        }

        let mut previous_non_ws = None;
        for (idx, ch) in ctx.input[..ctx.offset].char_indices().rev() {
            if !ch.is_whitespace() {
                previous_non_ws = Some((idx, ch));
                break;
            }
        }

        let (prev_idx, prev_char) = previous_non_ws?;
        if prev_char == ',' || !ends_argument(prev_char) {
            return None;
        }

        if !ctx.input[prev_idx + prev_char.len_utf8()..ctx.offset].contains('\n') {
            return None;
        }

        let insert_at = prev_idx + prev_char.len_utf8();
        Some(Suggestion {
            id: "insert-missing-comma-between-args",
            message: "did you mean to add a comma between arguments?".to_string(),
            replacement: Some(",".to_string()),
            span: Some(insert_at..insert_at),
            patch: Some(Patch::new(insert_at..insert_at, ",")),
            priority: 96,
        })
    }
}

fn starts_argument(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '"' | '(' | '!' | '-')
}

fn ends_argument(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | ')' | '"')
}

pub fn derive_best_suggestion(ctx: &HintContext<'_>) -> Option<Suggestion> {
    let rules: [&dyn HintRule; 5] = [
        &SinglePipeRule,
        &TrailingCommaRule,
        &MissingCommaBetweenArgsRule,
        &LoneBangRule,
        &MissingClosingParenRule,
    ];

    rules
        .iter()
        .filter_map(|rule| rule.apply(ctx))
        .max_by_key(|suggestion| suggestion.priority)
}

#[cfg(test)]
mod tests {
    use crate::validate_expression_detailed;

    #[test]
    fn test_pipe_hint_has_patch_suggestion() {
        let error = validate_expression_detailed("5 && (1 | 3)").err().unwrap();

        assert!(
            error
                .suggestion_ids
                .contains(&"replace-single-pipe-with-double-pipe")
        );
        assert!(
            error.rendered.contains("||") && error.rendered.contains("| 3"),
            "{}",
            error.rendered
        );
    }

    #[test]
    fn test_lone_bang_suggests_not_equal() {
        let error = validate_expression_detailed("1 ! 3").err().unwrap();

        assert!(
            error
                .suggestion_ids
                .contains(&"replace-lone-bang-with-not-equal")
        );
        assert!(error.rendered.contains("!="), "{}", error.rendered);
    }

    #[test]
    fn test_missing_close_paren_suggests_patch() {
        let error = validate_expression_detailed("5 && (1 + 3").err().unwrap();

        assert!(
            error
                .suggestion_ids
                .contains(&"insert-missing-closing-paren")
        );
        assert!(error.rendered.contains(")"), "{}", error.rendered);
    }

    #[test]
    fn test_valid_operators_do_not_show_patch_suggestions() {
        assert!(validate_expression_detailed("1 != 3").is_ok());
        assert!(validate_expression_detailed("True || False").is_ok());
    }

    #[test]
    fn test_trailing_comma_suggests_removal() {
        let error = validate_expression_detailed("AND(1,)").err().unwrap();

        assert!(error.suggestion_ids.contains(&"remove-trailing-comma"));
    }

    #[test]
    fn test_missing_comma_between_multiline_args_suggested() {
        let error = validate_expression_detailed("AND(\n  Amount\n  SBQQ__Uncalculated__c\n)")
            .err()
            .unwrap();

        assert!(
            error
                .suggestion_ids
                .contains(&"insert-missing-comma-between-args")
        );
    }

    #[test]
    fn test_missing_comma_not_suggested_when_comma_exists() {
        assert!(validate_expression_detailed("AND(\n  Amount,\n  SBQQ__Uncalculated__c\n)").is_ok());
    }
}
