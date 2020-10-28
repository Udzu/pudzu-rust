use std::collections::HashMap;
use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use std::iter::Iterator;

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let filename = &args.get(1).expect("Missing filename argument");
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut frequencies: HashMap<char, HashMap<char, u32>> = HashMap::new();
    for line in reader.lines() {
        let line = line?;
        let bigram_it = bigrams(line.chars());
        bigram_frequencies(bigram_it, &mut frequencies);
    }
    let serialised = serde_json::to_string(&frequencies).unwrap();
    println!("{}", serialised);
    Ok(())
}

// bigram iterator (Q: should this return refs instead?)
fn bigrams<T: std::clone::Clone>(items: impl Iterator<Item = T>) -> impl Iterator<Item = (T, T)> {
    BiGrams::new(items)
}

struct BiGrams<T: std::clone::Clone, I: Iterator<Item = T>> {
    previous: Option<T>,
    iter: I,
}

impl<T: std::clone::Clone, I: Iterator<Item = T>> BiGrams<T, I> {
    fn new(mut iter: I) -> BiGrams<T, I> {
        let previous = iter.next();
        BiGrams { previous, iter }
    }
}

impl<T: std::clone::Clone, I: Iterator<Item = T>> Iterator for BiGrams<T, I> {
    type Item = (T, T);
    fn next(&mut self) -> Option<(T, T)> {
        let previous = std::mem::replace(&mut self.previous, self.iter.next());
        match previous {
            None => None,
            Some(p) => self.previous.clone().map(|n: T| (p, n)),
        }
    }
}

// bigram frequency counter
fn bigram_frequencies<T: std::cmp::Eq + std::hash::Hash>(
    items: impl Iterator<Item = (T, T)>,
    frequencies: &mut HashMap<T, HashMap<T, u32>>,
) {
    for (a, b) in items {
        let d = frequencies.entry(a).or_insert(HashMap::new());
        *d.entry(b).or_insert(0) += 1;
    }
}
