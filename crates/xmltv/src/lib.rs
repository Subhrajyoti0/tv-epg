pub mod gzip;
pub mod models;
pub mod time;
pub mod writer;

pub use gzip::write_gzip_file;
pub use models::{XmltvChannel, XmltvProgramme};
pub use time::format_xmltv_time;
pub use writer::{build_xmltv_string, write_xmltv_file};
