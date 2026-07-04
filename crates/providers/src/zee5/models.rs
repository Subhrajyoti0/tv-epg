use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zee5CatalogResponse {
    #[serde(default)]
    pub total: Option<usize>,

    #[serde(default)]
    pub items: Vec<Zee5CatalogItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zee5CatalogItem {
    pub id: String,

    #[serde(default)]
    pub title: Option<String>,

    #[serde(default)]
    pub original_title: Option<String>,

    #[serde(default)]
    pub image: Zee5Image,

    #[serde(default)]
    pub languages: Vec<String>,

    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Zee5Image {
    #[serde(default)]
    pub channel_square: Option<String>,

    #[serde(default)]
    pub channel_list: Option<String>,

    #[serde(default)]
    pub square: Option<String>,

    #[serde(default)]
    pub list: Option<String>,

    #[serde(default)]
    pub cover: Option<String>,

    #[serde(default)]
    pub thumbnail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zee5EpgResponse {
    #[serde(default)]
    pub items: Vec<Zee5EpgChannelEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zee5EpgChannelEntry {
    #[serde(default)]
    pub id: Option<String>,

    #[serde(default)]
    pub title: Option<String>,

    #[serde(default)]
    pub items: Vec<serde_json::Value>,
}
