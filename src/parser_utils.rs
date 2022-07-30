use nom::{
    character::complete::digit1,
    combinator::{map_res, recognize},
    IResult,
};

pub(crate) fn parse_digits<T: std::str::FromStr>(input: &str) -> IResult<&str, T> {
    map_res(recognize(digit1), str::parse)(input)
}
