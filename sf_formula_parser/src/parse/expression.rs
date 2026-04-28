use winnow::{
    LocatingSlice, ModalResult, Parser,
    combinator::{Infix, alt, cut_err, delimited, expression, fail, trace},
    error::{StrContext, StrContextValue},
};

use crate::{
    parse::{
        field_reference::parse_field_reference, function::parse_function,
        literal_value::parse_literal, utils::spaced,
    },
    token::{Expression, Operator, UnaryExpr},
};

fn parse_primary_expression<'s>(input: &mut LocatingSlice<&'s str>) -> ModalResult<Expression<'s>> {
    trace(
        "parse_primary_expression",
        alt((
            parse_function.map(|(f, r)| Expression::function(f, r)),
            parse_literal.map(|(l, r)| Expression::literal(l, r)),
            parse_field_reference.map(|(f, r)| Expression::field_ref(f, r)),
            delimited(spaced("("), cut_err(parse_expression), cut_err(spaced(")"))),
            fail.context(StrContext::Label("primary expression"))
                .context(StrContext::Expected(StrContextValue::Description(
                    "function",
                )))
                .context(StrContext::Expected(StrContextValue::Description(
                    "literal",
                )))
                .context(StrContext::Expected(StrContextValue::Description(
                    "field reference",
                )))
                .context(StrContext::Expected(StrContextValue::Description(
                    "parenthesized expression",
                ))),
        )),
    )
    .parse_next(input)
}

fn parse_unary_expression<'s>(input: &mut LocatingSlice<&'s str>) -> ModalResult<Expression<'s>> {
    trace(
        "parse_unary_expression",
        alt((
            spaced(("!", parse_unary_expression).with_span())
                .map(|((_, rhs), r)| Expression::unary_expr(UnaryExpr(Operator::Not, rhs), r)),
            spaced(("-", parse_unary_expression).with_span())
                .map(|((_, rhs), r)| Expression::unary_expr(UnaryExpr(Operator::Negative, rhs), r)),
            parse_primary_expression,
            fail.context(StrContext::Expected(StrContextValue::CharLiteral('!')))
                .context(StrContext::Expected(StrContextValue::CharLiteral('-')))
                .context(StrContext::Expected(StrContextValue::Description(
                    "primary expression",
                ))),
        )),
    )
    .parse_next(input)
}

pub fn parse_expression<'s>(input: &mut LocatingSlice<&'s str>) -> ModalResult<Expression<'s>> {
    macro_rules! ops {
        ($varient:ident, $prec:literal, $s:literal $(,$ss:literal)+ $(,)?) => {
            spaced(alt(($s, $($ss),+))).value(Infix::Left($prec, |_, a, b| {
                Ok(Expression::from_binary_expr(a, Operator::$varient, b))
            }))
        };

        ($varient:ident, $prec:literal, $s:literal) => {
            spaced($s).value(Infix::Left($prec, |_, a, b| {
                Ok(Expression::from_binary_expr(a, Operator::$varient, b))
            }))
        };
    }

    let parser = alt((
        alt((
            ops!(Exponentiation, 10, "^"),
            ops!(Multiply, 9, "*"),
            ops!(Divide, 9, "/"),
            ops!(Add, 8, "+"),
            ops!(Subtract, 8, "-"),
        )),
        alt((
            ops!(Equal, 7, "==", "="),
            ops!(NotEqual, 7, "!=", "<>"),
            ops!(LessThanOrEqual, 7, "<="),
            ops!(LessThan, 7, "<"),
            ops!(GreaterThanOrEqual, 7, ">="),
            ops!(GreaterThan, 7, ">"),
        )),
        alt((
            ops!(And, 6, "&&"),
            ops!(Or, 6, "||"),
            ops!(Concatenate, 5, "&"),
        )),
        fail.context(StrContext::Label("binary expression"))
            .context(StrContext::Expected(StrContextValue::Description(
                "a valid binary operator",
            )))
            .context(StrContext::Expected(StrContextValue::StringLiteral("+")))
            .context(StrContext::Expected(StrContextValue::StringLiteral("-")))
            .context(StrContext::Expected(StrContextValue::StringLiteral("*")))
            .context(StrContext::Expected(StrContextValue::StringLiteral("/")))
            .context(StrContext::Expected(StrContextValue::StringLiteral("=")))
            .context(StrContext::Expected(StrContextValue::StringLiteral("!=")))
            .context(StrContext::Expected(StrContextValue::StringLiteral("<")))
            .context(StrContext::Expected(StrContextValue::StringLiteral(">")))
            .context(StrContext::Expected(StrContextValue::StringLiteral("&&")))
            .context(StrContext::Expected(StrContextValue::StringLiteral("||")))
            .context(StrContext::Expected(StrContextValue::StringLiteral("&"))),
    ));

    trace(
        "parse_expression",
        expression(parse_unary_expression)
            .infix(parser)
            .context(StrContext::Label("expression")),
    )
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use crate::token::{BinaryExpr, FieldReference, LiteralValue, UnaryExpr};

    use super::*;

    fn span_of(input: &LocatingSlice<&str>, needle: &str) -> std::ops::Range<usize> {
        let source = input.as_ref();
        let start = source
            .find(needle)
            .unwrap_or_else(|| panic!("missing span for {needle}"));
        start..(start + needle.len())
    }

    #[test]
    fn test_bin_expr() {
        let test_str = LocatingSlice::new("6 + 9");
        assert_eq!(
            parse_expression.parse(test_str).unwrap(),
            Expression::binary_expr(
                0..test_str.len(),
                BinaryExpr(
                    Expression::literal(6, span_of(&test_str, "6")),
                    Operator::Add,
                    Expression::literal(9, span_of(&test_str, "9"))
                ),
            )
        );

        let test_str = LocatingSlice::new("6 + 9 * 42");
        assert_eq!(
            parse_expression.parse(test_str).unwrap(),
            Expression::binary_expr(
                0..test_str.len(),
                BinaryExpr(
                    Expression::literal(6, span_of(&test_str, "6")),
                    Operator::Add,
                    Expression::binary_expr(
                        span_of(&test_str, "9 * 42"),
                        BinaryExpr(
                            Expression::literal(9, span_of(&test_str, "9")),
                            Operator::Multiply,
                            Expression::literal(42, span_of(&test_str, "42"))
                        )
                    )
                )
            )
        );

        let test_str = LocatingSlice::new("(6 + 9) * 42");
        assert_eq!(
            parse_expression.parse(test_str).unwrap(),
            Expression::binary_expr(
                1..test_str.len(),
                BinaryExpr(
                    Expression::binary_expr(
                        span_of(&test_str, "6 + 9"),
                        BinaryExpr(
                            Expression::literal(6, span_of(&test_str, "6")),
                            Operator::Add,
                            Expression::literal(9, span_of(&test_str, "9")),
                        )
                    ),
                    Operator::Multiply,
                    Expression::literal(42, span_of(&test_str, "42"))
                )
            )
        );
    }

    #[test]
    fn test_bad_op() {
        let test_str = LocatingSlice::new("1 | 3");
        let error = parse_expression.parse(test_str).err().unwrap();
        assert_eq!(error.char_span(), span_of(&test_str, "|"));

        let test_str = LocatingSlice::new("5 && (1 | 3)");
        let error = parse_expression.parse(test_str).err().unwrap();
        assert_eq!(error.char_span(), span_of(&test_str, "|"));
        assert!(!error.to_string().contains("unary expression"), "{}", error);
        assert!(
            !error.to_string().contains("binary expression"),
            "{}",
            error
        );
    }

    #[test]
    fn test_expr1() {
        let test_str =
            LocatingSlice::new("Amount - SBQQ__PrimaryQuote__r.Non_Commissionable_Revenue__c");
        assert_eq!(
            parse_expression.parse(test_str).unwrap(),
            Expression::binary_expr(
                0..test_str.len(),
                BinaryExpr(
                    Expression::field_ref("Amount", span_of(&test_str, "Amount")),
                    Operator::Subtract,
                    Expression::field_ref(
                        FieldReference::new("SBQQ__PrimaryQuote__r")
                            .with_next("Non_Commissionable_Revenue__c"),
                        span_of(
                            &test_str,
                            "SBQQ__PrimaryQuote__r.Non_Commissionable_Revenue__c"
                        )
                    )
                )
            )
        );
    }

    #[test]
    fn test_expr2() {
        let test_str = LocatingSlice::new("Short_Leg_Outside_Left_in__c == null");

        assert_eq!(
            parse_expression.parse(test_str).unwrap(),
            Expression::binary_expr(
                0..test_str.len(),
                BinaryExpr(
                    Expression::field_ref(
                        FieldReference::new("Short_Leg_Outside_Left_in__c"),
                        span_of(&test_str, "Short_Leg_Outside_Left_in__c")
                    ),
                    Operator::Equal,
                    Expression::literal(LiteralValue::Null, span_of(&test_str, "null"))
                )
            )
        );
    }

    #[test]
    fn test_not() {
        let test_str = LocatingSlice::new("!True");

        assert_eq!(
            parse_expression.parse(test_str).unwrap(),
            Expression::unary_expr(
                UnaryExpr(
                    Operator::Not,
                    Expression::literal(LiteralValue::Checkbox(true), span_of(&test_str, "True")),
                ),
                0..test_str.len()
            )
        );

        let test_str = LocatingSlice::new("3 + -2");
        assert_eq!(
            parse_expression.parse(test_str).unwrap(),
            Expression::binary_expr(
                0..test_str.len(),
                BinaryExpr(
                    Expression::literal(3, span_of(&test_str, "3")),
                    Operator::Add,
                    Expression::unary_expr(
                        UnaryExpr(
                            Operator::Negative,
                            Expression::literal(2, span_of(&test_str, "2"))
                        ),
                        span_of(&test_str, "-2")
                    )
                )
            )
        );
    }

    #[test]
    fn test_err() {
        let test_str = LocatingSlice::new("1 ! 3");
        let parsed = parse_expression.parse(test_str);

        assert!(parsed.is_err());
    }

    #[test]
    fn test_paren_expression() {
        let test_str =
            LocatingSlice::new("SBQQ__Quote__r.Exterior_Color_All__c != (Exterior_Color__c)");

        assert_eq!(
            parse_expression.parse(test_str).unwrap(),
            Expression::binary_expr(
                0..test_str.len() - 1,
                BinaryExpr(
                    Expression::field_ref(
                        FieldReference::new("SBQQ__Quote__r").with_next("Exterior_Color_All__c"),
                        span_of(&test_str, "SBQQ__Quote__r.Exterior_Color_All__c")
                    ),
                    Operator::NotEqual,
                    Expression::field_ref(
                        FieldReference::new("Exterior_Color__c"),
                        span_of(&test_str, "Exterior_Color__c")
                    ),
                )
            )
        );
    }

    #[test]
    fn test_comment_expression() {
        let test_str = LocatingSlice::new(
            "IF(ISNULL(Wall_Thickness_in__c), 'Depth:','Wall Thickness:') & ' __________'
/* --- COMMENTING OUT EXTERIOR ---
    & BR()
    & BR()
    & 'Exterior: W __________ x H __________'
*/
    & BR()",
        );

        parse_expression.parse(test_str).unwrap();
    }
}
