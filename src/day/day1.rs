use std::env::Args;
use std::io::BufRead;

use crate::error;
use crate::cli;

struct ElfInventory {
    total: u32,
    count: u32
}

pub fn run(mut args: Args) -> error::Result<()> {
    let mut file_path: Option<String> = None;
    let mut total_top_elves: usize = 3;

    loop {
        let Some(arg) = args.next() else {
            break;
        };

        match arg.as_str() {
            "-f" | "--file" => {
                file_path = Some(cli::get_arg_value(&mut args, "file")?);
            },
            "--top-elves" => {
                let v = cli::get_arg_value(&mut args, "top")?;

                total_top_elves = match usize::from_str_radix(&v, 10) {
                    Ok(i) => i,
                    Err(_) => {
                        return Err(error::Error::new(error::ErrorKind::InvalidArgument)
                            .with_message(format!("top elves value is not a valid usize. value: {}", v)));
                    }
                }
            },
            _ => {
                return Err(error::build::invalid_argument(arg));
            }
        }
    }

    if total_top_elves == 0 {
        return Err(error::Error::new(error::ErrorKind::InvalidArgument)
            .with_message(format!("total top elves is 0")));
    }

    let reader = cli::get_file_reader(file_path)?;
    let mut lines = reader.lines();
    let mut line_count: usize = 0;

    let mut current_index: usize = 0;
    let mut top_elves: Vec<usize> = Vec::with_capacity(total_top_elves);
    let mut elves: Vec<ElfInventory> = Vec::with_capacity(10);
    elves.push(ElfInventory {
        total: 0,
        count: 0
    });

    let start = std::time::Instant::now();

    loop {
        line_count += 1;

        let line = {
            let Some(result) = lines.next() else {
                break;
            };

            result?
        };

        if line.len() == 0 {
            for index in 0..top_elves.len() {
                if elves[current_index].total > elves[top_elves[index]].total {
                    let mut replace = current_index.clone();

                    for moving in index..top_elves.len() {
                        replace = std::mem::replace(&mut top_elves[moving], replace);
                    }

                    break;
                }
            }

            if top_elves.len() < top_elves.capacity() {
                top_elves.push(current_index.clone());
            }

            current_index += 1;

            elves.push(ElfInventory {
                total: 0, count: 0
            });

            continue;
        }

        let Ok(calories) = u32::from_str_radix(&line, 10) else {
            return Err(error::Error::new(error::ErrorKind::BadInput)
                .with_message(format!("a line in the file could not be parsed to a u32. line {} \"{}\"", line_count, line)));
        };

        if let Some(v) = elves[current_index].total.checked_add(calories) {
            elves[current_index].total = v;
            elves[current_index].count += 1;
        } else {
            return Err(error::Error::new(error::ErrorKind::BadInput)
                .with_message(format!("the total calories for an elf is larger than a u32. line {} \"{}\"", line_count, line)));
        }
    }

    for index in 0..top_elves.len() {
        if elves[current_index].total > elves[top_elves[index]].total {
            let mut replace = current_index.clone();

            for moving in index..top_elves.len() {
                replace = std::mem::replace(&mut top_elves[moving], replace);
            }

            break;
        }
    }

    if top_elves.len() < top_elves.capacity() {
        top_elves.push(current_index.clone());
    }

    let finish = std::time::Instant::now();

    println!("top elves");

    let mut total_calories: u64 = 0;

    for index in top_elves {
        println!(
            "    elf {} total: {} count: {}", 
            index + 1,
            elves[index].total,
            elves[index].count
        );

        total_calories += elves[index].total as u64;
    }

    println!("total of top elves: {}", total_calories);
    println!("total time: {:#?}", finish.duration_since(start));

    Ok(())
}