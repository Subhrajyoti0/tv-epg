use omega_core::{Channel, OmegaResult, ProviderKind};
use omega_playlist::parse_m3u;

pub fn parse_iptv_org_m3u(content: &str) -> OmegaResult<Vec<Channel>> {
    let entries = parse_m3u(content);

    let channels = entries
        .into_iter()
        .map(|entry| {
            let id = entry
                .tvg_id
                .clone()
                .unwrap_or_else(|| entry.name.clone());

            let mut channel = Channel::new(
                ProviderKind::IptvOrg,
                id,
                entry.name.clone(),
            );

            let clean_url = clean_stream_url(&entry.url);

            channel.tvg_id = entry.tvg_id.clone();
            channel.tvg_name = entry.tvg_name.clone();
            channel.group = entry.group.clone();
            channel.category = entry.group.clone();
            channel.logo = entry.logo.clone();
            channel.stream_url = Some(clean_url.clone());

            channel.raw = serde_json::json!({
                "tvg_id": entry.tvg_id,
                "tvg_name": entry.tvg_name,
                "name": entry.name,
                "label": entry.label,
                "group": entry.group,
                "logo": entry.logo,
                "url": clean_url,
                "raw_extinf": entry.raw_extinf
            });

            channel
        })
        .collect();

    Ok(channels)
}

fn clean_stream_url(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .trim()
        .to_string()
}
