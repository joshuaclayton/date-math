use crate::{parse, ParseResult};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Flags {
    value: String,
}

pub fn run() {
    let flags = Flags::from_args();

    match parse(&flags.value).into() {
        ParseResult::Success(math) => println!("{}", math.compute()),
        ParseResult::PartialSuccess(math, unparsed) => {
            eprintln!("Unparsed input: '{}'", unparsed);
            println!("{}", math.compute());
        }
        ParseResult::Error(e) => {
            eprintln!("{}", e);
            std::process::exit(1)
        }
    }
}
