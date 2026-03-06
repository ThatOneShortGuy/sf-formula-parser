use winnow::{
    Parser,
    ascii::multispace0,
    error::ParserError,
    stream::{AsChar, Stream, StreamIsPartial},
};

pub fn spaced<Input, Output, Error, ParseNext>(
    mut parser: ParseNext,
) -> impl Parser<Input, Output, Error>
where
    Input: Stream + StreamIsPartial,
    Error: ParserError<Input>,
    ParseNext: Parser<Input, Output, Error>,
    <Input as Stream>::Token: AsChar + Clone,
{
    winnow::combinator::trace("spaced", move |input: &mut Input| {
        let _ = multispace0.parse_next(input)?;
        let o2 = parser.parse_next(input)?;
        multispace0.parse_next(input).map(|_| o2)
    })
}
