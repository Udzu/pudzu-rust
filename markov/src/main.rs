use std::collections::HashMap;
use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use std::iter::Iterator;

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let filename = &args.get(1).expect("Missing filename argument, fool!");
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut frequencies: HashMap<char, HashMap<char, u32>> = HashMap::new();
    for line in reader.lines() {
        let line = line?;
        let bigrams = line.chars().bigrams();
        bigram_frequencies(bigrams, &mut frequencies);
    }
    let serialised = serde_json::to_string(&frequencies).unwrap();
    println!("{}", serialised);
    Ok(())
}

// bigram iterator
struct BiGramIterator<T: std::clone::Clone, I: Iterator<Item = T>> {
    previous: Option<T>,
    iter: I,
}

impl<T: std::clone::Clone, I: Iterator<Item = T>> BiGramIterator<T, I> {
    fn new(mut iter: I) -> BiGramIterator<T, I> {
        let previous = iter.next();
        BiGramIterator { previous, iter }
    }
}

impl<T: std::clone::Clone, I: Iterator<Item = T>> Iterator for BiGramIterator<T, I> {
    type Item = (T, T);
    fn next(&mut self) -> Option<(T, T)> {
        let previous = std::mem::replace(&mut self.previous, self.iter.next());
        match previous {
            None => None,
            Some(p) => self.previous.clone().map(|n| (p, n)),
        }
    }
}

// bigram trait to extend iterators
trait BiGrams<T: std::clone::Clone, I: Iterator<Item = T>> : Iterator<Item = T> {
    fn bigrams(self) -> BiGramIterator<T, I>;
}

impl<T: std::clone::Clone, I: Iterator<Item = T>> BiGrams<T, I> for I {
    fn bigrams(self) -> BiGramIterator<T, I> {
        BiGramIterator::new(self)
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
