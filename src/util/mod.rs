use std::mem;

pub use self::{cow::CowUtils, ext::*};

pub mod builder;
pub mod constants;
pub mod datetime;
pub mod hasher;
pub mod interaction;
pub mod numbers;

mod cow;
mod ext;

macro_rules! get {
    ($slice:ident[$idx:expr]) => {
        unsafe { *$slice.get_unchecked($idx) }
    };
}

macro_rules! set {
    ($slice:ident[$idx:expr] = $val:expr) => {
        unsafe { *$slice.get_unchecked_mut($idx) = $val }
    };
}

pub fn levenshtein_similarity(word_a: &str, word_b: &str) -> f32 {
    let (dist, len) = levenshtein_distance(word_a, word_b);

    (len - dist) as f32 / len as f32
}

/// "How many replace/delete/insert operations are necessary to morph one word into the other?"
///
/// Returns (distance, max word length) tuple
pub fn levenshtein_distance<'w>(mut word_a: &'w str, mut word_b: &'w str) -> (usize, usize) {
    let m = word_a.chars().count();
    let mut n = word_b.chars().count();

    if m > n {
        mem::swap(&mut word_a, &mut word_b);
        n = m;
    }

    // u16 is sufficient considering the max length
    // of discord messages is smaller than u16::MAX
    let mut costs: Vec<_> = (0..=n as u16).collect();

    // SAFETY for get! and set!:
    // chars(word_a) <= chars(word_b) = n < n + 1 = costs.len()

    for (a, i) in word_a.chars().zip(1..) {
        let mut last_val = i;

        for (b, j) in word_b.chars().zip(1..) {
            let new_val = if a == b {
                get!(costs[j - 1])
            } else {
                get!(costs[j - 1]).min(last_val).min(get!(costs[j])) + 1
            };

            set!(costs[j - 1] = last_val);
            last_val = new_val;
        }

        set!(costs[n] = last_val);
    }

    (get!(costs[n]) as usize, n)
}
