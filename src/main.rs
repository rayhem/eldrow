use std::collections::HashSet;
use std::io::Write;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tile {
    Correct(char),
    Incorrect(char),
    Unused(char),
    Unchecked(char),
}

fn main() {
    const WORD_LENGTH: usize = 5;
    let mut words: HashSet<_> = std::fs::read_to_string("/usr/share/dict/words")
        .expect("Could not open dictionary file")
        .lines()
        .filter(|word| word.is_ascii() && word.len() == WORD_LENGTH)
        .map(|word| word.trim().to_ascii_lowercase())
        .collect();

    let mut guesses: Vec<String> = Vec::new();

    while words.len() > 1 {
        println!("Guesses: {:?}", guesses);

        print!("Guess: ");
        std::io::stdout().flush().unwrap();
        let mut guess = String::new();
        std::io::stdin().read_line(&mut guess).unwrap();
        guess = guess.trim().to_string();

        if !words.contains(&guess) {
            println!(">> Invalid guess <<");
            continue;
        }

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

        println!("{:?}", words);
        println!("{}", words.len());
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
