use crate::parser_utils::*;
use chrono::Duration;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::space1,
    combinator::{map_res, opt, value},
    sequence::{pair, terminated},
    IResult,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Period {
    Day(usize),
    Week(usize),
    Month(usize),
    Year(usize),
}

impl Period {
    pub fn to_duration(&self) -> Duration {
        match self {
            Period::Day(v) => Duration::days(*v as i64),
            Period::Week(v) => Duration::weeks(*v as i64),
            Period::Month(v) => Duration::days(30 * *v as i64),
            Period::Year(v) => Duration::days(365 * *v as i64),
        }
    }
}

pub fn parse(input: &str) -> IResult<&str, Period> {
    map_res(
        pair(
            terminated(alt((parse_digits, parse_written_number)), space1),
            terminated(
                alt((tag("day"), tag("week"), tag("month"), tag("year"))),
                opt(tag("s")),
            ),
        ),
        |(digit, duration)| match duration {
            "day" => Ok(Period::Day(digit)),
            "week" => Ok(Period::Week(digit)),
            "month" => Ok(Period::Month(digit)),
            "year" => Ok(Period::Year(digit)),
            _ => Err("unable to parse duration"),
        },
    )(input)
}

fn parse_written_number(input: &str) -> IResult<&str, usize> {
    alt((
        value(1, tag("one")),
        value(2, tag("two")),
        value(3, tag("three")),
        value(4, tag("four")),
        value(5, tag("five")),
        value(6, tag("six")),
        value(7, tag("seven")),
        value(8, tag("eight")),
        value(9, tag("nine")),
        value(10, tag("ten")),
        value(11, tag("eleven")),
        value(12, tag("twelve")),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration() {
        assert_eq!(parse("1 day").unwrap().1, Period::Day(1));
        assert_eq!(parse("2 days").unwrap().1, Period::Day(2));
        assert_eq!(parse("300 days").unwrap().1, Period::Day(300));

        assert_eq!(parse("1 week").unwrap().1, Period::Week(1));
        assert_eq!(parse("2 weeks").unwrap().1, Period::Week(2));
        assert_eq!(parse("300 weeks").unwrap().1, Period::Week(300));

        assert_eq!(parse("1 month").unwrap().1, Period::Month(1));
        assert_eq!(parse("2 months").unwrap().1, Period::Month(2));
        assert_eq!(parse("300 months").unwrap().1, Period::Month(300));

        assert_eq!(parse("1 year").unwrap().1, Period::Year(1));
        assert_eq!(parse("2 years").unwrap().1, Period::Year(2));
        assert_eq!(parse("300 years").unwrap().1, Period::Year(300));
    }

    #[test]
    fn test_nonsense() {
        assert!(parse("1day").is_err());
    }

    #[test]
    fn test_spelled_numbers() {
        assert_eq!(parse("two days").unwrap().1, Period::Day(2));
    }
}
