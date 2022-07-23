use crate::{parse, ParseResult};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Flags {
    value: String,
}

pub fn run() {
    let flags = Flags::from_args();
    let today = chrono::Local::today().naive_local();

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
