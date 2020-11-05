use itertools::Itertools;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use std::iter::Iterator;

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let filename = &args.get(1).expect("Missing filename argument, fool!");
    let n = args
        .get(2)
        .unwrap_or(&String::from("1"))
        .parse::<u8>()
        .expect("Bad ngram n value, fool!");
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut frequencies: HashMap<String, HashMap<char, u32>> = HashMap::new();
    for line in reader.lines() {
        let line = line?;
        let ngrams = line.chars().ngrams(n);
        ngram_frequencies(ngrams, &mut frequencies);
    }
    let serialised = serde_json::to_string(&frequencies)?;
    println!("{}", serialised);
    Ok(())
}

// ngrams
struct ZipVec<T, I: Iterator<Item = T>>(Vec<I>);

impl<T: std::clone::Clone, I: Iterator<Item = T>> Iterator for ZipVec<T, I> {
    type Item = Vec<T>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.0.is_empty() {
            None
        } else {
            self.0.iter_mut().map(Iterator::next).collect()
        }
    }
}

trait NGrams<'a, T: std::clone::Clone>: Iterator<Item = T> {
    fn tee_n(self, n: u8) -> Vec<Box<dyn Iterator<Item = T> + 'a>>;
    fn ngrams(self, n: u8) -> ZipVec<T, Box<dyn Iterator<Item = T> + 'a>>;
}

impl<'a, T: std::clone::Clone + 'a, I: Iterator<Item = T> + 'a> NGrams<'a, T> for I {
    fn tee_n(self, n: u8) -> Vec<Box<dyn Iterator<Item = T> + 'a>> {
        let mut vec: Vec<Box<dyn Iterator<Item = T> + 'a>> = Vec::new();
        let mut teed: Box<dyn Iterator<Item = T> + 'a> = Box::new(self);
        for _ in 1..n {
            let (tee_1, tee_2) = teed.tee();
            vec.push(Box::new(tee_1));
            teed = Box::new(tee_2);
        }
        vec.push(teed);
        vec
    }

    fn ngrams(self, n: u8) -> ZipVec<T, Box<dyn Iterator<Item = T> + 'a>> {
        let mut tees = self.tee_n(n);
        for (i, tee) in tees.iter_mut().skip(1).enumerate() {
            tee.nth(i);
        }
        ZipVec(tees)
    }
}

fn ngram_frequencies(
    items: impl Iterator<Item = Vec<char>>,
    frequencies: &mut HashMap<String, HashMap<char, u32>>,
) {
    for v in items {
        if let Some((last, prefix)) = v.split_last() {
            let prefix: String = prefix.into_iter().collect();
            let d = frequencies.entry(prefix).or_insert(HashMap::new());
            *d.entry(*last).or_insert(0) += 1;
        }
    }
}

// bigrams
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

trait BiGrams<T: std::clone::Clone, I: Iterator<Item = T>>: Iterator<Item = T> {
    fn bigrams(self) -> BiGramIterator<T, I>;
}

impl<T: std::clone::Clone, I: Iterator<Item = T>> BiGrams<T, I> for I {
    fn bigrams(self) -> BiGramIterator<T, I> {
        BiGramIterator::new(self)
    }
}

#[allow(dead_code)]
fn bigram_frequencies<T: std::cmp::Eq + std::hash::Hash>(
    items: impl Iterator<Item = (T, T)>,
    frequencies: &mut HashMap<T, HashMap<T, u32>>,
) {
    for (a, b) in items {
        let d = frequencies.entry(a).or_insert(HashMap::new());
        *d.entry(b).or_insert(0) += 1;
    }
}
