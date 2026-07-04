use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::builder::build_m3u;
use crate::model::{Playlist, PlaylistEntry};
use crate::parser::parse_m3u;

pub fn build_indexes(playlist_dir: impl AsRef<Path>) -> anyhow::Result<()> {
    let playlist_dir = playlist_dir.as_ref();

    let genres_dir = playlist_dir.join("genres");
    let countries_dir = playlist_dir.join("countries");

    fs::create_dir_all(&genres_dir)?;
    fs::create_dir_all(&countries_dir)?;

    let mut all_entries = Vec::new();

    for item in fs::read_dir(playlist_dir)? {
        let item = item?;
        let path = item.path();

        if !path.is_file() {
            continue;
        }

        let Some(file_name) = path.file_name().and_then(|f| f.to_str()) else {
            continue;
        };

        if !file_name.ends_with(".m3u") {
            continue;
        }

        if file_name.starts_with("index") {
            continue;
        }

        let content = fs::read_to_string(&path)?;
        let entries = parse_m3u(&content);

        all_entries.extend(entries);
    }

    let genre_buckets = bucket_by_genre(&all_entries);
    let country_buckets = bucket_by_country(&all_entries);

    write_genre_playlists(&genres_dir, &genre_buckets)?;
    write_country_playlists(&countries_dir, &country_buckets)?;

    write_genre_index(playlist_dir, &genre_buckets)?;
    write_country_index(playlist_dir, &country_buckets)?;

    Ok(())
}

fn bucket_by_genre(entries: &[PlaylistEntry]) -> BTreeMap<String, Vec<PlaylistEntry>> {
    let mut buckets: BTreeMap<String, Vec<PlaylistEntry>> = BTreeMap::new();

    for entry in entries {
        buckets
            .entry(entry.genre())
            .or_default()
            .push(entry.clone());
    }

    buckets
}

fn bucket_by_country(entries: &[PlaylistEntry]) -> BTreeMap<String, Vec<PlaylistEntry>> {
    let mut buckets: BTreeMap<String, Vec<PlaylistEntry>> = BTreeMap::new();

    for entry in entries {
        buckets
            .entry(entry.country())
            .or_default()
            .push(entry.clone());
    }

    buckets
}

fn write_genre_playlists(
    genres_dir: &Path,
    buckets: &BTreeMap<String, Vec<PlaylistEntry>>,
) -> anyhow::Result<()> {
    for (genre, entries) in buckets {
        let mut playlist = Playlist::new(genre);
        playlist.entries = entries.clone();

        let safe_file = safe_name(genre);
        let output = build_m3u(&playlist);

        fs::write(genres_dir.join(format!("{safe_file}.m3u")), output)?;
    }

    Ok(())
}

fn write_country_playlists(
    countries_dir: &Path,
    buckets: &BTreeMap<String, Vec<PlaylistEntry>>,
) -> anyhow::Result<()> {
    for (country, entries) in buckets {
        let mut playlist = Playlist::new(country);
        playlist.entries = entries.clone();

        let safe_file = safe_name(country);
        let output = build_m3u(&playlist);

        fs::write(countries_dir.join(format!("{safe_file}.m3u")), output)?;
    }

    Ok(())
}

fn write_genre_index(
    playlist_dir: &Path,
    buckets: &BTreeMap<String, Vec<PlaylistEntry>>,
) -> anyhow::Result<()> {
    let mut output = String::new();
    output.push_str("#EXTM3U\n");

    for genre in buckets.keys() {
        let safe_file = safe_name(genre);

        output.push_str(&format!("#EXTINF:-1,{}\n", genre));
        output.push_str(&format!("genres/{}.m3u\n", safe_file));
    }

    fs::write(playlist_dir.join("index.genre.m3u"), output)?;

    Ok(())
}

fn write_country_index(
    playlist_dir: &Path,
    buckets: &BTreeMap<String, Vec<PlaylistEntry>>,
) -> anyhow::Result<()> {
    let mut output = String::new();
    output.push_str("#EXTM3U\n");

    for country in buckets.keys() {
        let safe_file = safe_name(country);

        output.push_str(&format!("#EXTINF:-1,{}\n", country.to_uppercase()));
        output.push_str(&format!("countries/{}.m3u\n", safe_file));
    }

    fs::write(playlist_dir.join("index.m3u"), output)?;

    Ok(())
}

fn safe_name(value: &str) -> String {
    value
        .to_lowercase()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>()
        .split('_')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}
