pub mod alias;
pub mod clean;
pub mod jaccard;
pub mod resolver;
pub mod reviewer;
pub mod scoring;
pub mod similarity;

pub use alias::AliasEngine;
pub use clean::{clean_name, compact_name, token_set};
pub use jaccard::jaccard_similarity;
pub use resolver::{find_best_match, resolve_channel_match};
pub use reviewer::create_review_item;
pub use scoring::{score_candidate, MatchScore};
pub use similarity::combined_similarity;
