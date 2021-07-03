use crate::parse;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Flags {
    value: String,
}

pub fn run() {
    let flags = Flags::from_args();

    match parse(&flags.value) {
        Ok(("", math)) => println!("{}", math.compute()),
        Ok((unparsed, math)) => {
            eprintln!("Unparsed input: '{}'", unparsed);
            println!("{}", math.compute())
        }
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1)
        }
    }
}
