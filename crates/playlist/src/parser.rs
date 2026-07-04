use crate::model::PlaylistEntry;

pub fn parse_m3u(content: &str) -> Vec<PlaylistEntry> {
    let mut entries = Vec::new();
    let mut current_extinf: Option<String> = None;

    for line in content.lines() {
        let line = line.trim();

        if line.starts_with("#EXTINF") {
            current_extinf = Some(line.to_string());
            continue;
        }

        if line.starts_with("http") {
            if let Some(extinf) = current_extinf.take() {
                let tvg_id = extract_attr(&extinf, "tvg-id");
                let tvg_name = extract_attr(&extinf, "tvg-name");
                let group = extract_attr(&extinf, "group-title");
                let logo = extract_attr(&extinf, "tvg-logo");
                let label = extract_label(&extinf);

                let name = tvg_name
                    .clone()
                    .or_else(|| label.clone())
                    .or_else(|| tvg_id.clone())
                    .unwrap_or_else(|| "Unknown Channel".to_string());

                entries.push(PlaylistEntry {
                    tvg_id,
                    tvg_name,
                    name,
                    label,
                    group,
                    logo,
                    url: line.to_string(),
                    raw_extinf: extinf,
                });
            }
        }
    }

    entries
}

fn extract_attr(line: &str, attr: &str) -> Option<String> {
    let pattern = format!("{attr}=\"");
    let start = line.find(&pattern)?;

    let value_start = start + pattern.len();
    let rest = &line[value_start..];

    let end = rest.find('"')?;

    let value = rest[..end].trim();

    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn extract_label(line: &str) -> Option<String> {
    let comma_index = line.rfind(',')?;
    let label = line[comma_index + 1..].trim();

    if label.is_empty() {
        None
    } else {
        Some(label.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_m3u_entry() {
        let input = r#"#EXTM3U
#EXTINF:-1 tvg-id="ZeeNews.in" tvg-name="Zee News" tvg-logo="https://example.com/logo.png" group-title="News (IN)",Zee News
https://example.com/zee-news.m3u8
"#;

        let entries = parse_m3u(input);

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].tvg_id.as_deref(), Some("ZeeNews.in"));
        assert_eq!(entries[0].tvg_name.as_deref(), Some("Zee News"));
        assert_eq!(entries[0].group.as_deref(), Some("News (IN)"));
        assert_eq!(entries[0].genre(), "News");
        assert_eq!(entries[0].country(), "in");
    }
}
