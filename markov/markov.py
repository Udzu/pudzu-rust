import argparse
import bisect
import functools
import itertools
import json
import operator as op
import pickle
import random
import string
import sys
import unicodedata
from collections import Counter
from numbers import Integral

# Simple Markov n-gram based generator.


def counter_random(counter, filter=None):
    """Return a single random elements from the Counter collection, weighted by count."""
    if filter is not None:
        counter = {k: v for k, v in counter.items() if filter(k)}
    if len(counter) == 0:
        raise Exception("No matching elements in Counter collection")
    seq = list(counter.keys())
    cum = list(itertools.accumulate(list(counter.values()), op.add))
    return seq[bisect.bisect_left(cum, random.uniform(0, cum[-1]))]


class MarkovGenerator(object):
    """Markov Chain n-gram-based generator for arbitrary iterables."""

    def __init__(self, frequency_file):
        """Initialise generator for a given frequency file."""
        with open(frequency_file) as f:
            frequencies = json.load(f)
        self.markov_dict = { k : Counter(v) for k,v in frequencies.items() }
        self.prob_dict = { k : sum(c.values()) for k,c in self.markov_dict.items() }
        
    def render(self, stop_when, start_ngram=None):
        """Return a tuple using the trained probabilities. Stop condition can be a maximum length or function."""
        stop_fn = stop_when if callable(stop_when) else lambda o: len(o) >= stop_when
        start_fn = start_ngram if (callable(start_ngram) or start_ngram is None) else lambda n: n == tuple(start_ngram)
        ngram = counter_random(self.prob_dict, filter=start_fn)
        output = ngram
        while True:
            if stop_fn(output):
                break
            elif ngram in self.markov_dict:
                v = counter_random(self.markov_dict[ngram])
                output += v
                ngram = ngram[1:] + v
            else:
                ngram = counter_random(self.prob_dict)
        return output

    def render_word(self, min_length=3, max_length=12):
        """Generates a word. Assumes training on characters including spaces.
        Doesn't filter out real words."""
        while True:
            word = self.render(lambda o: len(o) > 1 and o[-1] == " ", lambda n: n[0] == " ")
            if min_length <= len(word.strip()) <= max_length:
                return word.strip()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Generate pseudowords using Markov chains")
    parser.add_argument("frequencies", type=str, help="frequency file")
    parser.add_argument("number", type=int, help="number of words to generate")
    args = parser.parse_args()

    mk = MarkovGenerator(frequency_file=args.frequencies)
    for i in range(args.number):
        print(mk.render_word())

