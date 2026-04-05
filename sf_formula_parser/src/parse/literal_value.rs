use core::ops::Range;

use winnow::{
    LocatingSlice,
    ascii::{alpha1, float},
    combinator::{alt, delimited, repeat, trace},
    prelude::*,
    token::any,
};

use crate::{parse::utils::spaced, token::LiteralValue};

pub fn parse_checkbox<'s>(input: &mut LocatingSlice<&'s str>) -> ModalResult<bool> {
    trace(
        "parse_checkbox",
        alpha1.verify(|s: &&str| *s == "True" || *s == "False"),
    )
    .parse_next(input)
    .map(|s| s == "True")
}

pub fn parse_null<'s>(input: &mut LocatingSlice<&'s str>) -> ModalResult<()> {
    trace(
        "parse_null",
        alpha1.verify(|s: &&str| s.eq_ignore_ascii_case("null")),
    )
    .void()
    .parse_next(input)
}

pub fn parse_number<'s>(input: &mut LocatingSlice<&'s str>) -> ModalResult<f64> {
    trace("parse_number", float).parse_next(input)
}

pub fn parse_string<'s>(input: &mut LocatingSlice<&'s str>) -> ModalResult<&'s str> {
    trace(
        "parse_string",
        delimited(
            "\"",
            repeat::<_, _, (), _, _>(
                0..,
                alt((
                    "\"\"".void(),
                    any.verify(|c| *c != '"').void(), // any non-quote char
                )),
            )
            .take(),
            "\"",
        ),
    )
    .parse_next(input)
}

pub fn parse_literal<'s>(
    input: &mut LocatingSlice<&'s str>,
) -> ModalResult<(LiteralValue<'s>, Range<usize>)> {
    trace(
        "parse_literal",
        spaced(alt((
            parse_null.map(|()| LiteralValue::Null),
            parse_number.map(|f| LiteralValue::Number(f)),
            parse_checkbox.map(|b| LiteralValue::Checkbox(b)),
            parse_string.map(|s| LiteralValue::Text(s)),
        ))),
    )
    .with_span()
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number() {
        let test_str = LocatingSlice::new("0.5");
        assert_eq!(parse_number.parse(test_str).unwrap(), 0.5);

        let test_str = LocatingSlice::new(".5");
        assert_eq!(parse_number.parse(test_str).unwrap(), 0.5);

        let test_str = LocatingSlice::new("69420");
        assert_eq!(parse_number.parse(test_str).unwrap(), 69420.);

        let test_str = LocatingSlice::new("not3");
        assert!(parse_number.parse(test_str).is_err());

        let test_str = LocatingSlice::new("2not");
        assert!(parse_number.parse(test_str).is_err());
    }

    #[test]
    fn test_checkbox() {
        let test_str = LocatingSlice::new("True");
        assert_eq!(parse_checkbox.parse(test_str).unwrap(), true);
        let test_str = LocatingSlice::new("False");
        assert_eq!(parse_checkbox.parse(test_str).unwrap(), false);

        let test_str = LocatingSlice::new("true");
        assert!(parse_checkbox.parse(test_str).is_err());
    }

    #[test]
    fn test_null() {
        let test_str = LocatingSlice::new("null");
        assert_eq!(parse_null.parse(test_str).unwrap(), ());

        let test_str = LocatingSlice::new("Null");
        assert_eq!(parse_null.parse(test_str).unwrap(), ());

        let test_str = LocatingSlice::new("NULL");
        assert_eq!(parse_null.parse(test_str).unwrap(), ());

        let test_str = LocatingSlice::new("nul");
        assert!(parse_null.parse(test_str).is_err());

        let test_str = LocatingSlice::new("null");
        assert_eq!(parse_literal.parse(test_str).unwrap().0, LiteralValue::Null);
    }

    #[test]
    fn test_string() {
        let test_str = LocatingSlice::new("\"he said, \"\"hi\"\"\"");

        assert_eq!(parse_string.parse(test_str).unwrap(), "he said, \"\"hi\"\"");
    }
}
