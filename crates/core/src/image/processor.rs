use super::{CropPosition, Result, ThumbOptions};
use image::{imageops::FilterType, DynamicImage, GenericImageView, ImageFormat};
use std::path::Path;

pub struct ImageProcessor;

impl ImageProcessor {
    pub fn generate_thumbnail<P: AsRef<Path>>(
        source: P,
        dest: P,
        options: ThumbOptions,
    ) -> Result<()> {
        let img = image::open(source.as_ref())?;

        let processed = Self::process_image(img, options)?;

        if let Some(parent) = dest.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }

        processed.save(dest.as_ref())?;

        Ok(())
    }

    pub fn get_dimensions<P: AsRef<Path>>(path: P) -> Result<(u32, u32)> {
        let reader = image::ImageReader::open(path)?;
        let dimensions = reader.into_dimensions()?;
        Ok(dimensions)
    }

    pub fn process_image(mut img: DynamicImage, options: ThumbOptions) -> Result<DynamicImage> {
        if options.width.is_some() || options.height.is_some() {
            img = if options.crop.is_some() {
                Self::crop_image(img, options.width, options.height, options.crop.unwrap())?
            } else {
                Self::resize_image(img, options.width, options.height)?
            };
        }

        if let Some(blur_amount) = options.blur {
            img = img.blur(blur_amount as f32);
        }

        if options.grayscale {
            img = DynamicImage::ImageLuma8(img.to_luma8());
        }

        if let Some(sharpen_amount) = options.sharpen {
            img = img.unsharpen(2.0, sharpen_amount);
        }

        Ok(img)
    }

    fn resize_image(
        img: DynamicImage,
        width: Option<u32>,
        height: Option<u32>,
    ) -> Result<DynamicImage> {
        let (orig_width, orig_height) = img.dimensions();

        let (target_width, target_height) = match (width, height) {
            (Some(w), Some(h)) => (w, h),
            (Some(w), None) => {
                let h = (orig_height as f64 * w as f64 / orig_width as f64) as u32;
                (w, h)
            }
            (None, Some(h)) => {
                let w = (orig_width as f64 * h as f64 / orig_height as f64) as u32;
                (w, h)
            }
            (None, None) => return Ok(img),
        };

        Ok(img.resize(target_width, target_height, FilterType::Lanczos3))
    }

    fn crop_image(
        img: DynamicImage,
        width: Option<u32>,
        height: Option<u32>,
        position: CropPosition,
    ) -> Result<DynamicImage> {
        let (orig_width, orig_height) = img.dimensions();

        let (target_width, target_height) = match (width, height) {
            (Some(w), Some(h)) => (w, h),
            (Some(w), None) => (w, w),
            (None, Some(h)) => (h, h),
            (None, None) => return Ok(img),
        };

        let aspect_ratio = orig_width as f64 / orig_height as f64;
        let target_ratio = target_width as f64 / target_height as f64;

        let (resize_width, resize_height) = if aspect_ratio > target_ratio {
            let w = (target_height as f64 * aspect_ratio) as u32;
            (w, target_height)
        } else {
            let h = (target_width as f64 / aspect_ratio) as u32;
            (target_width, h)
        };

        let resized = img.resize_to_fill(resize_width, resize_height, FilterType::Lanczos3);

        let (x, y) = Self::calculate_crop_position(
            resize_width,
            resize_height,
            target_width,
            target_height,
            position,
        );

        Ok(resized.crop_imm(x, y, target_width, target_height))
    }

    fn calculate_crop_position(
        img_width: u32,
        img_height: u32,
        crop_width: u32,
        crop_height: u32,
        position: CropPosition,
    ) -> (u32, u32) {
        let max_x = img_width.saturating_sub(crop_width);
        let max_y = img_height.saturating_sub(crop_height);

        match position {
            CropPosition::Center => (max_x / 2, max_y / 2),
            CropPosition::Top => (max_x / 2, 0),
            CropPosition::Bottom => (max_x / 2, max_y),
            CropPosition::Left => (0, max_y / 2),
            CropPosition::Right => (max_x, max_y / 2),
            CropPosition::TopLeft => (0, 0),
            CropPosition::TopRight => (max_x, 0),
            CropPosition::BottomLeft => (0, max_y),
            CropPosition::BottomRight => (max_x, max_y),
        }
    }

    pub fn is_image<P: AsRef<Path>>(path: P) -> bool {
        if let Some(ext) = path.as_ref().extension() {
            matches!(
                ext.to_str().unwrap_or("").to_lowercase().as_str(),
                "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "ico" | "tiff" | "tif"
            )
        } else {
            false
        }
    }

    pub fn detect_format<P: AsRef<Path>>(path: P) -> Option<ImageFormat> {
        ImageFormat::from_path(path).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_image() {
        assert!(ImageProcessor::is_image("photo.jpg"));
        assert!(ImageProcessor::is_image("photo.JPEG"));
        assert!(ImageProcessor::is_image("photo.png"));
        assert!(ImageProcessor::is_image("photo.webp"));
        assert!(!ImageProcessor::is_image("document.pdf"));
        assert!(!ImageProcessor::is_image("video.mp4"));
    }

    #[test]
    fn test_calculate_crop_position() {
        let (x, y) =
            ImageProcessor::calculate_crop_position(1000, 800, 300, 200, CropPosition::Center);
        assert_eq!((x, y), (350, 300));

        let (x, y) =
            ImageProcessor::calculate_crop_position(1000, 800, 300, 200, CropPosition::TopLeft);
        assert_eq!((x, y), (0, 0));

        let (x, y) =
            ImageProcessor::calculate_crop_position(1000, 800, 300, 200, CropPosition::BottomRight);
        assert_eq!((x, y), (700, 600));
    }
}
