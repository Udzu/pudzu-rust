use itertools::{Itertools, Tee};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use std::iter::{Iterator, Zip};

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let filename = &args.get(1).expect("Missing filename argument, fool!");
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut frequencies: HashMap<char, HashMap<char, u32>> = HashMap::new();
    for line in reader.lines() {
        let line = line?;
        let bigrams = line.chars().bigrams_tee();
        bigram_frequencies(bigrams, &mut frequencies);
        // TODO: switch to ngrams
        // let ngrams : Vec<Vec<char>> = line.chars().ngrams(1).collect();
        // println!("{:#?}", ngrams);
    }
    let serialised = serde_json::to_string(&frequencies)?;
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

trait BiGrams<T: std::clone::Clone, I: Iterator<Item = T>>: Iterator<Item = T> {
    fn bigrams(self) -> BiGramIterator<T, I>;
}

impl<T: std::clone::Clone, I: Iterator<Item = T>> BiGrams<T, I> for I {
    fn bigrams(self) -> BiGramIterator<T, I> {
        BiGramIterator::new(self)
    }
}

// bigram iterator (teeing approach)
trait BiGramsZip<T: std::clone::Clone, I: Iterator<Item = T>>: Iterator<Item = T> {
    fn bigrams_tee(self) -> Zip<Tee<I>, Tee<I>>;
}

impl<T: std::clone::Clone, I: Iterator<Item = T>> BiGramsZip<T, I> for I {
    fn bigrams_tee(self) -> Zip<Tee<I>, Tee<I>> {
        let (a, mut b) = self.tee();
        b.next();
        a.zip(b)
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

// ngrams
struct ZipVec<T, I: Iterator<Item = T>>(Vec<I>);

impl<T: std::clone::Clone, I: Iterator<Item = T>> Iterator for ZipVec<T, I>
{
    type Item = Vec<T>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.iter_mut().map(Iterator::next).collect()
    }
}

trait NGrams<T: std::clone::Clone, I: Iterator<Item = T>>: Iterator<Item = T> {
    fn tee_n(self, n: u8) -> Vec<Tee<I>>;
    fn ngrams(self, n: u8) -> ZipVec<T, Tee<I>>;
}

impl<T: std::clone::Clone, I: Iterator<Item = T>> NGrams<T, I> for I {
    fn tee_n(self, _n: u8) -> Vec<Tee<I>> {
        let vec: Vec<Tee<I>> = Vec::new();
        // TODO: populate using tee
        vec
    }

    fn ngrams(self, n: u8) -> ZipVec<T, Tee<I>> {
        let mut tees = self.tee_n(n);
        for (i, tee) in (&mut tees).iter_mut().skip(0).enumerate() {
            tee.nth(i);
        }
        ZipVec(tees)
    }
}
