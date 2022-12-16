use std::env::Args;

use crate::error;
use crate::cli;
use crate::io;

struct Operation {
    amount: usize,
    from: usize,
    to: usize
}

enum FromStrError {
    InvalidInteger,
    InvalidFormat
}

impl Operation {
    fn from_str<S>(string: S) -> std::result::Result<Self, FromStrError>
    where
        S: AsRef<str>
    {
        let str_ref = string.as_ref();

        let mut splits = str_ref.split(' ');
        // ignore the command
        splits.next();
        // amount to move
        let amount = {
            let Some(amount_str) = splits.next() else {
                return Err(FromStrError::InvalidFormat);
            };
    
            let Ok(amount) = usize::from_str_radix(amount_str, 10) else {
                return Err(FromStrError::InvalidInteger);
            };

            amount
        };

        // from column
        let from = {
            let Some(_dest_str) = splits.next() else {
                return Err(FromStrError::InvalidFormat);
            };

            let Some(column_str) = splits.next() else {
                return Err(FromStrError::InvalidFormat);
            };

            let Ok(column) = usize::from_str_radix(column_str, 10) else {
                return Err(FromStrError::InvalidInteger);
            };

            column - 1
        };

        // to column
        let to = {
            let Some(_dest_str) = splits.next() else {
                return Err(FromStrError::InvalidFormat);
            };

            let Some(column_str) = splits.next() else {
                return Err(FromStrError::InvalidFormat);
            };

            let Ok(column) = usize::from_str_radix(column_str, 10) else {
                return Err(FromStrError::InvalidInteger);
            };

            column - 1
        };

        Ok(Operation { amount, from, to })
    }
}

fn next_token(iter: &mut impl Iterator<Item = char>) -> Option<String> {
    let mut collected = String::new();

    while let Some(ch) = iter.next() {
        if ch.is_ascii_whitespace() {
            if collected.len() > 0 {
                collected.shrink_to_fit();

                return Some(collected);
            }

            continue;
        } else {
            if collected.len() + 1 == collected.capacity() {
                collected.reserve(10);
            }

            collected.push(ch);
        }
    }

    if collected.len() > 0 {
        collected.shrink_to_fit();

        Some(collected)
    } else {
        None
    }
}

pub fn run(mut args: Args) -> error::Result<()> {
    let mut file_path: Option<String> = None;

    loop {
        let Some(arg) = args.next() else {
            break;
        };

        match arg.as_str() {
            "-f" | "--file" => {
                file_path = Some(cli::get_arg_value(&mut args, "file")?);
            },
            _ => {
                return Err(error::build::invalid_argument(arg));
            }
        }
    }

    let reader = cli::get_file_reader(file_path)?;
    let mut line_reader = io::LineReader::new(reader);

    let mut cargo_lines: Vec<String> = Vec::new();

    let start = std::time::Instant::now();

    loop {
        let Some(line) = line_reader.next_line()? else {
            break;
        };

        if line.len() == 0 {
            break;
        }

        cargo_lines.push(line);
    }

    // get the last line to determine how many columns there are
    let Some(column_str) = cargo_lines.last() else {
        return Err(error::Error::new(error::ErrorKind::BadInput)
            .with_message("no cargo lines specified"));
    };

    let mut chars = column_str.chars();
    let mut count: usize = 0;
    let mut prev_digit: u32 = 0;

    while let Some(token) = next_token(&mut chars) {
        if cfg!(debug_assertions) {
            println!("token: \"{}\"", token);
        }

        let Ok(num) = u32::from_str_radix(&token, 10) else {
            return Err(error::build::bad_line_input(cargo_lines.len(), column_str))
        };

        if num <= prev_digit {
            return Err(error::build::bad_line_input(cargo_lines.len(), column_str));
        }

        count += 1;
        prev_digit = num;
    }

    let mut running: usize = 0;
    let mut p1_columns: Vec<Vec<char>> = Vec::with_capacity(count);

    for _ in 0..count {
        p1_columns.push(Vec::new())
    }

    for index in (0..(cargo_lines.len() - 1)).rev() {
        let mut chars = cargo_lines[index].chars();
        chars.next();
        
        if cfg!(debug_assertions) {
            println!("cargo line: \"{}\"", cargo_lines[index]);
        }

        'outer: while running < count {
            let Some(ch) = chars.next() else {
                break;
            };

            if cfg!(debug_assertions) {
                println!("found crate: \"{}\"", ch);
            }

            if ch.is_alphabetic() {
                p1_columns[running].push(ch);
            }

            running += 1;

            for _ in 0..3 {
                if let None = chars.next() {
                    break 'outer;
                }
            }
        }

        running = 0;
    }

    let mut p2_columns = p1_columns.clone();
    let mut intermediate: Vec<char> = Vec::new();

    loop {
        let Some(line) = line_reader.next_line()? else {
            break;
        };

        let Ok(op) = Operation::from_str(&line) else {
            return Err(error::build::bad_line_input(line_reader.get_count().clone(), line));
        };

        {
            if op.from >= p1_columns.len() || op.to >= p1_columns.len() {
                return Err(error::build::bad_line_input(line_reader.get_count().clone(), line));
            };

            // check array sizes and make sure they have capacity
            let avail = p1_columns[op.to].capacity() - p1_columns[op.to].len();

            if avail < op.amount {
                p1_columns[op.to].reserve(op.amount - avail);
            }

            for _ in 0..op.amount {
                let Some(m) = p1_columns[op.from].pop() else {
                    return Err(error::build::bad_line_input(line_reader.get_count().clone(), line));
                };

                p1_columns[op.to].push(m);
            }
        }

        {
            // this is probably pointless
            if op.from >= p2_columns.len() || op.to >= p2_columns.len() {
                return Err(error::build::bad_line_input(line_reader.get_count().clone(), line));
            };

            // check array sizes and make sure they have capacity
            let avail = p2_columns[op.to].capacity() - p2_columns[op.to].len();

            if avail < op.amount {
                p2_columns[op.to].reserve(op.amount - avail);
            }

            if intermediate.capacity() < op.amount {
                intermediate.reserve(op.amount - intermediate.capacity());
            }

            for _ in 0..op.amount {
                let Some(m) = p2_columns[op.from].pop() else {
                    return Err(error::build::bad_line_input(line_reader.get_count().clone(), line));
                };

                intermediate.push(m);
            }

            for _ in 0..intermediate.len() {
                let m = intermediate.pop().unwrap();

                p2_columns[op.to].push(m);
            }
        }
    }

    let finish = std::time::Instant::now();

    let mut p1_output = String::new();
    let mut p2_output = String::new();

    for index in 0..p1_columns.len() {
        let Some(ch) = p1_columns[index].last() else {
            return Err(error::Error::new(error::ErrorKind::Unexpected)
                .with_message("a column has no value"));
        };

        p1_output.push(ch.clone());
    }

    for index in 0..p2_columns.len() {
        let Some(ch) = p2_columns[index].last() else {
            return Err(error::Error::new(error::ErrorKind::Unexpected)
                .with_message("a column has no value"));
        };

        p2_output.push(ch.clone());
    }

    println!("p1 end sequence: \"{}\"", p1_output);
    println!("p2 end sequence: \"{}\"", p2_output);
    println!("total duration: {:#?}", finish.duration_since(start));

    Ok(())
}