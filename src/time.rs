use crate::parser_utils::parse_digits;
use chrono::NaiveTime;
use nom::{
    branch::alt,
    bytes::complete::{tag, take},
    character::complete::space0,
    combinator::{eof, map, map_opt, map_res, opt, value, verify},
    sequence::{pair, preceded, terminated, tuple},
    IResult,
};

pub enum TimeValue {
    Hour(u32, AmPm),
    HourMinute(u32, u32, AmPm),
    HourMinuteSecond(u32, u32, u32, AmPm),
}

impl TimeValue {
    fn to_naive_time(&self) -> Option<NaiveTime> {
        match self {
            TimeValue::Hour(hour, AmPm::Am) => NaiveTime::from_hms_opt(*hour, 0, 0),
            TimeValue::Hour(hour, AmPm::Pm) if hour < &12 => {
                NaiveTime::from_hms_opt(hour + 12, 0, 0)
            }
            TimeValue::Hour(hour, AmPm::Pm) => NaiveTime::from_hms_opt(*hour, 0, 0),

            TimeValue::HourMinute(hour, minute, AmPm::Am) => {
                NaiveTime::from_hms_opt(*hour, *minute, 0)
            }
            TimeValue::HourMinute(hour, minute, AmPm::Pm) if hour < &12 => {
                NaiveTime::from_hms_opt(hour + 12, *minute, 0)
            }
            TimeValue::HourMinute(hour, minute, AmPm::Pm) => {
                NaiveTime::from_hms_opt(*hour, *minute, 0)
            }
            TimeValue::HourMinuteSecond(hour, minute, second, AmPm::Am) => {
                NaiveTime::from_hms_opt(*hour, *minute, *second)
            }
            TimeValue::HourMinuteSecond(hour, minute, second, AmPm::Pm) if hour < &12 => {
                NaiveTime::from_hms_opt(hour + 12, *minute, *second)
            }
            TimeValue::HourMinuteSecond(hour, minute, second, AmPm::Pm) => {
                NaiveTime::from_hms_opt(*hour, *minute, *second)
            }
        }
    }
}

fn parse_24_hours_minutes(input: &str) -> IResult<&str, (u32, u32)> {
    pair(terminated(parse_24_hour, opt(tag(":"))), parse_0_60)(input)
}

fn parse_hours_minutes(input: &str) -> IResult<&str, (u32, u32)> {
    pair(terminated(parse_12_hour, opt(tag(":"))), parse_0_60)(input)
}

fn parse_hours_minutes_seconds(input: &str) -> IResult<&str, (u32, u32, u32)> {
    tuple((
        terminated(parse_12_hour, tag(":")),
        terminated(parse_0_60, tag(":")),
        parse_0_60,
    ))(input)
}

pub fn parse(input: &str) -> IResult<&str, NaiveTime> {
    alt((
        map_opt(parse_military_time, |v| v.to_naive_time()),
        map_opt(
            pair(parse_12_hour, preceded(space0, parse_am_pm)),
            |(h, ampm)| TimeValue::Hour(h, ampm).to_naive_time(),
        ),
        map_opt(
            pair(parse_hours_minutes, preceded(space0, parse_am_pm)),
            |((h, m), ampm)| TimeValue::HourMinute(h, m, ampm).to_naive_time(),
        ),
        map_opt(terminated(parse_24_hours_minutes, eof), |(h, m)| {
            if h > 12 {
                TimeValue::HourMinute(h - 12, m, AmPm::Pm).to_naive_time()
            } else {
                TimeValue::HourMinute(h, m, AmPm::Am).to_naive_time()
            }
        }),
        map_opt(
            pair(parse_hours_minutes_seconds, preceded(space0, parse_am_pm)),
            |((h, m, s), ampm)| TimeValue::HourMinuteSecond(h, m, s, ampm).to_naive_time(),
        ),
    ))(input)
}

#[derive(Clone, Debug)]
pub enum AmPm {
    Am,
    Pm,
}

fn parse_military_time(input: &str) -> IResult<&str, TimeValue> {
    map(pair(parse_24_hour_two_digit, parse_0_60), |(h, m)| {
        if h > 12 {
            TimeValue::HourMinute(h - 12, m, AmPm::Pm)
        } else {
            TimeValue::HourMinute(h, m, AmPm::Am)
        }
    })(input)
}

fn is_within_24_hours(v: u32) -> bool {
    v < 24
}

fn is_within_12_hours(v: u32) -> bool {
    v < 13
}

fn is_within_0_60(v: u32) -> bool {
    v < 60
}

fn parse_12_hour(input: &str) -> IResult<&str, u32> {
    alt((
        verify(parse_two_digits, |v| is_within_12_hours(*v)),
        verify(parse_one_digit, |v| is_within_12_hours(*v)),
    ))(input)
}

fn parse_24_hour_two_digit(input: &str) -> IResult<&str, u32> {
    verify(parse_two_digits, |v| is_within_24_hours(*v))(input)
}

fn parse_24_hour(input: &str) -> IResult<&str, u32> {
    verify(parse_digits, |v| is_within_24_hours(*v))(input)
}

fn parse_0_60(input: &str) -> IResult<&str, u32> {
    verify(parse_two_digits, |v| is_within_0_60(*v))(input)
}

fn parse_two_digits<T: std::str::FromStr>(input: &str) -> IResult<&str, T> {
    map_res(take(2usize), str::parse)(input)
}

fn parse_one_digit<T: std::str::FromStr>(input: &str) -> IResult<&str, T> {
    map_res(take(1usize), str::parse)(input)
}

fn parse_am_pm(input: &str) -> IResult<&str, AmPm> {
    alt((
        value(AmPm::Am, tag("am")),
        value(AmPm::Am, tag("AM")),
        value(AmPm::Am, tag("Am")),
        value(AmPm::Am, tag("aM")),
        value(AmPm::Am, tag("a")),
        value(AmPm::Am, tag("A")),
        value(AmPm::Pm, tag("pm")),
        value(AmPm::Pm, tag("PM")),
        value(AmPm::Pm, tag("Pm")),
        value(AmPm::Pm, tag("pM")),
        value(AmPm::Pm, tag("p")),
        value(AmPm::Pm, tag("P")),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_smoke() {
        let examples = vec![
            "3pm",
            "9am",
            "09am",
            "12:15pm",
            "15:30",
            "1530",
            "0930",
            "930a",
            "0030",
            "3 pm",
            "9 am",
            "09 am",
            "12:15 pm",
            "930 a",
            "12:00:30pm",
            "2359",
        ];

        assert!(examples.iter().map(|v| parse(v)).all(|v| v.is_ok()));

        let failures = vec!["13pm", "13:20pm", "19:20am", "17", "369"];

        assert!(failures.iter().map(|v| parse(v)).all(|v| v.is_err()));
    }

    #[test]
    fn test_time_military() {
        assert_eq!(parse("1330").unwrap().1, NaiveTime::from_hms(13, 30, 0));
    }
}
