use std::fs;
use std::io::Write;
use std::path::Path;

use flate2::write::GzEncoder;
use flate2::Compression;

pub fn write_gzip_file(
    path: impl AsRef<Path>,
    content: &[u8],
) -> anyhow::Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }

    let file = fs::File::create(path)?;

    let mut encoder = GzEncoder::new(file, Compression::best());
    encoder.write_all(content)?;
    encoder.finish()?;

    Ok(())
}
