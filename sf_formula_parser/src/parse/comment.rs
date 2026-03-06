use core::ops::Range;

use winnow::{
    LocatingSlice,
    combinator::{delimited, trace},
    prelude::*,
    token::take_until,
};

use crate::{parse::utils::spaced, token::Comment};

pub fn parse_comment<'s>(
    input: &mut LocatingSlice<&'s str>,
) -> ModalResult<(Comment<'s>, Range<usize>)> {
    trace(
        "parse_comment",
        spaced(delimited("/*", take_until(0.., "*/"), "*/")).map(|s| Comment(s)),
    )
    .with_span()
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_comment() {
        let test_str = LocatingSlice::new("/*This is a\nformula\r\n comment*/");
        assert_eq!(
            parse_comment.parse(test_str).unwrap().0,
            Comment("This is a\nformula\r\n comment")
        );

        let test_str = LocatingSlice::new("/* /* comment */ */");
        assert!(parse_comment.parse(test_str).is_err());
    }
}
