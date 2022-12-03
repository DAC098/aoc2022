mod error;
mod file;
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
        _ => {
            Err(error::Error::new(error::ErrorKind::InvalidArgument)
                .with_message(format!("unknown day specified. given: {}", day)))
        }
    };

    if let Err(err) = result {
        print!("{}", err.kind);

        if let Some(message) = err.message {
            print!(" {}", message);
        }

        if let Some(source) = err.source {
            print!("\n{}", source);
        }

        print!("\n");
    }
}
