pub fn map_lang(id: Option<i64>) -> &'static str {
    match id {
        Some(1) => "Hindi",
        Some(2) => "English",
        Some(3) => "Punjabi",
        Some(4) => "Tamil",
        Some(5) => "Telugu",
        Some(6) => "English",
        Some(7) => "Marathi",
        Some(8) => "Bengali",
        Some(9) => "Gujarati",
        Some(10) => "Kannada",
        Some(11) => "Malayalam",
        Some(12) => "Odia",
        _ => "Unknown",
    }
}

pub fn map_category(id: Option<i64>) -> &'static str {
    match id {
        Some(5) => "Entertainment",
        Some(6) => "Movies",
        Some(10) => "Infotainment",
        Some(16) => "Business",
        _ => "Other",
    }
}

pub fn value_to_i64(value: &serde_json::Value) -> Option<i64> {
    if let Some(number) = value.as_i64() {
        return Some(number);
    }

    if let Some(text) = value.as_str() {
        return text.parse::<i64>().ok();
    }

    None
}

pub fn value_to_bool(value: &serde_json::Value) -> bool {
    if let Some(value) = value.as_bool() {
        return value;
    }

    if let Some(value) = value.as_i64() {
        return value != 0;
    }

    if let Some(text) = value.as_str() {
        let text = text.trim().to_lowercase();
        return matches!(text.as_str(), "1" | "true" | "yes");
    }

    false
}

pub fn value_to_string(value: &serde_json::Value) -> Option<String> {
    if let Some(text) = value.as_str() {
        if !text.trim().is_empty() {
            return Some(text.to_string());
        }
    }

    if let Some(number) = value.as_i64() {
        return Some(number.to_string());
    }

    if let Some(number) = value.as_u64() {
        return Some(number.to_string());
    }

    None
}
