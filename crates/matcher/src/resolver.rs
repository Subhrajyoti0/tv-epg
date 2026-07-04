use omega_core::{
    Channel, Confidence, MatchDecision, MatchResult, ProviderReference, UnifiedChannel,
};

use crate::alias::AliasEngine;
use crate::scoring::{score_candidate, MatchScore};

pub fn find_best_match(
    source: &Channel,
    targets: &[Channel],
    alias_engine: &AliasEngine,
) -> Option<(Channel, MatchScore)> {
    let mut best: Option<(Channel, MatchScore)> = None;

    for target in targets {
        let score = score_candidate(source, target, alias_engine);

        match &best {
            Some((_, best_score)) if best_score.total >= score.total => {}
            _ => {
                best = Some((target.clone(), score));
            }
        }
    }

    best
}

pub fn resolve_channel_match(
    source: Channel,
    targets: &[Channel],
    alias_engine: &AliasEngine,
    high_threshold: f64,
    review_threshold: f64,
) -> MatchResult {
    let best = find_best_match(&source, targets, alias_engine);

    let Some((target, score)) = best else {
        return MatchResult::needs_review(source, "no candidate found");
    };

    if score.total >= high_threshold {
        let mut unified = UnifiedChannel::new(&source.name);

        unified.display_name = source.name.clone();
        unified.canonical_name = source.name.clone();

        unified.tvg_id = target.tvg_id.clone().or(source.tvg_id.clone());
        unified.language = source.language.clone().or(target.language.clone());
        unified.country = source.country.clone().or(target.country.clone());
        unified.group = source.group.clone().or(target.group.clone());
        unified.category = source.category.clone().or(target.category.clone());
        unified.quality = source.quality.clone().or(target.quality.clone());
        unified.logo = source.logo.clone().or(target.logo.clone());
        unified.stream_url = target.stream_url.clone().or(source.stream_url.clone());

        unified.confidence = score.total;
        unified.confidence_source = format!(
            "{}+{}",
            source.provider.as_str(),
            target.provider.as_str()
        );

        unified.add_provider_reference(ProviderReference {
            provider: source.provider,
            id: source.id.clone(),
            name: source.name.clone(),
            confidence: 1.0,
        });

        unified.add_provider_reference(ProviderReference {
            provider: target.provider,
            id: target.id.clone(),
            name: target.name.clone(),
            confidence: score.total,
        });

        MatchResult {
            decision: MatchDecision::Matched,
            confidence: Confidence::from_score(score.total),
            score: score.total,
            source,
            target: Some(target),
            unified: Some(unified),
            reason: None,
        }
    } else if score.total >= review_threshold {
        MatchResult {
            decision: MatchDecision::NeedsReview,
            confidence: Confidence::Review,
            score: score.total,
            source,
            target: Some(target),
            unified: None,
            reason: Some("candidate found but below high-confidence threshold".to_string()),
        }
    } else {
        MatchResult {
            decision: MatchDecision::NeedsReview,
            confidence: Confidence::Review,
            score: score.total,
            source,
            target: Some(target),
            unified: None,
            reason: Some("best candidate below review threshold".to_string()),
        }
    }
}
