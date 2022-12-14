mod error;
mod io;
mod cli;
mod day;

fn main() {
    let mut args = std::env::args();
    args.next();

    let Some(day) = args.next() else {
        println!("no day specified");
        return;
    };

    let result = match day.as_str() {
        "1" => day::day1::run(args),
        "2" => day::day2::run(args),
        "3" => day::day3::run(args),
        "4" => day::day4::run(args),
        "5" => day::day5::run(args),
        "6" => day::day6::run(args),
        _ => {
            Err(error::Error::new(error::ErrorKind::InvalidArgument)
                .with_message(format!("unknown day specified. given: {}", day)))
        }
    };

    if let Err(err) = result {
        print!("{}", err.kind);

        if let Some(message) = err.message {
            print!(": {}", message);
        }

        if let Some(source) = err.source {
            print!("\n{}", source);
        }

        print!("\n");
    }
}
