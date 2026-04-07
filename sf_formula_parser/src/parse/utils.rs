use winnow::{
    LocatingSlice, Parser,
    ascii::multispace1,
    combinator::{alt, delimited, repeat},
    error::ParserError,
    token::take_until,
};

pub fn spaced<'s, Output, Error, ParseNext>(
    mut parser: ParseNext,
) -> impl Parser<LocatingSlice<&'s str>, Output, Error>
where
    Error: ParserError<LocatingSlice<&'s str>>,
    ParseNext: Parser<LocatingSlice<&'s str>, Output, Error>,
{
    winnow::combinator::trace("spaced", move |input: &mut LocatingSlice<&'s str>| {
        let _ = repeat::<_, _, (), _, _>(
            0..,
            alt((
                multispace1.void(),
                delimited("/*", take_until(0.., "*/"), "*/").void(),
            )),
        )
        .parse_next(input)?;
        let o2 = parser.parse_next(input)?;
        repeat::<_, _, (), _, _>(
            0..,
            alt((
                multispace1.void(),
                delimited("/*", take_until(0.., "*/"), "*/").void(),
            )),
        )
        .parse_next(input)
        .map(|_| o2)
    })
}
