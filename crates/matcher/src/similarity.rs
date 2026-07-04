use strsim::{jaro_winkler, levenshtein};

use crate::clean::{clean_name, compact_name};
use crate::jaccard::jaccard_similarity;

pub fn levenshtein_similarity(a: &str, b: &str) -> f64 {
    let a = clean_name(a);
    let b = clean_name(b);

    if a.is_empty() || b.is_empty() {
        return 0.0;
    }

    let distance = levenshtein(&a, &b);
    let max_len = a.len().max(b.len());

    if max_len == 0 {
        1.0
    } else {
        1.0 - (distance as f64 / max_len as f64)
    }
}

pub fn substring_similarity(a: &str, b: &str) -> f64 {
    let a = compact_name(a);
    let b = compact_name(b);

    if a.is_empty() || b.is_empty() {
        return 0.0;
    }

    if a == b {
        return 1.0;
    }

    if a.contains(&b) || b.contains(&a) {
        return 0.92;
    }

    0.0
}

pub fn prefix_similarity(a: &str, b: &str) -> f64 {
    let a = compact_name(a);
    let b = compact_name(b);

    if a.is_empty() || b.is_empty() {
        return 0.0;
    }

    let min_len = a.len().min(b.len());

    if min_len < 4 {
        return 0.0;
    }

    let common = a
        .chars()
        .zip(b.chars())
        .take_while(|(x, y)| x == y)
        .count();

    common as f64 / min_len as f64
}

pub fn combined_similarity(a: &str, b: &str) -> f64 {
    let clean_a = clean_name(a);
    let clean_b = clean_name(b);

    if clean_a.is_empty() || clean_b.is_empty() {
        return 0.0;
    }

    if compact_name(&clean_a) == compact_name(&clean_b) {
        return 1.0;
    }

    let jw = jaro_winkler(&clean_a, &clean_b);
    let lev = levenshtein_similarity(&clean_a, &clean_b);
    let jac = jaccard_similarity(&clean_a, &clean_b);
    let sub = substring_similarity(&clean_a, &clean_b);
    let prefix = prefix_similarity(&clean_a, &clean_b);

    let weighted =
        jw * 0.38 +
        lev * 0.18 +
        jac * 0.24 +
        sub * 0.15 +
        prefix * 0.05;

    weighted
        .max(sub)
        .max(prefix * 0.90)
        .min(1.0)
}
