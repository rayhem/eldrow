use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::str::FromStr;

#[derive(Clone, Debug)]
enum Command {
    Fix(char, usize),
    Guess(String, Vec<usize>, Vec<usize>),
    Prune(String),
    PruneAt(char, usize),
}

impl FromStr for Command {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (cmd, args) = s.split_once(' ').ok_or("Invalid command")?;
        match cmd {
            "guess" | "g" => {
                let to_vec =
                    |s: &str| Vec::from_iter(s.chars().map(|c| c.to_digit(10).unwrap() as usize));

                let (word, indices) = args.split_once(' ').unwrap();
                let (correct, incorrect) = dbg!(indices).trim().split_once(',').unwrap();
                Ok(Command::Guess(
                    word.to_string(),
                    to_vec(correct),
                    to_vec(incorrect),
                ))
            }
            "fix" | "f" => {
                let (ch, idx) = args.split_once(' ').unwrap();
                Ok(Command::Fix(
                    ch.chars().next().ok_or("Invalid character argument")?,
                    idx.trim().parse()?,
                ))
            }
            "pruneAt" | "pa" => {
                let (ch, idx) = args.split_once(' ').unwrap();
                Ok(Command::PruneAt(
                    ch.chars().next().ok_or("Invalid character argument")?,
                    idx.trim().parse()?,
                ))
            }
            "prune" | "p" => Ok(Command::Prune(args.trim().to_string())),
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
            Command::Prune(chars) => {
                chars.chars().for_each(|ch| prune(&mut words, ch));
            }
            Command::Fix(ch, idx) => {
                fix(&mut words, ch, idx);
            }
            Command::PruneAt(ch, idx) => {
                fix(&mut words, ch, idx);
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
                    match *tile {
                        Tile::Correct(c) => fix(&mut words, c, i),
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
        println!(
            "Recommended guess: {}",
            words
                .iter()
                .max_by(|a, b| score(a, &letter_counts).cmp(&score(b, &letter_counts)))
                .unwrap()
        );
    }
}

fn prune(words: &mut HashSet<String>, ch: char) {
    words.retain(|word| !word.contains(ch));
}

fn prune_at(words: &mut HashSet<String>, ch: char, idx: usize) {
    words.retain(|word| word.chars().nth(idx).unwrap() != ch);
}

fn fix(words: &mut HashSet<String>, ch: char, idx: usize) {
    words.retain(|word| word.chars().nth(idx).unwrap() == ch);
}

fn require(words: &mut HashSet<String>, ch: char) {
    words.retain(|word| word.contains(ch));
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
