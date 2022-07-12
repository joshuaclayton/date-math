use crate::{period, Period};
use chrono::NaiveDate;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::space0,
    combinator::map,
    sequence::{preceded, terminated},
    IResult,
};

#[derive(Debug, Eq, PartialEq)]
pub enum PeriodOp {
    Add(Period),
    Subtract(Period),
}

impl PeriodOp {
    pub fn apply(&self, value: NaiveDate) -> NaiveDate {
        match self {
            PeriodOp::Add(period) => value + period.to_duration(),
            PeriodOp::Subtract(period) => value - period.to_duration(),
        }
    }
}

pub fn parse(input: &str) -> IResult<&str, PeriodOp> {
    preceded(
        space0,
        alt((
            preceded(
                terminated(tag("+"), space0),
                map(period::parse, PeriodOp::Add),
            ),
            preceded(
                terminated(tag("-"), space0),
                map(period::parse, PeriodOp::Subtract),
            ),
        )),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(
            parse("+ 2 weeks").unwrap().1,
            PeriodOp::Add(Period::Week(2))
        );

        assert_eq!(
            parse("   +    2 weeks").unwrap().1,
            PeriodOp::Add(Period::Week(2))
        );
    }

    #[test]
    fn test_subtract() {
        assert_eq!(
            parse("- 3 days").unwrap().1,
            PeriodOp::Subtract(Period::Day(3))
        );

        assert_eq!(
            parse("   -    3 days").unwrap().1,
            PeriodOp::Subtract(Period::Day(3))
        );
    }
}
