use core::ops::Range;
use std::str::FromStr;

use winnow::{
    LocatingSlice,
    ascii::alpha1,
    combinator::{cut_err, delimited, opt, preceded, repeat, separated, trace},
    error::ErrMode,
    prelude::*,
};

use crate::{
    parse::{expression::parse_expression, utils::spaced},
    token::{Function, FunctionArgumentList, FunctionName},
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
) -> ModalResult<(FunctionArgumentList<'s>, Range<usize>)> {
    trace(
        "parse_function_args",
        delimited(
            spaced("("),
            (
                opt(parse_expression.map(|(e, _r)| e)),
                repeat(
                    0..,
                    preceded(spaced(","), cut_err(parse_expression.map(|(e, _r)| e))),
                ),
            )
                .map(|(first, mut rest): (Option<_>, Vec<_>)| {
                    if let Some(first) = first {
                        rest.insert(0, first);
                    }
                    FunctionArgumentList::from(rest)
                }),
            spaced(")"),
        ),
    )
    .with_span()
    .parse_next(input)
}

pub fn parse_function<'s>(
    input: &mut LocatingSlice<&'s str>,
) -> ModalResult<(Function<'s>, Range<usize>)> {
    trace(
        "parse_function",
        (parse_function_name, cut_err(parse_function_args))
            .map(|((fname, _), (args, _))| Function::new(fname, args)),
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
            parse_function_args.parse(test_str).unwrap().0,
            FunctionArgumentList(Vec::new())
        );

        let test_str = LocatingSlice::new("(1, \"\")");
        assert_eq!(
            parse_function_args.parse(test_str).unwrap().0,
            FunctionArgumentList(vec![Expression::literal(1), Expression::literal("")])
        );
    }

    #[test]
    fn test_parse_function() {
        let test_str = LocatingSlice::new("AND ( 1, \"\" )");
        assert_eq!(
            parse_function.parse(test_str).unwrap().0,
            Function::new(
                FunctionName::AND,
                FunctionArgumentList(vec![Expression::literal(1), Expression::literal("")])
            )
        );
    }
}
