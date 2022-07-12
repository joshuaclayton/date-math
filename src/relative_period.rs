use crate::{period, Period, PeriodOp};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::space1,
    combinator::map,
    multi::many1,
    sequence::{delimited, pair, preceded, separated_pair, terminated},
    IResult,
};

pub fn parse(input: &str) -> IResult<&str, (PeriodOp, Vec<PeriodOp>)> {
    let (input, (period, rest)) = parse_sentence(input)?;

    let result = alt((
        map(tag(" ago"), |_| {
            build_period_op_pair(period, rest.clone(), PeriodOp::Subtract)
        }),
        map(tag(" from now"), |_| {
            build_period_op_pair(period, rest.clone(), PeriodOp::Add)
        }),
    ))(input);

    result
}

fn build_period_op_pair<F>(
    period: Period,
    rest: Vec<Period>,
    builder: F,
) -> (PeriodOp, Vec<PeriodOp>)
where
    F: Fn(Period) -> PeriodOp,
{
    (
        builder(period),
        rest.into_iter().map(builder).collect::<Vec<PeriodOp>>(),
    )
}

fn period_and_comma(input: &str) -> IResult<&str, Period> {
    terminated(period::parse, tag(","))(input)
}

fn parse_sentence(input: &str) -> IResult<&str, (Period, Vec<Period>)> {
    let comma_delimited = map(
        separated_pair(
            pair(period_and_comma, many1(preceded(space1, period_and_comma))),
            delimited(space1, tag("and"), space1),
            period::parse,
        ),
        |((period, mut rest), last)| {
            rest.extend([last]);
            (period, rest)
        },
    );

    let single_and = map(
        separated_pair(
            period::parse,
            delimited(space1, tag("and"), space1),
            period::parse,
        ),
        |(period, other)| (period, vec![other]),
    );

    let single = map(period::parse, |period| (period, vec![]));

    alt((comma_delimited, single_and, single))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_human_subtract() {
        assert_eq!(
            parse("3 days ago").unwrap().1,
            (PeriodOp::Subtract(Period::Day(3)), Vec::new())
        );

        assert_eq!(
            parse("12 years ago").unwrap().1,
            (PeriodOp::Subtract(Period::Year(12)), Vec::new())
        );
    }

    #[test]
    fn test_human_pair() {
        assert_eq!(
            parse("3 days and 1 week ago").unwrap().1,
            (
                PeriodOp::Subtract(Period::Day(3)),
                vec![PeriodOp::Subtract(Period::Week(1))]
            )
        );
    }

    #[test]
    fn test_human_sentence() {
        assert_eq!(
            parse_sentence("1 year, 2 months, and 3 days").unwrap().1,
            (Period::Year(1), vec![Period::Month(2), Period::Day(3)])
        );
    }

    #[test]
    fn test_human_multiple() {
        assert_eq!(
            parse("1 year, 2 months, and 3 days from now").unwrap().1,
            (
                PeriodOp::Add(Period::Year(1)),
                vec![
                    PeriodOp::Add(Period::Month(2)),
                    PeriodOp::Add(Period::Day(3))
                ]
            )
        );
    }

    #[test]
    fn test_human_add() {
        assert_eq!(
            parse("3 days from now").unwrap().1,
            (PeriodOp::Add(Period::Day(3)), Vec::new())
        );

        assert_eq!(
            parse("12 weeks from now").unwrap().1,
            (PeriodOp::Add(Period::Week(12)), Vec::new())
        );
    }
}
