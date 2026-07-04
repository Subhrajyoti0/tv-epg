
use std::collections::{HashMap, HashSet};

use omega_core::{Channel, Programme, ProviderKind};
use omega_matcher::{combined_similarity, compact_name};
use omega_xmltv::{XmltvChannel, XmltvProgramme};

#[derive(Debug)]
pub struct XmltvMappingResult {
    pub channels: Vec<XmltvChannel>,
    pub programmes: Vec<XmltvProgramme>,
    pub jio_map_count: usize,
    pub zee5_map_count: usize,
}

pub fn build_xmltv_from_iptv_tvg_ids(
    iptv_channels: &[Channel],
    jio_channels: &[Channel],
    zee5_channels: &[Channel],
    programmes: &[Programme],
    jio_direct_map: &HashMap<String, String>,
) -> XmltvMappingResult {
    let iptv_by_id = build_iptv_lookup(iptv_channels);
    let mut jio_map = jio_direct_map.clone();

    for channel in jio_channels {
        if jio_map.contains_key(&channel.id) {
            continue;
        }

        if let Some((tvg_id, score)) = best_iptv_match(channel, iptv_channels) {
            if score >= 0.78 {
                jio_map.insert(channel.id.clone(), tvg_id);
            }
        }
    }

    let mut zee5_map: HashMap<String, String> = HashMap::new();

    for channel in zee5_channels {
        if let Some((tvg_id, score)) = best_iptv_match(channel, iptv_channels) {
            if score >= 0.80 {
                zee5_map.insert(channel.id.clone(), tvg_id);
            }
        }
    }

    let mut xml_programmes = Vec::new();
    let mut used_tvg_ids: HashSet<String> = HashSet::new();
    let mut seen_programmes: HashSet<String> = HashSet::new();

    for programme in programmes {
        let tvg_id = match programme.provider {
            ProviderKind::Jio => jio_map.get(&programme.channel_id),
            ProviderKind::Zee5 => zee5_map.get(&programme.channel_id),
            _ => None,
        };

        let Some(tvg_id) = tvg_id else {
            continue;
        };

        let dedup_key = format!(
            "{}|{}|{}|{}",
            tvg_id,
            programme.start.to_rfc3339(),
            programme.stop.to_rfc3339(),
            compact_name(&programme.title)
        );

        if seen_programmes.contains(&dedup_key) {
            continue;
        }

        seen_programmes.insert(dedup_key);
        used_tvg_ids.insert(tvg_id.clone());

        let mut xml_programme = XmltvProgramme::from(programme);
        xml_programme.channel_id = tvg_id.clone();
        xml_programmes.push(xml_programme);
    }

    let mut xml_channels = Vec::new();

    for tvg_id in used_tvg_ids {
        if let Some(channel) = iptv_by_id.get(&tvg_id) {
            let name = channel
                .tvg_name
                .clone()
                .unwrap_or_else(|| channel.name.clone());

            let mut xml_channel = XmltvChannel::new(tvg_id.clone(), name);
            xml_channel.icon = channel.logo.clone();
            xml_channels.push(xml_channel);
        }
    }

    xml_channels.sort_by(|a, b| a.id.cmp(&b.id));
    xml_programmes.sort_by_key(|programme| programme.start);

    XmltvMappingResult {
        channels: xml_channels,
        programmes: xml_programmes,
        jio_map_count: jio_map.len(),
        zee5_map_count: zee5_map.len(),
    }
}

fn build_iptv_lookup(iptv_channels: &[Channel]) -> HashMap<String, Channel> {
    let mut map = HashMap::new();

    for channel in iptv_channels {
        map.insert(channel.id.clone(), channel.clone());
    }

    map
}

fn best_iptv_match(channel: &Channel, iptv_channels: &[Channel]) -> Option<(String, f64)> {
    let mut best_tvg_id: Option<String> = None;
    let mut best_score = 0.0f64;

    for iptv in iptv_channels {
        let score = score_channel_against_iptv(channel, iptv);

        if score > best_score {
            best_score = score;
            best_tvg_id = Some(iptv.id.clone());
        }
    }

    best_tvg_id.map(|id| (id, best_score))
}

fn score_channel_against_iptv(source: &Channel, iptv: &Channel) -> f64 {
    let source_name = source.name.as_str();

    let iptv_display = iptv
        .tvg_name
        .as_deref()
        .unwrap_or(iptv.name.as_str());

    let tvg_words = tvg_id_to_words(&iptv.id);

    let source_compact = compact_name(source_name);
    let iptv_display_compact = compact_name(iptv_display);
    let tvg_compact = compact_name(&tvg_words);

    let exact_display = !source_compact.is_empty() && source_compact == iptv_display_compact;
    let exact_tvg = !source_compact.is_empty() && source_compact == tvg_compact;

    if exact_display || exact_tvg {
        return 1.0;
    }

    let display_score = combined_similarity(source_name, iptv_display);
    let tvg_score = combined_similarity(source_name, &tvg_words);
    let raw_name_score = combined_similarity(source_name, &iptv.name);

    let substring_score = if !source_compact.is_empty()
        && !tvg_compact.is_empty()
        && (source_compact.contains(&tvg_compact) || tvg_compact.contains(&source_compact))
    {
        0.94
    } else {
        0.0
    };

    let quality_score = quality_match_score(source, iptv);

    let mut score =
        display_score * 0.35 +
        tvg_score * 0.40 +
        raw_name_score * 0.15 +
        substring_score * 0.05 +
        quality_score * 0.05;

    if weak_generic_match(source_name, &tvg_words, iptv_display) {
        score = score.min(0.59);
    }

    score.min(1.0)
}

fn tvg_id_to_words(tvg_id: &str) -> String {
    let mut base = tvg_id.to_string();

    if let Some((left, _)) = base.split_once('@') {
        base = left.to_string();
    }

    if let Some((left, _)) = base.split_once('.') {
        base = left.to_string();
    }

    let mut output = String::new();
    let chars = base.chars().collect::<Vec<_>>();

    for (index, ch) in chars.iter().enumerate() {
        if index > 0 {
            let prev = chars[index - 1];

            if ch.is_ascii_uppercase()
                && (prev.is_ascii_lowercase() || prev.is_ascii_digit())
            {
                output.push(' ');
            }

            if ch.is_ascii_digit() && prev.is_ascii_alphabetic() {
                output.push(' ');
            }

            if ch.is_ascii_alphabetic() && prev.is_ascii_digit() {
                output.push(' ');
            }
        }

        if ch.is_ascii_alphanumeric() {
            output.push(*ch);
        } else {
            output.push(' ');
        }
    }

    normalize_known_tvg_words(&output)
}

fn normalize_known_tvg_words(value: &str) -> String {
    value
        .replace("TV 18", "TV18")
        .replace("Tv 18", "TV18")
        .replace("HD", "")
        .replace("SD", "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn quality_match_score(source: &Channel, iptv: &Channel) -> f64 {
    let source_quality = source
        .quality
        .as_deref()
        .unwrap_or("")
        .to_lowercase();

    let iptv_quality = iptv
        .id
        .split('@')
        .nth(1)
        .unwrap_or("")
        .to_lowercase();

    if source_quality.is_empty() || iptv_quality.is_empty() {
        return 0.60;
    }

    if source_quality.contains("hd") && iptv_quality.contains("hd") {
        return 1.0;
    }

    if source_quality.contains("sd") && iptv_quality.contains("sd") {
        return 1.0;
    }

    0.50
}

fn weak_generic_match(source_name: &str, tvg_words: &str, iptv_display: &str) -> bool {
    let source = compact_name(source_name);
    let tvg = compact_name(tvg_words);
    let display = compact_name(iptv_display);

    if source.len() < 4 {
        return true;
    }

    let generic = [
        "tv",
        "channel",
        "news",
        "music",
        "movies",
        "india",
        "bharat",
    ];

    let source_words = source_name
        .split_whitespace()
        .map(|word| word.to_lowercase())
        .collect::<Vec<_>>();

    if source_words.len() == 1 && generic.contains(&source_words[0].as_str()) {
        return true;
    }

    if tvg == "indiatv" && source != "indiatv" && display != "indiatv" {
        return true;
    }

    false
}
