use std::ops::Range;

use winnow::{
    LocatingSlice, ModalResult, Parser,
    combinator::{Infix, alt, delimited, expression, trace},
};

use crate::{
    parse::{
        field_reference::parse_field_reference, function::parse_function,
        literal_value::parse_literal, utils::spaced,
    },
    token::{BinaryExpr, Expression, Operator, UnaryExpr},
};

fn parse_primary_expression<'s>(input: &mut LocatingSlice<&'s str>) -> ModalResult<Expression<'s>> {
    trace(
        "parse_primary_expression",
        spaced(alt((
            parse_function.map(|(f, _r)| Expression::function(f)),
            parse_literal.map(|(l, _r)| Expression::literal(l)),
            parse_field_reference.map(|(f, _r)| Expression::field_ref(f)),
            delimited("(", parse_expression.map(|e| e.0), ")"),
        ))),
    )
    .parse_next(input)
}

fn parse_unary_expression<'s>(input: &mut LocatingSlice<&'s str>) -> ModalResult<Expression<'s>> {
    trace(
        "parse_unary_expression",
        alt((
            (spaced("!"), parse_unary_expression)
                .map(|(_, rhs)| Expression::unary_expr(UnaryExpr(Operator::Not, rhs))),
            (spaced("-"), parse_unary_expression)
                .map(|(_, rhs)| Expression::unary_expr(UnaryExpr(Operator::Negative, rhs))),
            parse_primary_expression,
        )),
    )
    .parse_next(input)
}

pub fn parse_expression<'s>(
    input: &mut LocatingSlice<&'s str>,
) -> ModalResult<(Expression<'s>, Range<usize>)> {
    macro_rules! ops {
        ($varient:ident, $prec:literal, $s:literal $(,$ss:literal)+ $(,)?) => {
            alt(($s, $($ss),+)).value(Infix::Left($prec, |_, a, b| {
                Ok(Expression::binary_expr(BinaryExpr(a, Operator::$varient, b)))
            }))
        };

        ($varient:ident, $prec:literal, $s:literal) => {
            $s.value(Infix::Left($prec, |_, a, b| {
                Ok(Expression::binary_expr(BinaryExpr(a, Operator::$varient, b)))
            }))
        };
    }

    let parser = alt((
        ops!(Exponentiation, 10, "^"),
        ops!(Multiply, 9, "*"),
        ops!(Divide, 9, "/"),
        ops!(Add, 8, "+"),
        ops!(Subtract, 8, "-"),
        ops!(Equal, 7, "=", "=="),
        ops!(NotEqual, 7, "!=", "<>"),
        ops!(LessThanOrEqual, 7, "<="),
        ops!(LessThan, 7, "<"),
        ops!(GreaterThanOrEqual, 7, ">="),
        ops!(GreaterThan, 7, ">"),
        ops!(And, 6, "&&"),
        ops!(Or, 6, "||"),
        ops!(Concatenate, 5, "&"),
    ));

    trace(
        "parse_expression",
        expression(parse_unary_expression).infix(spaced(parser)),
    )
    .with_span()
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use crate::token::{FieldReference, LiteralValue, UnaryExpr};

    use super::*;

    #[test]
    fn test_bin_expr() {
        let test_str = LocatingSlice::new("6 + 9");
        assert_eq!(
            parse_expression.parse(test_str).unwrap().0,
            Expression::binary_expr(BinaryExpr(
                Expression::literal(6),
                Operator::Add,
                Expression::literal(9)
            ))
        );

        let test_str = LocatingSlice::new("6 + 9 * 42");
        assert_eq!(
            parse_expression.parse(test_str).unwrap().0,
            Expression::binary_expr(BinaryExpr(
                Expression::literal(6),
                Operator::Add,
                Expression::binary_expr(BinaryExpr(
                    Expression::literal(9),
                    Operator::Multiply,
                    Expression::literal(42)
                ))
            ))
        );

        let test_str = LocatingSlice::new("(6 + 9) * 42");
        assert_eq!(
            parse_expression.parse(test_str).unwrap().0,
            Expression::binary_expr(BinaryExpr(
                Expression::binary_expr(BinaryExpr(
                    Expression::literal(6),
                    Operator::Add,
                    Expression::literal(9),
                )),
                Operator::Multiply,
                Expression::literal(42)
            ))
        );
    }

    #[test]
    fn test_expr1() {
        let test_str =
            LocatingSlice::new("Amount - SBQQ__PrimaryQuote__r.Non_Commissionable_Revenue__c");
        assert_eq!(
            parse_expression.parse(test_str).unwrap().0,
            Expression::binary_expr(BinaryExpr(
                Expression::field_ref("Amount"),
                Operator::Subtract,
                Expression::field_ref(
                    FieldReference::new("SBQQ__PrimaryQuote__r")
                        .with_next("Non_Commissionable_Revenue__c")
                )
            ))
        );
    }

    #[test]
    fn test_not() {
        let test_str = LocatingSlice::new("!True");

        assert_eq!(
            parse_expression.parse(test_str).unwrap().0,
            Expression::unary_expr(UnaryExpr(
                Operator::Not,
                Expression::Literal(LiteralValue::Checkbox(true)),
            ))
        );
    }
}
