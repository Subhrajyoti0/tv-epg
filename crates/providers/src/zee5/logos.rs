use crate::zee5::models::Zee5Image;

const ZEE5_IMAGE_BASE: &str = "https://akamaividz2.zee5.com/image/upload";

pub fn resolve_logo_from_image(image: &Zee5Image) -> Option<String> {
    let key = image
        .channel_square
        .as_ref()
        .or(image.channel_list.as_ref())
        .or(image.square.as_ref())
        .or(image.list.as_ref())
        .or(image.cover.as_ref())
        .or(image.thumbnail.as_ref())?;

    Some(format!("{}/{}.png", ZEE5_IMAGE_BASE, key))
}
