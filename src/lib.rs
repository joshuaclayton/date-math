mod calculated_date;
pub mod cli;
mod period;
mod period_operation;

use calculated_date::CalculatedDate;
use period::Period;
use period_operation::PeriodOp;

use chrono::NaiveDate;
use nom::{branch::alt, combinator::map, multi::many0, sequence::pair, IResult};

#[derive(Debug, PartialEq, Eq)]
pub enum DateMath {
    Periods(Period, Vec<PeriodOp>),
    Start(CalculatedDate),
    StartWithPeriods(CalculatedDate, PeriodOp, Vec<PeriodOp>),
}

impl DateMath {
    pub fn compute(&self) -> NaiveDate {
        match self {
            DateMath::Start(v) => v.calculate(),
            DateMath::StartWithPeriods(v, base, rest) => rest
                .iter()
                .fold(base.apply(v.calculate()), |acc, x| x.apply(acc)),
            DateMath::Periods(base, rest) => rest.iter().fold(
                chrono::Local::today().naive_local() + base.to_duration(),
                |acc, x| x.apply(acc),
            ),
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
        .compute();

        assert_eq!(result, NaiveDate::from_ymd(2021, 7, 15));
    }
}
