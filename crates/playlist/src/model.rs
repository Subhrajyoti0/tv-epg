use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub name: String,
    pub entries: Vec<PlaylistEntry>,
}

impl Playlist {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            entries: Vec::new(),
        }
    }

    pub fn push(&mut self, entry: PlaylistEntry) {
        self.entries.push(entry);
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistEntry {
    pub tvg_id: Option<String>,
    pub tvg_name: Option<String>,
    pub name: String,
    pub label: Option<String>,
    pub group: Option<String>,
    pub logo: Option<String>,
    pub url: String,
    pub raw_extinf: String,
}

impl PlaylistEntry {
    pub fn genre(&self) -> String {
        let Some(group) = &self.group else {
            return "General".to_string();
        };

        let first = group
            .split('|')
            .next()
            .unwrap_or("General")
            .trim();

        let without_country = match first.find('(') {
            Some(index) => first[..index].trim(),
            None => first,
        };

        if without_country.is_empty() {
            "General".to_string()
        } else {
            without_country.to_string()
        }
    }

    pub fn country(&self) -> String {
        let Some(group) = &self.group else {
            return "unknown".to_string();
        };

        if let Some(start) = group.find('(') {
            if let Some(end) = group[start + 1..].find(')') {
                let country = &group[start + 1..start + 1 + end];

                if !country.trim().is_empty() {
                    return country.trim().to_lowercase();
                }
            }
        }

        "unknown".to_string()
    }
}
