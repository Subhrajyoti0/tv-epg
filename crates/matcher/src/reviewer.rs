use omega_core::{Channel, ReviewItem};

use crate::alias::AliasEngine;
use crate::resolver::find_best_match;
use crate::scoring::score_candidate;

pub fn create_review_item(
    source: Channel,
    candidates: &[Channel],
    alias_engine: &AliasEngine,
    reason: impl Into<String>,
) -> ReviewItem {
    let mut review = ReviewItem::new(source.clone(), reason);

    let mut scored_candidates = candidates
        .iter()
        .map(|candidate| {
            let score = score_candidate(&source, candidate, alias_engine);
            (candidate.clone(), score.total)
        })
        .collect::<Vec<_>>();

    scored_candidates.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    review.candidates = scored_candidates
        .iter()
        .take(5)
        .map(|(channel, _)| channel.clone())
        .collect();

    if let Some((_, best_score)) = find_best_match(&source, candidates, alias_engine) {
        review.best_score = best_score.total;
    }

    review
}
