use core::ops::Range;
use std::str::FromStr;

use winnow::{
    LocatingSlice,
    ascii::alpha1,
    combinator::{alt, cut_err, delimited, peek, repeat, separated, trace},
    error::{ErrMode, StrContext, StrContextValue},
    prelude::*,
};

use crate::{
    parse::{expression::parse_expression, utils::spaced},
    token::{Expression, Function, FunctionArgumentList, FunctionName},
};
#[derive(Debug)]
pub struct UnknownFunction;

impl FromStr for FunctionName {
    type Err = UnknownFunction;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        FunctionName::ALL
            .iter()
            .copied()
            .find(|name| name.as_str().eq_ignore_ascii_case(s))
            .ok_or(UnknownFunction)
    }
}

pub fn parse_function_name<'s>(
    input: &mut LocatingSlice<&'s str>,
) -> ModalResult<(FunctionName, Range<usize>)> {
    let parsed = trace(
        "parse_function",
        spaced(alpha1.map(|s| FunctionName::from_str(s))),
    )
    .with_span()
    .parse_next(input)?;

    match parsed {
        (Ok(f), r) => Ok((f, r)),
        (Err(_), _) => Err(ErrMode::from_input(input)),
    }
}

pub fn parse_function_args<'s>(
    input: &mut LocatingSlice<&'s str>,
) -> ModalResult<FunctionArgumentList<'s>> {
    let parse_arg = || parse_expression;

    trace(
        "parse_function_args",
        delimited(
            spaced("(").context(StrContext::Expected(StrContextValue::CharLiteral('('))),
            alt((
                peek(spaced(")")).value(Vec::new()),
                (
                    cut_err(parse_arg()),
                    repeat(
                        0..,
                        (spaced(","), cut_err(parse_arg())).map(|(_, expr)| expr),
                    ),
                )
                    .map(|(first, rest): (Expression<'_>, Vec<Expression<'_>>)| {
                        let mut args = Vec::with_capacity(rest.len() + 1);
                        args.push(first);
                        args.extend(rest);
                        args
                    }),
            ))
            .with_span(),
            spaced(")").context(StrContext::Expected(StrContextValue::CharLiteral(')'))),
        ),
    )
    .map(|(data, r): (Vec<Expression<'_>>, Range<usize>)| FunctionArgumentList::from((data, r)))
    .parse_next(input)
}

pub fn parse_function<'s>(
    input: &mut LocatingSlice<&'s str>,
) -> ModalResult<(Function<'s>, Range<usize>)> {
    trace(
        "parse_function",
        (
            parse_function_name,
            peek(spaced("(")),
            cut_err(parse_function_args),
        )
            .map(|((fname, _), _, args)| Function::new(fname, args)),
    )
    .with_span()
    .parse_next(input)
}

pub fn parser_thing<'s>(input: &mut LocatingSlice<&'s str>) -> ModalResult<Vec<&'s str>> {
    delimited("(", separated(0.., alpha1, spaced(",")), ")")
        .map(|it: Vec<_>| it)
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use crate::token::Expression;

    use super::*;

    #[test]
    fn test_parse_function_name() {
        let test_str = LocatingSlice::new("ADDMONTHS");
        assert_eq!(
            parse_function_name.parse(test_str).unwrap().0,
            FunctionName::ADDMONTHS
        );

        let test_str = LocatingSlice::new("ReQuIReScriPT");
        assert_eq!(
            parse_function_name.parse(test_str).unwrap().0,
            FunctionName::REQUIRESCRIPT
        );

        let test_str = LocatingSlice::new("UPER");
        assert!(parse_function_name.parse(test_str).is_err(),);
    }

    #[test]
    fn test_parse_function_args() {
        let test_str = LocatingSlice::new("()");
        assert_eq!(
            parse_function_args.parse(test_str).unwrap(),
            FunctionArgumentList::new(Vec::new(), 1..1)
        );

        let test_str = LocatingSlice::new("(1, \"\")");
        assert_eq!(
            parse_function_args.parse(test_str).unwrap(),
            FunctionArgumentList::new(
                vec![Expression::literal(1, 1..2), Expression::literal("", 4..6)],
                1..6
            )
        );
    }

    #[test]
    fn test_parse_function() {
        let test_str = LocatingSlice::new("AND ( 1, \"\" )");
        assert_eq!(
            parse_function.parse(test_str).unwrap().0,
            Function::new(
                FunctionName::AND,
                FunctionArgumentList::new(
                    vec![Expression::literal(1, 6..7), Expression::literal("", 9..11)],
                    6..12
                )
            )
        );
    }

    #[test]
    fn test_single_arg() {
        let test_str = LocatingSlice::new("ISNULL(Wall_Thickness_in__c)");

        assert_eq!(
            parse_function.parse(test_str).unwrap().0,
            Function::new(
                FunctionName::ISNULL,
                FunctionArgumentList::new(
                    [Expression::field_ref("Wall_Thickness_in__c", 7..27)],
                    7..27
                )
            )
        );
    }

    #[test]
    fn test_comment_func() {
        let test_str = LocatingSlice::new(
            "IF(ISNULL(Wall_Thickness_in__c), /* wall */ 'Depth:','Wall Thickness:')",
        );

        parse_expression.parse(test_str).unwrap();
    }
}
