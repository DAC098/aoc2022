use std::env::Args;
use std::io::BufRead;

use crate::error;
use crate::cli;

struct Range {
    lower: u32,
    upper: u32
}

enum FromStrError {
    InvalidFormat,
    InvalidInteger,
    InvalidLowerBound,
    InvalidUpperBound
}

impl Range {
    fn from_str<S>(string: S) -> std::result::Result<Self, FromStrError>
    where
        S: AsRef<str>
    {
        let str_ref = string.as_ref();
        let Some((lower_str, upper_str)) = str_ref.split_once('-') else {
            return Err(FromStrError::InvalidFormat);
        };

        let Ok(lower) = u32::from_str_radix(lower_str, 10) else {
            return Err(FromStrError::InvalidInteger);
        };
        let Ok(upper) = u32::from_str_radix(upper_str, 10) else {
            return Err(FromStrError::InvalidInteger);
        };

        if lower > upper {
            Err(FromStrError::InvalidLowerBound)
        } else if upper < lower {
            Err(FromStrError::InvalidUpperBound)
        } else {
            Ok(Range { lower, upper })
        }
    }

    fn contains(&self, check: &Self) -> bool {
        self.lower <= check.lower && self.upper >= check.upper
    }

    fn overlaps(&self, check: &Self) -> bool {
        (self.lower >= check.lower && self.lower <= check.upper) ||
        (self.upper <= check.lower && self.upper >= check.upper)
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
                return Err(error::build::invalid_argument(arg))
             }
        }
    }

    let reader = cli::get_file_reader(file_path)?;
    let mut lines = reader.lines();
    let mut line_count: usize = 0;

    let mut total_contained_pairs: u32 = 0;
    let mut total_overlapping_pairs: u32 = 0;

    let start = std::time::Instant::now();

    loop {
        line_count += 1;

        let line = {
            let Some(result) = lines.next() else {
                break;
            };

            result?
        };

        let Some((first_pair, second_pair)) = line.split_once(',') else {
            return Err(error::Error::new(error::ErrorKind::BadInput)
                .with_message(format!("a line in the file is not formatted properly. line {} \"{}\"", line_count, line)));
        };

        let Ok(first_range) = Range::from_str(first_pair) else {
            return Err(error::Error::new(error::ErrorKind::BadInput)
                .with_message(format!("a line in the file is not formatted properly. line {} \"{}\"", line_count, line)))
        };
        let Ok(second_range) = Range::from_str(second_pair) else {
            return Err(error::Error::new(error::ErrorKind::BadInput)
                .with_message(format!("a line in the file is not formatted properly. line {} \"{}\"", line_count, line)))
        };

        if first_range.contains(&second_range) || second_range.contains(&first_range) {
            total_contained_pairs += 1;
        }

        if first_range.overlaps(&second_range) || second_range.overlaps(&first_range) {
            total_overlapping_pairs += 1;
        }
    }

    let finish = std::time::Instant::now();

    println!("total contained pairs: {}", total_contained_pairs);
    println!("total overlapping pairs: {}", total_overlapping_pairs);
    println!("total time: {:#?}", finish.duration_since(start));

    Ok(())
}