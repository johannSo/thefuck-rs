//! Sequence comparison, a port of the parts of Python's `difflib` that the
//! rules rely on. Scores have to be the same as in `difflib`, otherwise the
//! suggested fixes and their order would change.

use std::collections::{HashMap, HashSet};

/// Compares two strings using the Ratcliff/Obershelp ("gestalt") approach.
pub struct SequenceMatcher {
    a: Vec<char>,
    b: Vec<char>,
    /// For each char of `b`, the positions it occurs at.
    b2j: HashMap<char, Vec<usize>>,
}

fn calculate_ratio(matches: usize, length: usize) -> f64 {
    if length == 0 {
        1.0
    } else {
        2.0 * matches as f64 / length as f64
    }
}

impl SequenceMatcher {
    pub fn new(a: &str, b: &str) -> Self {
        let b: Vec<char> = b.chars().collect();
        let mut b2j: HashMap<char, Vec<usize>> = HashMap::new();
        for (j, ch) in b.iter().enumerate() {
            b2j.entry(*ch).or_default().push(j);
        }
        // Chars that are too popular in long sequences are not used as
        // starting points for matches ("autojunk").
        if b.len() >= 200 {
            let ntest = b.len() / 100 + 1;
            b2j.retain(|_, indices| indices.len() <= ntest);
        }
        SequenceMatcher {
            a: a.chars().collect(),
            b,
            b2j,
        }
    }

    /// Finds the longest matching block in `a[alo..ahi]` and `b[blo..bhi]`,
    /// returns `(i, j, size)`.
    fn find_longest_match(
        &self,
        alo: usize,
        ahi: usize,
        blo: usize,
        bhi: usize,
    ) -> (usize, usize, usize) {
        let (a, b) = (&self.a, &self.b);
        let (mut besti, mut bestj, mut bestsize) = (alo, blo, 0);
        // j2len[j] is the length of the longest match ending with a[i-1] and b[j].
        let mut j2len: HashMap<usize, usize> = HashMap::new();

        for (i, ch) in a.iter().enumerate().take(ahi).skip(alo) {
            let mut newj2len = HashMap::new();
            if let Some(indices) = self.b2j.get(ch) {
                for &j in indices {
                    if j < blo {
                        continue;
                    }
                    if j >= bhi {
                        break;
                    }
                    let k = if j == 0 {
                        0
                    } else {
                        j2len.get(&(j - 1)).copied().unwrap_or(0)
                    } + 1;
                    newj2len.insert(j, k);
                    if k > bestsize {
                        besti = i + 1 - k;
                        bestj = j + 1 - k;
                        bestsize = k;
                    }
                }
            }
            j2len = newj2len;
        }

        // Extend the match with popular chars, which were ignored above.
        while besti > alo && bestj > blo && a[besti - 1] == b[bestj - 1] {
            besti -= 1;
            bestj -= 1;
            bestsize += 1;
        }
        while besti + bestsize < ahi
            && bestj + bestsize < bhi
            && a[besti + bestsize] == b[bestj + bestsize]
        {
            bestsize += 1;
        }

        (besti, bestj, bestsize)
    }

    /// Returns the total size of all matching blocks.
    fn matches(&self) -> usize {
        let mut queue = vec![(0, self.a.len(), 0, self.b.len())];
        let mut total = 0;
        while let Some((alo, ahi, blo, bhi)) = queue.pop() {
            let (i, j, k) = self.find_longest_match(alo, ahi, blo, bhi);
            if k > 0 {
                total += k;
                if alo < i && blo < j {
                    queue.push((alo, i, blo, j));
                }
                if i + k < ahi && j + k < bhi {
                    queue.push((i + k, ahi, j + k, bhi));
                }
            }
        }
        total
    }

    /// Returns a measure of the sequences' similarity in the range [0, 1].
    pub fn ratio(&self) -> f64 {
        calculate_ratio(self.matches(), self.a.len() + self.b.len())
    }

    /// Returns an upper bound on `ratio()` relatively quickly.
    pub fn quick_ratio(&self) -> f64 {
        let mut avail: HashMap<char, isize> = HashMap::new();
        for ch in &self.b {
            *avail.entry(*ch).or_insert(0) += 1;
        }
        let mut matches = 0;
        for ch in &self.a {
            let count = avail.entry(*ch).or_insert(0);
            if *count > 0 {
                matches += 1;
            }
            *count -= 1;
        }
        calculate_ratio(matches, self.a.len() + self.b.len())
    }

    /// Returns an upper bound on `ratio()` very quickly.
    pub fn real_quick_ratio(&self) -> f64 {
        let (la, lb) = (self.a.len(), self.b.len());
        calculate_ratio(la.min(lb), la + lb)
    }
}

/// Returns a list of the best "good enough" matches, the most similar first.
///
/// At most `n` matches are returned, all of them score at least `cutoff`.
pub fn get_close_matches<S: AsRef<str>>(
    word: &str,
    possibilities: &[S],
    n: usize,
    cutoff: f64,
) -> Vec<String> {
    let mut result: Vec<(f64, &str)> = Vec::new();
    for possibility in possibilities {
        let possibility = possibility.as_ref();
        let matcher = SequenceMatcher::new(possibility, word);
        if matcher.real_quick_ratio() >= cutoff && matcher.quick_ratio() >= cutoff {
            let ratio = matcher.ratio();
            if ratio >= cutoff {
                result.push((ratio, possibility));
            }
        }
    }
    // The best scorers go first, equally good ones are in reverse
    // lexicographic order.
    result.sort_by(|x, y| y.0.total_cmp(&x.0).then_with(|| y.1.cmp(x.1)));
    result.truncate(n);
    result
        .into_iter()
        .map(|(_, possibility)| possibility.to_string())
        .collect()
}

/// Returns the items without duplicates, keeping the first occurrences.
pub fn unique(items: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut seen = HashSet::new();
    items
        .into_iter()
        .filter(|item| seen.insert(item.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ratio(a: &str, b: &str) -> f64 {
        SequenceMatcher::new(a, b).ratio()
    }

    #[test]
    fn ratio_is_the_same_as_in_python() {
        assert_eq!(ratio("abcd", "bcde"), 0.75);
        assert_eq!(ratio("", ""), 1.0);
        assert_eq!(ratio("abc", ""), 0.0);
        assert!((ratio("fuck", "git push") - 0.16666666666666666).abs() < 1e-15);
        assert!(
            (ratio(
                "private Thread currentThread;",
                "private volatile Thread currentThread;"
            ) - 0.8656716417910447)
                .abs()
                < 1e-15
        );
        assert_eq!(SequenceMatcher::new("abcd", "bcde").quick_ratio(), 0.75);
        assert_eq!(SequenceMatcher::new("abcd", "bcde").real_quick_ratio(), 1.0);
    }

    #[test]
    fn popular_chars_are_junk_in_long_sequences() {
        // No char of the long sequence can start a match, so only the
        // common prefix is found: difflib gives 0.5 here as well.
        assert_eq!(ratio(&"ab".repeat(50), &"ab".repeat(150)), 0.5);
        assert_eq!(
            ratio(&"ab".repeat(50), &"ab".repeat(99)),
            100.0 * 2.0 / 298.0
        );
    }

    #[test]
    fn close_matches_are_the_same_as_in_python() {
        let words = ["ape", "apple", "peach", "puppy"];
        assert_eq!(get_close_matches("appel", &words, 3, 0.6), ["apple", "ape"]);
        assert_eq!(get_close_matches("appel", &words, 1, 0.6), ["apple"]);
        assert_eq!(
            get_close_matches("wheel", &["if", "while", "with", "else"], 3, 0.6),
            ["while"]
        );
        assert!(get_close_matches("accept", &["if", "while"], 3, 0.6).is_empty());
        // Equal scores: the lexicographically greater goes first.
        assert_eq!(
            get_close_matches("ab", &["abc", "abd", "abb"], 3, 0.6),
            ["abd", "abc", "abb"]
        );
    }
}
