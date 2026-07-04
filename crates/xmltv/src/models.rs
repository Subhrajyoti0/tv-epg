use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use omega_core::{Channel, Programme};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XmltvChannel {
    pub id: String,
    pub display_names: Vec<String>,
    pub icon: Option<String>,
}

impl XmltvChannel {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            display_names: vec![name.into()],
            icon: None,
        }
    }
}

impl From<&Channel> for XmltvChannel {
    fn from(channel: &Channel) -> Self {
        let mut item = Self::new(channel.id.clone(), channel.name.clone());

        if let Some(tvg_name) = &channel.tvg_name {
            if tvg_name != &channel.name {
                item.display_names.push(tvg_name.clone());
            }
        }

        item.icon = channel.logo.clone();

        item
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XmltvProgramme {
    pub channel_id: String,

    pub start: DateTime<Utc>,
    pub stop: DateTime<Utc>,

    pub title: String,
    pub subtitle: Option<String>,
    pub description: Option<String>,

    pub categories: Vec<String>,

    pub icon: Option<String>,

    pub actors: Vec<String>,
    pub directors: Vec<String>,

    pub rating_system: Option<String>,
    pub rating_value: Option<String>,

    pub previously_shown: bool,
}

impl From<&Programme> for XmltvProgramme {
    fn from(programme: &Programme) -> Self {
        let mut categories = programme.categories.clone();

        for genre in &programme.genres {
            if !categories.contains(genre) {
                categories.push(genre.clone());
            }
        }

        Self {
            channel_id: programme.channel_id.clone(),
            start: programme.start,
            stop: programme.stop,
            title: programme.title.clone(),
            subtitle: programme.subtitle.clone(),
            description: programme.description.clone(),
            categories,
            icon: programme.image.clone(),
            actors: programme.actors.clone(),
            directors: programme.directors.clone(),
            rating_system: programme.rating_system.clone(),
            rating_value: programme.rating_value.clone(),
            previously_shown: programme.is_repeat,
        }
    }
}
