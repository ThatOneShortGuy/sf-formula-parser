use core::ops::Range;

use winnow::{
    LocatingSlice,
    combinator::{separated, trace},
    prelude::*,
    stream::AsChar,
    token::{one_of, take_while},
};

use crate::{parse::utils::spaced, token::FieldReference};

pub fn parse_field_name<'s>(input: &mut LocatingSlice<&'s str>) -> ModalResult<&'s str> {
    trace(
        "parse_field_name",
        (
            one_of(AsChar::is_alpha),
            take_while(0.., |c| AsChar::is_alphanum(c) || c == '_'),
        )
            .take(),
    )
    .parse_next(input)
}

pub fn parse_field_reference<'s>(
    input: &mut LocatingSlice<&'s str>,
) -> ModalResult<(FieldReference<'s>, Range<usize>)> {
    trace(
        "parse_field_reference",
        spaced(
            separated(1.., parse_field_name, ".")
                .map(|s: Vec<&'s str>| {
                    FieldReference::from_iter(s).expect("There should be at least one occurrence")
                })
                .with_span(),
        ),
    )
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_name() {
        let test_str = LocatingSlice::new("amount");
        assert_eq!(parse_field_name.parse(test_str).unwrap(), "amount");

        let test_str = LocatingSlice::new("amount is");
        assert_eq!(
            parse_field_name
                .parse_peek(test_str)
                .map(|(remain, s)| (*remain.as_ref(), s)),
            Ok((" is", "amount"))
        );
    }

    #[test]
    fn test_field_ref() {
        let test_str = LocatingSlice::new("Opportunity__r.Amount + 2");
        let (remaining, (parsed, _range)) = parse_field_reference.parse_peek(test_str).unwrap();
        assert_eq!(remaining.as_ref(), &"+ 2");
        assert_eq!(
            parsed,
            FieldReference::new("Opportunity__r").with_next("Amount")
        )
    }
}
