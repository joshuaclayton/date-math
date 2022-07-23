use crate::{calculated_date, parse, ParseResult};
use chrono::NaiveDate;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Flags {
    value: String,
}

pub fn run() {
    let flags = Flags::from_args();
    let today = today_from_env().unwrap_or(chrono::Local::today().naive_local());

    match parse(&flags.value).into() {
        ParseResult::Success(math) => println!("{}", math.compute(today)),
        ParseResult::PartialSuccess(math, unparsed) => {
            eprintln!("Unparsed input: '{}'", unparsed);
            println!("{}", math.compute(today));
        }
        ParseResult::Error(e) => {
            eprintln!("{}", e);
            std::process::exit(1)
        }
    }
}

fn today_from_env() -> Option<NaiveDate> {
    std::env::var("TODAY")
        .ok()
        .and_then(|v| calculated_date::parse_date(&v))
}
