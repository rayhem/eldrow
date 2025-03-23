use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::str::FromStr;
use clap::Parser;

#[derive(Clone, Debug, Parser)]
struct Args {
    #[arg(short, long)]
    wordlist: std::path::PathBuf,

    #[arg(short, long, default_value_t = 5)]
    length: usize
}

#[derive(Clone, Debug)]
enum Command {
    Contains(String),
    Guess(String, Vec<usize>, Vec<usize>),
    Prune(String),
    PruneAt(char, usize),
    Require(String),
    RequireAt(char, usize),
}

#[derive(Clone, Copy, Debug)]
enum CommandError {
    ImproperArgumentCount,
    MalformedArgument,
    InvalidCommand,
}

impl FromStr for Command {
    type Err = CommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use CommandError::*;

        let (cmd, args) = s.trim().split_once(' ').ok_or(ImproperArgumentCount)?;
        match cmd {
            "contains" => Ok(Command::Contains(args.to_string())),
            "guess" | "g" => {
                let to_vec =
                    |s: &str| Vec::from_iter(s.chars().map(|c| c.to_digit(10).unwrap() as usize));

                let (word, indices) = args.split_once(' ').ok_or(ImproperArgumentCount)?;
                let (correct, incorrect) =
                    indices.trim().split_once(',').ok_or(MalformedArgument)?;
                Ok(Command::Guess(
                    word.to_string(),
                    to_vec(correct),
                    to_vec(incorrect),
                ))
            }
            "prune" | "p" => {
                let mut tokens = args.split(" at ");
                let chars = tokens.next().ok_or(ImproperArgumentCount)?;
                if let Some(location) = tokens.next() {
                    Ok(Command::PruneAt(
                        chars.chars().next().ok_or(ImproperArgumentCount)?,
                        location.parse().map_err(|_| MalformedArgument)?,
                    ))
                } else {
                    Ok(Command::Prune(chars.to_string()))
                }
            }
            "require" | "r" => {
                let mut tokens = args.split(" at ");
                let chars = tokens.next().ok_or(ImproperArgumentCount)?;
                if let Some(location) = tokens.next() {
                    Ok(Command::RequireAt(
                        chars.chars().next().ok_or(ImproperArgumentCount)?,
                        location.parse().map_err(|_| MalformedArgument)?,
                    ))
                } else {
                    Ok(Command::Require(chars.to_string()))
                }
            }
            _ => Err(InvalidCommand),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tile {
    Correct(char),
    Incorrect(char),
    Unused(char),
    Unchecked(char),
}

fn main() {
    let args = Args::parse();
    
    let mut words: HashSet<_> = std::fs::read_to_string(args.wordlist)
        .expect("Could not open wordlist")
        .lines()
        .filter(|word| word.chars().count() == args.length)
        .map(|word| word.trim().to_ascii_lowercase())
        .collect();

    println!("Wordlist contains {} words", words.len());

    let mut guesses: Vec<String> = Vec::new();

    while words.len() > 1 {
        println!("Guesses: {:?}", guesses);

        print!("$ ");
        std::io::stdout().flush().unwrap();
        let mut command_string = String::new();
        std::io::stdin().read_line(&mut command_string).unwrap();
        let command = Command::from_str(&command_string);

        if command.is_err() {
            println!("Invalid command (reason: {:?})", command.unwrap_err());
            continue;
        }

        match command.unwrap() {
            Command::Contains(word) => {
                println!("{}", words.contains(&word));
                continue;
            }
            Command::Prune(chars) => {
                chars.chars().for_each(|ch| prune(&mut words, ch));
            }
            Command::PruneAt(ch, idx) => {
                prune_at(&mut words, ch, idx);
            }
            Command::Require(chars) => {
                chars.chars().for_each(|ch| require(&mut words, ch));
            }
            Command::RequireAt(ch, idx) => {
                require_at(&mut words, ch, idx);
            }
            Command::Guess(arg, correct_indices, incorrect_indices) => {
                let guess = arg.trim().to_string();

                guesses.push(guess.clone());
                let mut tiles: Vec<_> = guess.chars().map(Tile::Unchecked).collect();

                correct_indices.into_iter().for_each(|i| match tiles[i] {
                    Tile::Unchecked(c) => tiles[i] = Tile::Correct(c),
                    _ => unreachable!(),
                });

                incorrect_indices.into_iter().for_each(|i| match tiles[i] {
                    Tile::Unchecked(c) => tiles[i] = Tile::Incorrect(c),
                    _ => unreachable!(),
                });

                for tile in tiles.iter_mut() {
                    if let Tile::Unchecked(c) = *tile {
                        *tile = Tile::Unused(c);
                    }
                }

                let marked_tiles: Vec<char> = tiles
                    .iter()
                    .filter_map(|tile| match tile {
                        Tile::Correct(c) => Some(*c),
                        Tile::Incorrect(c) => Some(*c),
                        _ => None,
                    })
                    .collect();

                for char in tiles.iter().filter_map(|tile| match tile {
                    Tile::Unused(c) => Some(c),
                    _ => None,
                }) {
                    if marked_tiles.len() == 0 || !marked_tiles.contains(char) {
                        prune(&mut words, *char);
                    }
                }

                for (i, tile) in tiles.iter().enumerate() {
                    match *tile {
                        Tile::Correct(c) => require_at(&mut words, c, i),
                        Tile::Incorrect(c) => {
                            prune_at(&mut words, c, i);
                            require(&mut words, c);
                        }
                        Tile::Unused(c) => prune_at(&mut words, c, i),
                        Tile::Unchecked(_) => unreachable!(),
                    }
                }
            }
        }

        println!("{:?}", words);
        println!("{}", words.len());
        let letter_counts = letter_counts(&words);
        let recommendation = words
            .iter()
            .max_by(|a, b| score(a, &letter_counts).cmp(&score(b, &letter_counts)))
            .unwrap_or_else(|| panic!("Empty wordlist"));
        println!(
            "Recommended guess: {} ({})",
            recommendation,
            (score(recommendation, &letter_counts) as f64) / (words.len() as f64)
        );
    }
}

fn prune(words: &mut HashSet<String>, ch: char) {
    words.retain(|word| !word.contains(ch));
}

fn prune_at(words: &mut HashSet<String>, ch: char, idx: usize) {
    words.retain(|word| word.chars().nth(idx).unwrap() != ch);
}

fn require(words: &mut HashSet<String>, ch: char) {
    words.retain(|word| word.contains(ch));
}

fn require_at(words: &mut HashSet<String>, ch: char, idx: usize) {
    words.retain(|word| word.chars().nth(idx).unwrap() == ch);
}

fn letter_counts(words: &HashSet<String>) -> HashMap<char, i32> {
    let mut counts = HashMap::new();
    words
        .iter()
        .map(|word| word.chars())
        .flatten()
        .for_each(|c| *counts.entry(c).or_insert(0) += 1);

    counts
}

fn score(word: &str, letter_counts: &HashMap<char, i32>) -> i32 {
    word.chars().map(|c| letter_counts.get(&c).unwrap()).sum()
}
