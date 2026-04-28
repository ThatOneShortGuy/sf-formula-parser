use annotate_snippets::Patch;

pub struct HintContext<'a> {
    pub input: &'a str,
    pub offset: usize,
    pub end_offset: usize,
    pub expected: &'a [String],
    pub labels: &'a [&'static str],
}

pub struct Suggestion {
    pub message: String,
    pub patch: Option<Patch<'static>>,
    pub priority: u8,
}

trait HintRule {
    fn apply(&self, ctx: &HintContext<'_>) -> Option<Suggestion>;
}

struct SinglePipeRule;

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
                message: "did you mean `||` for logical OR?".to_string(),
                patch: Some(Patch::new(ctx.offset..ctx.end_offset, "||")),
                priority: 100,
            });
        }

        None
    }
}

pub fn derive_best_suggestion(ctx: &HintContext<'_>) -> Option<Suggestion> {
    let rules: [&dyn HintRule; 1] = [&SinglePipeRule];

    rules
        .iter()
        .filter_map(|rule| rule.apply(ctx))
        .max_by_key(|suggestion| suggestion.priority)
}
