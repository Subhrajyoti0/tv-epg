use omega_core::Channel;
use serde::{Deserialize, Serialize};

use crate::alias::AliasEngine;
use crate::clean::{compact_name, meaningful_token_count};
use crate::similarity::combined_similarity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchScore {
    pub total: f64,

    pub name_score: f64,
    pub alias_score: f64,
    pub quality_score: f64,
    pub language_score: f64,
    pub group_score: f64,
    pub stream_score: f64,
}

pub fn score_candidate(
    source: &Channel,
    target: &Channel,
    alias_engine: &AliasEngine,
) -> MatchScore {
    let name_score = combined_similarity(&source.name, &target.name);
    let alias_score = alias_engine.alias_score(&source.name, &target.name);

    let quality_score = metadata_score(
        source.quality.as_deref(),
        target.quality.as_deref(),
        0.65,
        1.0,
        0.45,
    );

    let language_score = metadata_score(
        source.language.as_deref(),
        target.language.as_deref(),
        0.70,
        1.0,
        0.55,
    );

    let group_score = metadata_score(
        source.group.as_deref().or(source.category.as_deref()),
        target.group.as_deref().or(target.category.as_deref()),
        0.70,
        1.0,
        0.55,
    );

    let stream_score = if target.stream_url.is_some() {
        1.0
    } else {
        0.40
    };

    let mut total =
        name_score * 0.72 +
        alias_score * 0.14 +
        quality_score * 0.04 +
        language_score * 0.03 +
        group_score * 0.03 +
        stream_score * 0.04;

    let source_compact = compact_name(&source.name);
    let target_compact = compact_name(&target.name);

    let source_tokens = meaningful_token_count(&source.name);
    let target_tokens = meaningful_token_count(&target.name);

    let exact_compact_match =
        !source_compact.is_empty()
            && source_compact == target_compact;

    let trusted_alias = alias_score >= 0.95;

    let reliable_name_match =
        exact_compact_match
            || trusted_alias
            || name_score >= 0.72;

    if !reliable_name_match {
        total = total.min(0.59);
    }

    if source_tokens == 0 || target_tokens == 0 {
        total = total.min(0.40);
    }

    if source_compact.len() < 4 && !trusted_alias && !exact_compact_match {
        total = total.min(0.50);
    }

    MatchScore {
        total: total.min(1.0),
        name_score,
        alias_score,
        quality_score,
        language_score,
        group_score,
        stream_score,
    }
}

fn metadata_score(
    source: Option<&str>,
    target: Option<&str>,
    unknown_score: f64,
    match_score: f64,
    mismatch_score: f64,
) -> f64 {
    match (source, target) {
        (Some(a), Some(b)) => {
            if a.trim().eq_ignore_ascii_case(b.trim()) {
                match_score
            } else {
                mismatch_score
            }
        }
        _ => unknown_score,
    }
}
