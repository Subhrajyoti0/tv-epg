use crate::model::{Playlist, PlaylistEntry};

pub fn build_m3u(playlist: &Playlist) -> String {
    let mut output = String::new();

    output.push_str("#EXTM3U\n");

    for entry in &playlist.entries {
        output.push_str(&build_entry(entry));
    }

    output
}

fn build_entry(entry: &PlaylistEntry) -> String {
    let tvg_id = entry.tvg_id.as_deref().unwrap_or("");
    let tvg_name = entry.tvg_name.as_deref().unwrap_or(&entry.name);
    let logo = entry.logo.as_deref().unwrap_or("");
    let group = entry.group.as_deref().unwrap_or("General");

    format!(
        "#EXTINF:-1 tvg-id=\"{}\" tvg-name=\"{}\" tvg-logo=\"{}\" group-title=\"{}\",{}\n{}\n",
        escape_attr(tvg_id),
        escape_attr(tvg_name),
        escape_attr(logo),
        escape_attr(group),
        entry.name,
        entry.url
    )
}

fn escape_attr(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
