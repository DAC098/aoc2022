use std::collections::HashMap;
use std::collections::VecDeque;
use std::env::Args;

use crate::error;
use crate::cli;
use crate::io;

struct UniqueSequence {
    total: usize,
    repeating: u32,
    seq: VecDeque<char>,
    known: HashMap<char, u32>
}

impl UniqueSequence {
    fn new(total: usize) -> UniqueSequence {
        let seq = VecDeque::with_capacity(total);
        let known = HashMap::with_capacity(total);

        UniqueSequence { 
            total, 
            repeating: 0, 
            seq, 
            known 
        }
    }

    fn is_filled(&self) -> bool {
        self.seq.len() == self.total
    }

    fn has_repeating(&self) -> bool {
        self.repeating != 0
    }

    fn remove_front(&mut self) -> () {
        let dropped = self.seq.pop_front().unwrap();

        let remove = {
            let count = self.known.get_mut(&dropped).unwrap();

            *count -= 1;

            if *count == 1 {
                self.repeating -= 1;

                false
            } else {
                *count == 0
            }
        };

        if remove {
            self.known.remove(&dropped);
        }
    }

    fn add_char(&mut self, ch: char) -> () {
        if self.seq.len() == self.total {
            self.remove_front();
        }

        if let Some(count) = self.known.get_mut(&ch) {
            if *count == 1 {
                self.repeating += 1;
            }

            *count += 1;
        } else {
            self.known.insert(ch.clone(), 1);
        }

        self.seq.push_back(ch);
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
    let mut signal_data = String::new();

    let start = std::time::Instant::now();

    loop {
        let Some(line) = line_reader.next_line()? else {
            break;
        };

        if line.len() == 0 {
            return Err(error::build::bad_line_input(line_reader.get_count().clone(), line));
        }

        signal_data = line;
        break;
    }

    let mut index: usize = 0;
    let mut start_of_packet: usize = 0;
    let mut start_of_message: usize = 0;
    let mut completed: u32 = 0;
    let mut chars = signal_data.chars();
    let mut packet_seq = UniqueSequence::new(4);
    let mut message_seq = UniqueSequence::new(14);

    loop {
        let Some(ch) = chars.next() else {
            break;
        };

        index += 1;

        if start_of_packet == 0 {
            packet_seq.add_char(ch.clone());

            if cfg!(debug_assertions) {
                println!("packet seq: {:?}", packet_seq.seq);
            }

            if packet_seq.is_filled() {
                if !packet_seq.has_repeating() {
                    start_of_packet = index;

                    completed += 1;

                    if completed == 2 {
                        break;
                    }
                }
            }
        }

        if start_of_message == 0 {
            message_seq.add_char(ch.clone());

            if cfg!(debug_assertions) {
                println!("message seq: {:?}", message_seq.seq);
            }

            if message_seq.is_filled() {
                if !message_seq.has_repeating() {
                    start_of_message = index;

                    completed += 1;
                }
            }
        }

        if completed == 2 {
            break;
        }
    }

    let finish = std::time::Instant::now();

    println!("packet start: {} {:?}", start_of_packet, packet_seq.seq);
    println!("message start: {} {:?}", start_of_message, message_seq.seq);
    println!("total duration: {:#?}", finish.duration_since(start));

    Ok(())
}