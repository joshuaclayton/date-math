mod calculated_date;
pub mod cli;
mod parser_utils;
mod period;
mod period_operation;
mod relative_period;

use calculated_date::CalculatedDate;
use chrono::NaiveDate;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::space0,
    combinator::map,
    multi::many0,
    sequence::{delimited, pair, separated_pair},
    IResult,
};
use period::Period;
use period_operation::PeriodOp;
use std::convert::TryInto;

#[derive(Debug, PartialEq, Eq)]
pub enum DateMath {
    Periods(Period, Vec<PeriodOp>),
    Start(CalculatedDate),
    StartWithPeriods(CalculatedDate, PeriodOp, Vec<PeriodOp>),
    DateDiff(CalculatedDate, CalculatedDate),
}

#[derive(Debug, PartialEq)]
pub enum ComputeOutcome {
    Date(NaiveDate),
    DifferenceInDays(usize),
}

impl std::fmt::Display for ComputeOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ComputeOutcome::Date(date) => write!(f, "{}", date),
            ComputeOutcome::DifferenceInDays(1) => write!(f, "1 day"),
            ComputeOutcome::DifferenceInDays(days) => write!(f, "{} days", days),
        }
    }
}

impl From<NaiveDate> for ComputeOutcome {
    fn from(date: NaiveDate) -> Self {
        ComputeOutcome::Date(date)
    }
}

impl DateMath {
    pub fn compute(&self, today: NaiveDate) -> ComputeOutcome {
        match self {
            DateMath::DateDiff(from, to) => ComputeOutcome::DifferenceInDays(
                (from.calculate(today) - to.calculate(today))
                    .num_days()
                    .abs()
                    .try_into()
                    .unwrap(),
            ),
            DateMath::Start(v) => v.calculate(today).into(),
            DateMath::StartWithPeriods(v, base, rest) => rest
                .iter()
                .fold(base.apply(v.calculate(today)), |acc, x| x.apply(acc))
                .into(),
            DateMath::Periods(base, rest) => rest
                .iter()
                .fold(
                    chrono::Local::today().naive_local() + base.to_duration(),
                    |acc, x| x.apply(acc),
                )
                .into(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseResult<'a> {
    Success(DateMath),
    PartialSuccess(DateMath, &'a str),
    Error(nom::Err<nom::error::Error<&'a str>>),
}

impl<'a> From<IResult<&'a str, DateMath>> for ParseResult<'a> {
    fn from(result: IResult<&'a str, DateMath>) -> Self {
        match result {
            Ok(("", math)) => ParseResult::Success(math),
            Ok((unparsed, math)) => ParseResult::PartialSuccess(math, unparsed),
            Err(e) => ParseResult::Error(e),
        }
    }
}

pub fn parse(input: &str) -> IResult<&str, DateMath> {
    alt((
        map(
            pair(
                calculated_date::parse,
                pair(period_operation::parse, many0(period_operation::parse)),
            ),
            |(a, (b, c))| DateMath::StartWithPeriods(a, b, c),
        ),
        map(
            separated_pair(
                calculated_date::parse,
                delimited(space0, tag("-"), space0),
                calculated_date::parse,
            ),
            |(from, to)| DateMath::DateDiff(from, to),
        ),
        map(relative_period::parse, |(date, period_op, rest)| {
            DateMath::StartWithPeriods(date, period_op, rest)
        }),
        map(calculated_date::parse, DateMath::Start),
        map(
            pair(period::parse, many0(period_operation::parse)),
            |(period, periods)| DateMath::Periods(period, periods),
        ),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smoke() {
        let examples = vec![
            "today",
            "yesterday",
            "tomorrow",
            "2 weeks ago",
            "2022-01-20",
            "2 weeks from now",
            "two weeks from tomorrow",
            "1 week and 2 days ago",
            "1 year, 2 weeks, and 3 days ago",
            "January 15, 2021 + 2 weeks + 1 day",
            "2 weeks and 1 day before January 15",
            "2 weeks and 1 day from January 15",
            "2 weeks and 1 day after January 15",
        ];

        assert!(examples
            .iter()
            .map(|v| parse(v).into())
            .all(|v| is_parse_success(&v)));
    }

    #[test]
    fn test_parse() {
        assert_eq!(
            parse("1 day + 2 months").unwrap().1,
            DateMath::Periods(Period::Day(1), vec![PeriodOp::Add(Period::Month(2))])
        );

        assert_eq!(
            parse("Jan 2, 2021 + 15 weeks").unwrap().1,
            DateMath::StartWithPeriods(
                CalculatedDate::Raw(NaiveDate::from_ymd_opt(2021, 1, 2).unwrap()),
                PeriodOp::Add(Period::Week(15)),
                vec![]
            )
        );

        assert_eq!(
            parse("2 weeks and 1 day ago").unwrap().1,
            DateMath::StartWithPeriods(
                CalculatedDate::Today,
                PeriodOp::Subtract(Period::Week(2)),
                vec![PeriodOp::Subtract(Period::Day(1))]
            )
        );

        assert_eq!(
            parse("Mar 31, 2021 + 15 weeks + 2 days").unwrap().1,
            DateMath::StartWithPeriods(
                CalculatedDate::Raw(NaiveDate::from_ymd_opt(2021, 3, 31).unwrap()),
                PeriodOp::Add(Period::Week(15)),
                vec![PeriodOp::Add(Period::Day(2))]
            )
        );

        assert_eq!(
            parse("Mar 31, 2021 - 15 weeks + 2 days").unwrap().1,
            DateMath::StartWithPeriods(
                CalculatedDate::Raw(NaiveDate::from_ymd_opt(2021, 3, 31).unwrap()),
                PeriodOp::Subtract(Period::Week(15)),
                vec![PeriodOp::Add(Period::Day(2))]
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

        assert_eq!(
            parse("Mar 31, 2021 - Mar 24, 2021").unwrap().1,
            DateMath::DateDiff(
                CalculatedDate::Raw(date(2021, 3, 31)),
                CalculatedDate::Raw(date(2021, 3, 24)),
            )
        );
    }

    #[test]
    fn test_date_math_start_with_periods() {
        let result = DateMath::StartWithPeriods(
            CalculatedDate::Raw(NaiveDate::from_ymd_opt(2021, 3, 31).unwrap()),
            PeriodOp::Add(Period::Week(15)),
            vec![
                PeriodOp::Add(Period::Day(2)),
                PeriodOp::Subtract(Period::Day(1)),
            ],
        )
        .compute(date(2022, 1, 31));

        assert_eq!(
            result,
            ComputeOutcome::Date(NaiveDate::from_ymd(2021, 7, 15))
        );
    }

    #[test]
    fn test_date_math_date_diff() {
        let result = DateMath::DateDiff(
            CalculatedDate::Raw(date(2021, 3, 31)),
            CalculatedDate::Raw(date(2021, 3, 24)),
        )
        .compute(date(2022, 1, 31));

        assert_eq!("7 days", result.to_string());
    }

    #[test]
    fn test_date_math_date_diff_no_difference() {
        let result = DateMath::DateDiff(
            CalculatedDate::Raw(date(2021, 3, 31)),
            CalculatedDate::Raw(date(2021, 3, 31)),
        )
        .compute(date(2022, 1, 31));

        assert_eq!("0 days", result.to_string());
    }

    #[test]
    fn test_date_math_date_diff_single_day_difference() {
        let result = DateMath::DateDiff(
            CalculatedDate::Raw(date(2021, 3, 31)),
            CalculatedDate::Raw(date(2021, 3, 30)),
        )
        .compute(date(2022, 1, 31));

        assert_eq!("1 day", result.to_string());
    }

    #[test]
    fn test_date_math_date_diff_negative_difference() {
        let result = DateMath::DateDiff(
            CalculatedDate::Raw(date(2021, 3, 24)),
            CalculatedDate::Raw(date(2021, 3, 31)),
        )
        .compute(date(2022, 1, 31));

        assert_eq!("7 days", result.to_string());
    }

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    fn is_parse_success(result: &ParseResult) -> bool {
        match result {
            ParseResult::Success(_) => true,
            _ => false,
        }
    }
}
