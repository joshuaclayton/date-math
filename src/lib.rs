mod calculated_date;
pub mod cli;
mod period;

use calculated_date::CalculatedDate;
use period::Period;

use chrono::NaiveDate;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::space0,
    combinator::map,
    multi::separated_list1,
    sequence::{pair, preceded, separated_pair},
    IResult,
};

#[derive(Debug, PartialEq, Eq)]
pub enum DateMath {
    Periods(Period, Vec<Period>),
    Start(CalculatedDate),
    StartWithPeriods(CalculatedDate, Period, Vec<Period>),
}

impl DateMath {
    pub fn compute(&self) -> NaiveDate {
        match self {
            DateMath::Start(v) => v.calculate(),
            DateMath::StartWithPeriods(v, base, rest) => {
                v.calculate()
                    + rest
                        .iter()
                        .fold(base.to_duration(), |acc, x| acc + x.to_duration())
            }
            DateMath::Periods(base, rest) => {
                chrono::Local::today().naive_local()
                    + rest
                        .iter()
                        .fold(base.to_duration(), |acc, x| acc + x.to_duration())
            }
        }
    }
}

pub fn parse(input: &str) -> IResult<&str, DateMath> {
    alt((
        map(
            pair(
                calculated_date::parse,
                preceded(separated_pair(space0, tag("+"), space0), parse_periods),
            ),
            |(a, (b, c))| DateMath::StartWithPeriods(a, b, c),
        ),
        map(calculated_date::parse, DateMath::Start),
        map(parse_periods, |(period, periods)| {
            DateMath::Periods(period, periods)
        }),
    ))(input)
}

fn parse_periods(input: &str) -> IResult<&str, (Period, Vec<Period>)> {
    map(
        separated_list1(separated_pair(space0, tag("+"), space0), period::parse),
        |mut periods| {
            let first = periods.remove(0);
            (first, periods)
        },
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        assert_eq!(
            parse("1 day + 2 months").unwrap().1,
            DateMath::Periods(Period::Day(1), vec![Period::Month(2)])
        );

        assert_eq!(
            parse("Jan 2, 2021 + 15 weeks").unwrap().1,
            DateMath::StartWithPeriods(
                CalculatedDate::Raw(NaiveDate::from_ymd_opt(2021, 1, 2).unwrap()),
                Period::Week(15),
                vec![]
            )
        );

        assert_eq!(
            parse("Mar 31, 2021 + 15 weeks + 2 days").unwrap().1,
            DateMath::StartWithPeriods(
                CalculatedDate::Raw(NaiveDate::from_ymd_opt(2021, 3, 31).unwrap()),
                Period::Week(15),
                vec![Period::Day(2)]
            )
        );

        assert_eq!(
            parse("Mar 31, 2021").unwrap().1,
            DateMath::Start(CalculatedDate::Raw(
                NaiveDate::from_ymd_opt(2021, 3, 31).unwrap()
            ))
        );

        assert_eq!(
            parse("today").unwrap().1,
            DateMath::Start(CalculatedDate::Today)
        );
    }
}
