use crate::clean::token_set;

pub fn jaccard_similarity(a: &str, b: &str) -> f64 {
    let a_tokens = token_set(a);
    let b_tokens = token_set(b);

    if a_tokens.is_empty() || b_tokens.is_empty() {
        return 0.0;
    }

    let intersection = a_tokens
        .intersection(&b_tokens)
        .count();

    let union = a_tokens
        .union(&b_tokens)
        .count();

    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}
