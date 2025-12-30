use image::{DynamicImage, ImageBuffer, Rgba};
use std::path::Path;

pub fn resize_image(image: &DynamicImage, width: u32, height: u32) -> DynamicImage {
    image.resize(width, height, image::imageops::FilterType::Lanczos3)
}

pub fn save_image(image: &DynamicImage, path: &Path) -> Result<(), image::ImageError> {
    image.save(path)
}

pub fn create_thumbnail(image: &DynamicImage, thumbnail_size: u32) -> DynamicImage {
    resize_image(image, thumbnail_size, thumbnail_size)
}

pub fn convert_to_rgba(image: &DynamicImage) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    image.to_rgba8()
}
