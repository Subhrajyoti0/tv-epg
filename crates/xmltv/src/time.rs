use chrono::{DateTime, FixedOffset, Utc};

pub fn format_xmltv_time(value: DateTime<Utc>, offset_minutes: i32) -> String {
    let offset_seconds = offset_minutes * 60;

    let offset = FixedOffset::east_opt(offset_seconds)
        .unwrap_or_else(|| FixedOffset::east_opt(0).expect("zero offset must exist"));

    value
        .with_timezone(&offset)
        .format("%Y%m%d%H%M%S %z")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn formats_india_time() {
        let dt = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();

        let formatted = format_xmltv_time(dt, 330);

        assert_eq!(formatted, "20260101053000 +0530");
    }
}
