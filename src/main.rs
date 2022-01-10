use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::str::FromStr;

#[derive(Clone, Debug)]
enum Command {
    Guess(String),
    Prune(char, usize),
    PruneAll(char),
}

impl FromStr for Command {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (cmd, args) = s.split_once(' ').unwrap();
        match cmd {
            "guess" | "g" => Ok(Command::Guess(args.to_string())),
            "prune" | "p" => {
                let (ch, idx) = args.split_once(' ').unwrap();
                Ok(Command::Prune(
                    ch.chars().next().ok_or("Invalid character argument")?,
                    idx.trim().parse()?,
                ))
            }
            "pruneAll" | "pa" => Ok(Command::PruneAll(
                args.chars().next().ok_or("Invalid character argument")?,
            )),
            _ => Err("Invalid command".into()),
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
    const WORD_LENGTH: usize = 5;
    let mut words: HashSet<_> = std::fs::read_to_string("/usr/share/dict/american-english-small")
        .expect("Could not open dictionary file")
        .lines()
        .filter(|word| word.is_ascii() && word.len() == WORD_LENGTH)
        .map(|word| word.trim().to_ascii_lowercase())
        .collect();

    let mut guesses: Vec<String> = Vec::new();

    while words.len() > 1 {
        println!("Guesses: {:?}", guesses);

        print!("$ ");
        std::io::stdout().flush().unwrap();
        let mut command_string = String::new();
        std::io::stdin().read_line(&mut command_string).unwrap();
        let command = Command::from_str(&command_string);

        if let Err(_) = command {
            continue;
        }

        match command.unwrap() {
            Command::PruneAll(ch) => {
                words.retain(|word| !word.contains(ch));
            }
            Command::Prune(ch, idx) => {
                words.retain(|word| word.chars().nth(idx).unwrap() != ch);
            }
            Command::Guess(arg) => {
                let guess = arg.trim().to_string();

                guesses.push(guess.clone());
                let mut tiles: Vec<_> = guess.chars().map(Tile::Unchecked).collect();

                input_digits("  Correct placement: ")
                    .into_iter()
                    .for_each(|i| match tiles[i] {
                        Tile::Unchecked(c) => tiles[i] = Tile::Correct(c),
                        _ => unreachable!(),
                    });

                input_digits("Incorrect placement: ")
                    .into_iter()
                    .for_each(|i| match tiles[i] {
                        Tile::Unchecked(c) => tiles[i] = Tile::Incorrect(c),
                        _ => unreachable!(),
                    });

                for tile in tiles.iter_mut() {
                    if let Tile::Unchecked(c) = *tile {
                        *tile = Tile::Unused(c);
                    }
                }

                if tiles.iter().all(|t| matches!(t, Tile::Unused(_))) {
                    words.retain(|word| {
                        !tiles.iter().any(|t| -> bool {
                            match t {
                                Tile::Unused(c) => word.contains(*c),
                                _ => false,
                            }
                        })
                    })
                }

                for (i, tile) in tiles.iter().enumerate() {
                    words.retain(|word| {
                        let letter = word.chars().nth(i).unwrap();
                        match *tile {
                            Tile::Correct(c) => letter == c,
                            Tile::Incorrect(c) => tiles
                                .iter()
                                .zip(word.chars())
                                .filter_map(|(&tile, char)| -> Option<bool> {
                                    match tile {
                                        Tile::Unused(_) => Some(c == char),
                                        Tile::Incorrect(d) => Some(c != d && c == char),
                                        _ => None,
                                    }
                                })
                                .any(|b| b),
                            Tile::Unused(c) => letter != c,
                            Tile::Unchecked(_) => unreachable!(),
                        }
                    });
                }
            }
        }

        println!("{:?}", words);
        println!("{}", words.len());
        let letter_counts = letter_counts(&words);
        println!(
            "Recommended guess: {}",
            words
                .iter()
                .max_by(|a, b| score(a, &letter_counts).cmp(&score(b, &letter_counts)))
                .unwrap()
        );
    }
}

fn input_digits(msg: &str) -> Vec<usize> {
    print!("{}", msg);
    std::io::stdout().flush().unwrap();
    let mut digits = String::new();
    std::io::stdin().read_line(&mut digits).unwrap();
    digits
        .trim()
        .chars()
        .map(|ch| ch.to_digit(10).unwrap() as usize)
        .collect()
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
