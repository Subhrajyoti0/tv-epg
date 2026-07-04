use std::collections::HashSet;

pub fn clean_name(value: &str) -> String {
    let mut lower = value.to_lowercase();

    lower = lower
        .replace("&", "&")
        .replace('&', "and")
        .replace('+', " plus ")
        .replace("@hd", " ")
        .replace("@sd", " ")
        .replace("@uhd", " ")
        .replace("@fhd", " ");

    let mut without_brackets = String::new();
    let mut inside_brackets = false;

    for ch in lower.chars() {
        match ch {
            '(' | '[' | '{' => {
                inside_brackets = true;
                without_brackets.push(' ');
            }
            ')' | ']' | '}' => {
                inside_brackets = false;
                without_brackets.push(' ');
            }
            _ if !inside_brackets => without_brackets.push(ch),
            _ => {}
        }
    }

    let mut output = String::new();

    for ch in without_brackets.chars() {
        if ch.is_ascii_alphanumeric() || ch.is_whitespace() {
            output.push(ch);
        } else {
            output.push(' ');
        }
    }

    let noise_words = [
        "hd",
        "sd",
        "uhd",
        "fhd",
        "4k",
        "1080p",
        "720p",
        "576p",
        "480p",
        "360p",
        "live",
        "channel",
        "official",
        "stream",
        "india",
        "in",
        "tv",
    ];

    output
        .split_whitespace()
        .filter(|token| !noise_words.contains(token))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn compact_name(value: &str) -> String {
    clean_name(value)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("")
}

pub fn token_set(value: &str) -> HashSet<String> {
    clean_name(value)
        .split_whitespace()
        .filter(|token| !token.is_empty())
        .map(|token| token.to_string())
        .collect()
}

pub fn meaningful_token_count(value: &str) -> usize {
    token_set(value).len()
}

pub fn normalize_quality(value: Option<&str>) -> Option<String> {
    let value = value?.to_lowercase();

    if value.contains("4k") || value.contains("uhd") {
        Some("4K".to_string())
    } else if value.contains("hd") || value.contains("1080p") || value.contains("720p") {
        Some("HD".to_string())
    } else if value.contains("sd") || value.contains("576p") || value.contains("480p") {
        Some("SD".to_string())
    } else {
        None
    }
}

pub fn infer_quality_from_name(name: &str) -> Option<String> {
    normalize_quality(Some(name))
}
