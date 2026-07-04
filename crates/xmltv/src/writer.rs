use std::fs;
use std::path::Path;

use crate::models::{XmltvChannel, XmltvProgramme};
use crate::time::format_xmltv_time;

pub fn build_xmltv_string(
    channels: &[XmltvChannel],
    programmes: &[XmltvProgramme],
    generator_name: &str,
    offset_minutes: i32,
) -> anyhow::Result<String> {
    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str(&format!("<tv generator-info-name=\"{}\">\n", esc(generator_name)));

    for channel in channels {
        xml.push_str(&format!("  <channel id=\"{}\">\n", esc(&channel.id)));
        for name in &channel.display_names {
            xml.push_str(&format!("    <display-name>{}</display-name>\n", esc(name)));
        }
        if let Some(icon) = &channel.icon {
            if !icon.trim().is_empty() {
                xml.push_str(&format!("    <icon src=\"{}\"/>\n", esc(icon)));
            }
        }
        xml.push_str("  </channel>\n");
    }

    for programme in programmes {
        if programme.stop <= programme.start || programme.title.trim().is_empty() {
            continue;
        }
        let start = format_xmltv_time(programme.start, offset_minutes);
        let stop = format_xmltv_time(programme.stop, offset_minutes);
        xml.push_str(&format!("  <programme start=\"{}\" stop=\"{}\" channel=\"{}\">\n", esc(&start), esc(&stop), esc(&programme.channel_id)));
        xml.push_str(&format!("    <title>{}</title>\n", esc(&programme.title)));
        if let Some(subtitle) = &programme.subtitle {
            if !subtitle.trim().is_empty() { xml.push_str(&format!("    <sub-title>{}</sub-title>\n", esc(subtitle))); }
        }
        if let Some(description) = &programme.description {
            if !description.trim().is_empty() { xml.push_str(&format!("    <desc>{}</desc>\n", esc(description))); }
        }
        for category in &programme.categories {
            if !category.trim().is_empty() { xml.push_str(&format!("    <category>{}</category>\n", esc(category))); }
        }
        if let Some(icon) = &programme.icon {
            if !icon.trim().is_empty() { xml.push_str(&format!("    <icon src=\"{}\"/>\n", esc(icon))); }
        }
        if !programme.actors.is_empty() || !programme.directors.is_empty() {
            xml.push_str("    <credits>\n");
            for director in &programme.directors {
                if !director.trim().is_empty() { xml.push_str(&format!("      <director>{}</director>\n", esc(director))); }
            }
            for actor in &programme.actors {
                if !actor.trim().is_empty() { xml.push_str(&format!("      <actor>{}</actor>\n", esc(actor))); }
            }
            xml.push_str("    </credits>\n");
        }
        if let Some(value) = &programme.rating_value {
            if !value.trim().is_empty() {
                let system = programme.rating_system.as_deref().unwrap_or("unknown");
                xml.push_str(&format!("    <rating system=\"{}\"><value>{}</value></rating>\n", esc(system), esc(value)));
            }
        }
        if programme.previously_shown { xml.push_str("    <previously-shown/>\n"); }
        xml.push_str("  </programme>\n");
    }

    xml.push_str("</tv>\n");
    Ok(xml)
}

pub fn write_xmltv_file(
    path: impl AsRef<Path>,
    channels: &[XmltvChannel],
    programmes: &[XmltvProgramme],
    generator_name: &str,
    offset_minutes: i32,
) -> anyhow::Result<()> {
    let xml = build_xmltv_string(channels, programmes, generator_name, offset_minutes)?;
    if let Some(parent) = path.as_ref().parent() { fs::create_dir_all(parent)?; }
    fs::write(path, xml)?;
    Ok(())
}

fn esc(value: &str) -> String {
    value.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&apos;")
}
