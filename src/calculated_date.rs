use chrono::{format, Datelike, Duration, NaiveDate};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_till},
    combinator::{map, map_opt, value},
    IResult,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CalculatedDate {
    Today,
    Yesterday,
    Tomorrow,
    Raw(NaiveDate),
}

impl CalculatedDate {
    pub fn calculate(&self, today: NaiveDate) -> NaiveDate {
        match self {
            CalculatedDate::Raw(v) => *v,
            CalculatedDate::Today => today,
            CalculatedDate::Yesterday => today - Duration::days(1),
            CalculatedDate::Tomorrow => today + Duration::days(1),
        }
    }
}

pub fn parse(input: &str) -> IResult<&str, CalculatedDate> {
    alt((
        value(CalculatedDate::Today, tag("today")),
        value(CalculatedDate::Today, tag("now")),
        value(CalculatedDate::Yesterday, tag("yesterday")),
        value(CalculatedDate::Tomorrow, tag("tomorrow")),
        map(
            map_opt(take_till(|c: char| c == '+' || c == '-'), parse_date),
            CalculatedDate::Raw,
        ),
    ))(input)
}

pub(crate) fn parse_date(value: &str) -> Option<NaiveDate> {
    let value = value.trim();
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .or(NaiveDate::parse_from_str(value, "%h %d, %Y"))
        .or(NaiveDate::parse_from_str(value, "%B %d"))
        .or(NaiveDate::parse_from_str(value, "%B %d, %Y"))
        .or(NaiveDate::parse_from_str(value, "%m/%d/%Y"))
        .ok()
        .or(parse_partial_date(value))
}

fn parse_partial_date(value: &str) -> Option<NaiveDate> {
    let mut parsed = format::Parsed::new();
    let long_month_name_format = vec![
        format::Item::Fixed(format::Fixed::LongMonthName),
        format::Item::Space(" "),
        format::Item::Numeric(format::Numeric::Day, format::Pad::None),
    ];

    if format::parse(&mut parsed, value, long_month_name_format.iter()).is_ok() {
        match (parsed.month, parsed.day) {
            (Some(m), Some(d)) => NaiveDate::from_ymd_opt(chrono::Local::today().year(), m, d),
            _ => None,
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_and_calculate(value: &str, today: NaiveDate) -> NaiveDate {
        parse(value).unwrap().1.calculate(today)
    }

    #[test]
    fn test_date_relative() {
        let date = NaiveDate::from_ymd_opt(2022, 1, 31).unwrap();
        let one_day = Duration::days(1);

        assert_eq!(parse_and_calculate("today", date), date);
        assert_eq!(parse_and_calculate("now", date), date);
        assert_eq!(parse_and_calculate("yesterday", date), date - one_day);
        assert_eq!(parse_and_calculate("tomorrow", date), date + one_day);
    }

    #[test]
    fn test_date_parse() {
        assert_eq!(
            parse_date("2021-01-31"),
            NaiveDate::from_ymd_opt(2021, 1, 31)
        );

        assert_eq!(
            parse_date(" 2021-01-31  "),
            NaiveDate::from_ymd_opt(2021, 1, 31)
        );

        assert_eq!(
            parse_date("Jan 31, 2021"),
            NaiveDate::from_ymd_opt(2021, 1, 31)
        );

        assert_eq!(
            parse_date("Jan 1, 2021"),
            NaiveDate::from_ymd_opt(2021, 1, 1)
        );

        assert_eq!(
            parse_date("jan 1, 2021"),
            NaiveDate::from_ymd_opt(2021, 1, 1)
        );

        assert_eq!(
            parse_date("january 1, 2021"),
            NaiveDate::from_ymd_opt(2021, 1, 1)
        );

        assert_eq!(
            parse_date("january 1"),
            NaiveDate::from_ymd_opt(chrono::Local::today().year(), 1, 1)
        );

        assert_eq!(
            parse_date("apr 30"),
            NaiveDate::from_ymd_opt(chrono::Local::today().year(), 4, 30)
        );

        assert_eq!(
            parse_date("Jan 01, 2021"),
            NaiveDate::from_ymd_opt(2021, 1, 1)
        );

        assert_eq!(
            parse_date("1/31/2021"),
            NaiveDate::from_ymd_opt(2021, 1, 31)
        );
    }
}
