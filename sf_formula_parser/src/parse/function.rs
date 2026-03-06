use core::ops::Range;
use std::str::FromStr;

use winnow::{
    LocatingSlice,
    ascii::alpha1,
    combinator::{delimited, separated, trace},
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
        Ok(match s.to_lowercase().as_str() {
            "abs" => Self::ABS,
            "addmonths" => Self::ADDMONTHS,
            "and" => Self::AND,
            "begins" => Self::BEGINS,
            "blankvalue" => Self::BLANKVALUE,
            "br" => Self::BR,
            "case" => Self::CASE,
            "casesafeid" => Self::CASESAFEID,
            "ceiling" => Self::CEILING,
            "contains" => Self::CONTAINS,
            "currencyrate" => Self::CURRENCYRATE,
            "date" => Self::DATE,
            "datevalue" => Self::DATEVALUE,
            "datetimevalue" => Self::DATETIMEVALUE,
            "day" => Self::DAY,
            "distance" => Self::DISTANCE,
            "exp" => Self::EXP,
            "find" => Self::FIND,
            "floor" => Self::FLOOR,
            "geolocation" => Self::GEOLOCATION,
            "getrecordids" => Self::GETRECORDIDS,
            "getsessionid" => Self::GETSESSIONID,
            "hour" => Self::HOUR,
            "htmlencode" => Self::HTMLENCODE,
            "hyperlink" => Self::HYPERLINK,
            "if" => Self::IF,
            "image" => Self::IMAGE,
            "imageproxyurl" => Self::IMAGEPROXYURL,
            "include" => Self::INCLUDE,
            "includes" => Self::INCLUDES,
            "isblank" => Self::ISBLANK,
            "ischanged" => Self::ISCHANGED,
            "isclone" => Self::ISCLONE,
            "isnew" => Self::ISNEW,
            "isnull" => Self::ISNULL,
            "isnumber" => Self::ISNUMBER,
            "ispickval" => Self::ISPICKVAL,
            "jsencode" => Self::JSENCODE,
            "jsinhtmlencode" => Self::JSINHTMLENCODE,
            "junctionidlist" => Self::JUNCTIONIDLIST,
            "left" => Self::LEFT,
            "len" => Self::LEN,
            "linkto" => Self::LINKTO,
            "ln" => Self::LN,
            "log" => Self::LOG,
            "lower" => Self::LOWER,
            "lpad" => Self::LPAD,
            "max" => Self::MAX,
            "mceiling" => Self::MCEILING,
            "mfloor" => Self::MFLOOR,
            "mid" => Self::MID,
            "millisecond" => Self::MILLISECOND,
            "min" => Self::MIN,
            "minute" => Self::MINUTE,
            "mod" => Self::MOD,
            "month" => Self::MONTH,
            "not" => Self::NOT,
            "now" => Self::NOW,
            "nullvalue" => Self::NULLVALUE,
            "or" => Self::OR,
            "parentgroupval" => Self::PARENTGROUPVAL,
            "predict" => Self::PREDICT,
            "prevgroupval" => Self::PREVGROUPVAL,
            "priorvalue" => Self::PRIORVALUE,
            "regex" => Self::REGEX,
            "requirescript" => Self::REQUIRESCRIPT,
            "reverse" => Self::REVERSE,
            "right" => Self::RIGHT,
            "round" => Self::ROUND,
            "rpad" => Self::RPAD,
            "second" => Self::SECOND,
            "sqrt" => Self::SQRT,
            "substitute" => Self::SUBSTITUTE,
            "text" => Self::TEXT,
            "timenow" => Self::TIMENOW,
            "timevalue" => Self::TIMEVALUE,
            "today" => Self::TODAY,
            "trim" => Self::TRIM,
            "upper" => Self::UPPER,
            "urlencode" => Self::URLENCODE,
            "urlfor" => Self::URLFOR,
            "value" => Self::VALUE,
            "vlookup" => Self::VLOOKUP,
            "weekday" => Self::WEEKDAY,
            "year" => Self::YEAR,
            _ => return Err(UnknownFunction),
        })
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
            separated(0.., parse_expression.map(|(e, _r)| e), spaced(","))
                .map(|it: Vec<_>| FunctionArgumentList::from(it)),
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
        (parse_function_name, parse_function_args)
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
