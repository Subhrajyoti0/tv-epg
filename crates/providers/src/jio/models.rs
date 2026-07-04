use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JioChannelListResponse {
    #[serde(default)]
    pub result: Vec<JioRawChannel>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JioRawChannel {
    #[serde(default)]
    pub channel_id: serde_json::Value,

    #[serde(default)]
    pub channel_name: Option<String>,

    #[serde(default)]
    pub channel_order: serde_json::Value,

    #[serde(default, rename = "channelLanguageId")]
    pub channel_language_id: serde_json::Value,

    #[serde(default, rename = "channelCategoryId")]
    pub channel_category_id: serde_json::Value,

    #[serde(default, rename = "broadcasterId")]
    pub broadcaster_id: serde_json::Value,

    #[serde(default, rename = "isHD")]
    pub is_hd: serde_json::Value,

    #[serde(default, rename = "logoUrl")]
    pub logo_url: Option<String>,

    #[serde(default)]
    pub is_premium: serde_json::Value,

    #[serde(default, rename = "isCatchupAvailable")]
    pub is_catchup_available: serde_json::Value,

    #[serde(default, rename = "stbCatchup")]
    pub stb_catchup: serde_json::Value,

    #[serde(default)]
    pub business_type: Option<String>,

    #[serde(default)]
    pub plan_type: Option<String>,

    #[serde(default, rename = "channelPrice")]
    pub channel_price: serde_json::Value,

    #[serde(default, rename = "isHidden")]
    pub is_hidden: serde_json::Value,

    #[serde(default, rename = "isFast")]
    pub is_fast: serde_json::Value,

    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JioEpgResponse {
    #[serde(default)]
    pub epg: Option<Vec<serde_json::Value>>,

    #[serde(default)]
    pub result: Option<Vec<serde_json::Value>>,
}
