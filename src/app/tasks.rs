use super::state::{target_dimensions, AspectAction, CurrentGameImages, RawImageData};
use crate::lutris::database::{Game, LutrisDatabase};
use crate::lutris::paths::LutrisPaths;
use crate::sources::traits::ImageKind;
use ::image::{
    self, imageops::FilterType, DynamicImage, GenericImageView, ImageFormat, Rgba, RgbaImage,
};
use std::io::{Cursor, ErrorKind};
use std::path::{Path, PathBuf};

pub(crate) async fn load_games(path: PathBuf) -> Result<Vec<Game>, String> {
    tokio::task::spawn_blocking(move || {
        let db = LutrisDatabase::new(path);
        db.get_games().map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

pub(crate) async fn check_current_images(slug: String, paths: LutrisPaths) -> CurrentGameImages {
    let cover = find_existing_image(paths.covers_dir(), &slug, None).await;
    let banner = find_existing_image(paths.banners_dir(), &slug, None).await;
    let icon = find_existing_image(paths.icons_dir(), &slug, Some(LutrisPaths::ICON_PREFIX)).await;

    CurrentGameImages {
        cover,
        banner,
        icon,
    }
}

pub(crate) async fn download_full_image(url: String) -> Result<RawImageData, String> {
    let bytes = reqwest::get(&url)
        .await
        .map_err(|e| e.to_string())?
        .bytes()
        .await
        .map_err(|e| e.to_string())?
        .to_vec();

    let format = image::guess_format(&bytes).map_err(|_| "Unsupported image format".to_string())?;
    if !matches!(format, ImageFormat::Png | ImageFormat::Jpeg) {
        return Err("Only PNG and JPG images are supported".to_string());
    }

    let img = image::load_from_memory(&bytes).map_err(|e| e.to_string())?;
    let (width, height) = img.dimensions();

    Ok(RawImageData {
        bytes,
        width,
        height,
        format,
    })
}

pub(crate) async fn process_and_save_image(
    data: RawImageData,
    kind: ImageKind,
    slug: String,
    action: AspectAction,
    paths: LutrisPaths,
) -> Result<(), String> {
    let target = target_dimensions(&kind);
    let format = data.format;
    let processed_bytes =
        tokio::task::spawn_blocking(move || transform_image_bytes(data, target, action))
            .await
            .map_err(|e| e.to_string())??;

    save_image_bytes(processed_bytes, format, kind, slug, paths).await
}

pub(crate) async fn generate_preview_image(
    data: RawImageData,
    kind: ImageKind,
    action: AspectAction,
) -> Result<Vec<u8>, String> {
    let target = target_dimensions(&kind);
    tokio::task::spawn_blocking(move || transform_image_bytes(data, target, action))
        .await
        .map_err(|e| e.to_string())?
}

async fn find_existing_image(dir: PathBuf, slug: &str, prefix: Option<&str>) -> Option<PathBuf> {
    let base = match prefix {
        Some(prefix) => format!("{}{}", prefix, slug),
        None => slug.to_string(),
    };
    for ext in ["jpg", "png"] {
        let candidate = dir.join(format!("{}.{}", base, ext));
        if tokio::fs::try_exists(&candidate).await.unwrap_or(false) {
            return Some(candidate);
        }
    }
    None
}

async fn cleanup_alternate_variants(dir: &Path, stem: &str, keep_ext: &str) -> Result<(), String> {
    for ext in ["jpg", "png"] {
        if ext == keep_ext {
            continue;
        }
        let candidate = dir.join(format!("{}.{}", stem, ext));
        if tokio::fs::try_exists(&candidate).await.unwrap_or(false) {
            match tokio::fs::remove_file(&candidate).await {
                Ok(_) => {}
                Err(err) if err.kind() == ErrorKind::NotFound => {}
                Err(err) => return Err(err.to_string()),
            }
        }
    }
    Ok(())
}

fn transform_image_bytes(
    data: RawImageData,
    target: (u32, u32),
    action: AspectAction,
) -> Result<Vec<u8>, String> {
    if matches!(action, AspectAction::Original) {
        return Ok(data.bytes);
    }

    let image = image::load_from_memory(&data.bytes).map_err(|e| e.to_string())?;
    let transformed = match action {
        AspectAction::Stretch => resize_stretch(&image, target.0, target.1),
        AspectAction::Cover => resize_cover(&image, target.0, target.1),
        AspectAction::Contain => resize_contain(&image, target.0, target.1, is_png(data.format)),
        AspectAction::Original => unreachable!(),
    };

    encode_image(&transformed, data.format)
}

fn resize_stretch(image: &DynamicImage, width: u32, height: u32) -> DynamicImage {
    image.resize_exact(width, height, FilterType::Lanczos3)
}

fn resize_cover(image: &DynamicImage, width: u32, height: u32) -> DynamicImage {
    let (src_w, src_h) = image.dimensions();
    let scale = f32::max(width as f32 / src_w as f32, height as f32 / src_h as f32);
    let resized = image.resize(
        (src_w as f32 * scale).round() as u32,
        (src_h as f32 * scale).round() as u32,
        FilterType::Lanczos3,
    );

    let x = (resized.width() - width) / 2;
    let y = (resized.height() - height) / 2;
    resized.crop_imm(x, y, width, height)
}

fn resize_contain(
    image: &DynamicImage,
    width: u32,
    height: u32,
    transparent_bg: bool,
) -> DynamicImage {
    let (src_w, src_h) = image.dimensions();
    let scale = f32::min(width as f32 / src_w as f32, height as f32 / src_h as f32);
    let target_w = (src_w as f32 * scale).round().max(1.0) as u32;
    let target_h = (src_h as f32 * scale).round().max(1.0) as u32;
    let resized = image.resize(target_w, target_h, FilterType::Lanczos3);

    let mut canvas = RgbaImage::from_pixel(
        width,
        height,
        if transparent_bg {
            Rgba([0, 0, 0, 0])
        } else {
            Rgba([0, 0, 0, 255])
        },
    );

    let offset_x = ((width - target_w) / 2) as i64;
    let offset_y = ((height - target_h) / 2) as i64;
    image::imageops::overlay(&mut canvas, &resized.to_rgba8(), offset_x, offset_y);
    DynamicImage::ImageRgba8(canvas)
}

fn encode_image(image: &DynamicImage, format: ImageFormat) -> Result<Vec<u8>, String> {
    let mut buffer = Vec::new();
    {
        let mut cursor = Cursor::new(&mut buffer);
        image
            .write_to(&mut cursor, format)
            .map_err(|e| e.to_string())?;
    }
    Ok(buffer)
}

async fn save_image_bytes(
    bytes: Vec<u8>,
    format: ImageFormat,
    kind: ImageKind,
    slug: String,
    paths: LutrisPaths,
) -> Result<(), String> {
    let extension = match format {
        ImageFormat::Png => "png",
        ImageFormat::Jpeg => "jpg",
        _ => return Err("Unsupported image format".to_string()),
    };

    let (dir, stem) = match kind {
        ImageKind::Cover => (paths.covers_dir(), slug.to_string()),
        ImageKind::Banner | ImageKind::Hero => (paths.banners_dir(), slug.to_string()),
        ImageKind::Icon | ImageKind::Logo => (
            paths.icons_dir(),
            format!("{}{}", LutrisPaths::ICON_PREFIX, slug),
        ),
    };

    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| e.to_string())?;

    cleanup_alternate_variants(&dir, &stem, extension).await?;
    let path = dir.join(format!("{}.{}", stem, extension));
    tokio::fs::write(path, bytes)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn is_png(format: ImageFormat) -> bool {
    matches!(format, ImageFormat::Png)
}
