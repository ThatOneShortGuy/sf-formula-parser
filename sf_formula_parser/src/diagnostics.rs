use annotate_snippets::Patch;

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
    pub patch: Option<Patch<'static>>,
    pub priority: u8,
}

trait HintRule {
    fn apply(&self, ctx: &HintContext<'_>) -> Option<Suggestion>;
}

struct SinglePipeRule;
struct LoneBangRule;
struct MissingClosingParenRule;

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
                message: "did you mean `||` for logical OR?".to_string(),
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
                message: "did you mean `!=` for not equal?".to_string(),
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
                patch: Some(Patch::new(ctx.offset..ctx.end_offset, ")")),
                priority: 90,
            });
        }

        None
    }
}

pub fn derive_best_suggestion(ctx: &HintContext<'_>) -> Option<Suggestion> {
    let rules: [&dyn HintRule; 3] = [&SinglePipeRule, &LoneBangRule, &MissingClosingParenRule];

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
}
