use std::collections::HashMap;
use std::collections::HashSet;
use std::env::Args;
use std::io::BufRead;

use crate::error;
use crate::cli;

fn get_item_value(ch_int: &u32) -> Option<u32> {
    if *ch_int >= 0x61 && *ch_int <= 0x7a {
        // lower case letter
        Some(*ch_int - 0x60)
    } else if *ch_int >= 0x41 && *ch_int <= 0x5a {
        // upper case letter
        Some(*ch_int - 0x26)
    } else {
        None
    }
}

fn get_char_value(item: &u32) -> u32 {
    if *item < 27 {
        // lower case letter
        *item + 0x60
    } else {
        // upper case letter
        *item + 0x26
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
    let mut lines = reader.lines();
    let mut line_count: usize = 0;

    let mut total: u32 = 0;
    let mut badge_total: u32 = 0;
    let mut inventory_group: HashMap<u32, u8> = HashMap::new();
    let mut flag_id: u8 = 0b001;

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
            return Err(error::Error::new(error::ErrorKind::BadInput)
                .with_message(format!("a line in the file has no characters. line {}", line_count)));
        }

        if cfg!(debug_assertions) {
            println!("inventory: {} {}\nflag_id: {}", line, line.len(), flag_id);
        }

        let (comp_one, comp_two) = line.split_at(line.len() / 2);
        let mut comp_one_set: HashSet<u32> = HashSet::with_capacity(comp_one.len());
        let mut comp_two_set: HashSet<u32> = HashSet::with_capacity(comp_two.len());

        if cfg!(debug_assertions) {
            println!("    comp one: {}\n    comp two: {}", comp_one, comp_two);
        }

        for ch in comp_one.chars() {
            let ch_int: u32 = ch.into();

            if let Some(_) = get_item_value(&ch_int) {
                if let Some(ids) = inventory_group.get_mut(&ch_int) {
                    *ids |= flag_id;
                } else {
                    inventory_group.insert(ch_int, flag_id);
                }

                comp_one_set.insert(ch_int);
            } else {
                return Err(error::Error::new(error::ErrorKind::BadInput)
                    .with_message(format!("a line in the file contains invalid characters. line {} \"{}\"", line_count, line)));
            }
        }

        for ch in comp_two.chars() {
            let ch_int: u32 = ch.into();

            if let Some(item_value) = get_item_value(&ch_int) {
                if let Some(ids) = inventory_group.get_mut(&ch_int) {
                    *ids |= flag_id;
                } else {
                    inventory_group.insert(ch_int, flag_id);
                }

                if !comp_two_set.insert(ch_int) {
                    continue;
                }

                if comp_one_set.contains(&ch_int) {
                    if cfg!(debug_assertions) {
                        println!("    duplicate item found: {} {}", ch, item_value);
                    }

                    if let Some(v) = total.checked_add(item_value) {
                        total = v;
                    } else {
                        return Err(error::Error::new(error::ErrorKind::BadInput)
                            .with_message(format!("total count of duplicate items is larger than a u32. line {} \"{}\"", line_count, line)))
                    }
                }
            } else {
                return Err(error::Error::new(error::ErrorKind::BadInput)
                    .with_message(format!("a line in the file contains invalid characters. line {} \"{}\"", line_count, line)));
            }
        }

        if cfg!(debug_assertions) {
            println!("current total: {}", total);
        }

        if flag_id == 0b100 {
            if cfg!(debug_assertions) {
                println!("checking group inventory")
            }

            for (key, value) in &inventory_group {
                if cfg!(debug_assertions) {
                    println!("    item: {} ids: {:03b}", char::try_from(key.clone()).unwrap(), value);
                }

                if *value == 0b111 {
                    let item_value = get_item_value(key).unwrap();

                    if cfg!(debug_assertions) {
                        println!("    duplicate item found: {} {}", char::try_from(key.clone()).unwrap(), item_value);
                    }

                    if let Some(v) = badge_total.checked_add(item_value) {
                        badge_total = v;
                    } else {
                        return Err(error::Error::new(error::ErrorKind::BadInput)
                            .with_message("total count of badge is larger than a u32."))
                    }
                }
            }

            inventory_group.clear();
            flag_id = 0b001;
        } else {
            flag_id <<= 1;
        }
    }

    if flag_id == 0b100 {
        if cfg!(debug_assertions) {
            println!("checking group inventory")
        }

        for (key, value) in inventory_group {
            if cfg!(debug_assertions) {
                println!("    item: {} ids: {:03b}", char::try_from(key.clone()).unwrap(), value);
            }

            if value == 0b111 {
                let item_value = get_item_value(&key).unwrap();

                if cfg!(debug_assertions) {
                    println!("    duplicate item found: {} {}", char::try_from(key).unwrap(), item_value);
                }

                if let Some(v) = badge_total.checked_add(item_value) {
                    badge_total = v;
                } else {
                    return Err(error::Error::new(error::ErrorKind::BadInput)
                        .with_message("total count of badge is larger than a u32."))
                }
            }
        }
    }

    let finish = std::time::Instant::now();

    println!("total: {}", total);
    println!("badge total: {}", badge_total);
    println!("total time: {:#?}", finish.duration_since(start));

    Ok(())
}