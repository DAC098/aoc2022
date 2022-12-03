use std::env::Args;
use std::io::BufRead;

use crate::error;
use crate::cli;

enum Choice {
    Rock,
    Paper,
    Scissors
}

enum PlayOutcome {
    Win,
    Loose,
    Draw
}

impl Choice {
    fn try_from_str<S>(parse: S) -> Option<Choice>
    where
        S: AsRef<str>
    {
        match parse.as_ref() {
            "A" | "X" => Some(Choice::Rock),
            "B" | "Y" => Some(Choice::Paper),
            "C" | "Z" => Some(Choice::Scissors),
            _ => None
        }
    }

    fn points(&self) -> u8 {
        match self {
            Choice::Rock => 1,
            Choice::Paper => 2,
            Choice::Scissors => 3
        }
    }

    fn play(&self, against: &Self) -> PlayOutcome {
        match self {
            Choice::Rock => {
                match against {
                    Choice::Rock => PlayOutcome::Draw,
                    Choice::Paper => PlayOutcome::Loose,
                    Choice::Scissors => PlayOutcome::Win
                }
            },
            Choice::Paper => {
                match against {
                    Choice::Rock => PlayOutcome::Win,
                    Choice::Paper => PlayOutcome::Draw,
                    Choice::Scissors => PlayOutcome::Loose
                }
            },
            Choice::Scissors => {
                match against {
                    Choice::Rock => PlayOutcome::Loose,
                    Choice::Paper => PlayOutcome::Win,
                    Choice::Scissors => PlayOutcome::Draw
                }
            }
        }
    }

    fn from_outcome(&self, desired: &PlayOutcome) -> Self {
        match desired {
            PlayOutcome::Win => {
                match self {
                    Choice::Rock => Choice::Paper,
                    Choice::Paper => Choice::Scissors,
                    Choice::Scissors => Choice::Rock,
                }
            },
            PlayOutcome::Loose => {
                match self {
                    Choice::Rock => Choice::Scissors,
                    Choice::Paper => Choice::Rock,
                    Choice::Scissors => Choice::Paper
                }
            },
            PlayOutcome::Draw => {
                match self {
                    Choice::Rock => Choice::Rock,
                    Choice::Paper => Choice::Paper,
                    Choice::Scissors => Choice::Scissors
                }
            }
        }
    }
}

impl std::fmt::Display for Choice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Choice::Rock => f.write_str("Rock"),
            Choice::Paper => f.write_str("Paper"),
            Choice::Scissors => f.write_str("Sciccors")
        }
    }
}

impl PlayOutcome {
    fn try_from_str<S>(parse: S) -> Option<PlayOutcome>
    where
        S: AsRef<str>
    {
        match parse.as_ref() {
            "X" => Some(PlayOutcome::Loose),
            "Y" => Some(PlayOutcome::Draw),
            "Z" => Some(PlayOutcome::Win),
            _ => None
        }
    }

    fn points(&self) -> u8 {
        match self {
            PlayOutcome::Win => 6,
            PlayOutcome::Draw => 3,
            PlayOutcome::Loose => 0
        }
    }
}

impl std::fmt::Display for PlayOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlayOutcome::Win => f.write_str("Win"),
            PlayOutcome::Loose => f.write_str("Loose"),
            PlayOutcome::Draw => f.write_str("Draw")
        }
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

    let mut play_total_points: u32 = 0;
    let mut outcome_total_points: u32 = 0;

    let start = std::time::Instant::now();

    loop {
        line_count += 1;

        let line = {
            let Some(result) = lines.next() else {
                break;
            };

            result?
        };

        let Some((played_str, recommended_str)) = line.split_once(" ") else {
            return Err(error::build::bad_line_input(line_count, line));
        };

        let Some(played) = Choice::try_from_str(played_str) else {
            return Err(error::build::bad_line_input(line_count, line));
        };

        {// recommended play
            let Some(recommended) = Choice::try_from_str(recommended_str) else {
                return Err(error::build::bad_line_input(line_count, line));
            };

            let outcome = recommended.play(&played);

            if cfg!(debug_assertions) {
                let points = (outcome.points() as u32) + (recommended.points() as u32);

                println!("as played  {} vs {} -> {} {} points", recommended, played, outcome, points);
            }

            play_total_points += (outcome.points() as u32) + (recommended.points() as u32);
        }

        {// recommended outcome
            let Some(desired) = PlayOutcome::try_from_str(recommended_str) else {
                return Err(error::build::bad_line_input(line_count, line));
            };

            let recommended = played.from_outcome(&desired);

            if cfg!(debug_assertions) {
                let points = (desired.points() as u32) + (recommended.points() as u32);

                println!("as outcome {} vs {} -> {} {} points", recommended, played, desired, points);
            }

            outcome_total_points += (desired.points() as u32) + (recommended.points() as u32);
        }
    }

    let finish = std::time::Instant::now();

    println!("play total points: {}", play_total_points);
    println!("outcome total points: {}", outcome_total_points);
    println!("total time: {:#?}", finish.duration_since(start));

    Ok(())
}