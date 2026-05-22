use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CropPosition {
    Center,
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl CropPosition {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Center => "center",
            Self::Top => "top",
            Self::Bottom => "bottom",
            Self::Left => "left",
            Self::Right => "right",
            Self::TopLeft => "topleft",
            Self::TopRight => "topright",
            Self::BottomLeft => "bottomleft",
            Self::BottomRight => "bottomright",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThumbOptions {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub crop: Option<CropPosition>,
    pub quality: Option<u8>,
    pub blur: Option<u32>,
    pub grayscale: bool,
    pub sharpen: Option<i32>,
}

impl Default for ThumbOptions {
    fn default() -> Self {
        Self {
            width: None,
            height: None,
            crop: None,
            quality: Some(85),
            blur: None,
            grayscale: false,
            sharpen: None,
        }
    }
}

impl ThumbOptions {
    pub fn builder() -> ThumbOptionsBuilder {
        ThumbOptionsBuilder::default()
    }

    pub fn to_suffix(&self) -> String {
        let mut parts = Vec::new();

        if let (Some(w), Some(h)) = (self.width, self.height) {
            parts.push(format!("{w}x{h}"));
        } else if let Some(w) = self.width {
            parts.push(format!("{w}x"));
        } else if let Some(h) = self.height {
            parts.push(format!("x{h}"));
        }

        if let Some(crop) = &self.crop {
            if crop == &CropPosition::Center {
                parts.push("crop".to_string());
            } else {
                parts.push(format!("crop-{}", crop.as_str()));
            }
        }

        if let Some(q) = self.quality {
            if q != 85 {
                parts.push(format!("q{q}"));
            }
        }

        if let Some(blur) = self.blur {
            parts.push(format!("blur{blur}"));
        }

        if self.grayscale {
            parts.push("bw".to_string());
        }

        if let Some(sharpen) = self.sharpen {
            parts.push(format!("sharpen{sharpen}"));
        }

        parts.join("-")
    }

    pub fn to_filename(&self, base_name: &str, extension: &str) -> String {
        let suffix = self.to_suffix();
        if suffix.is_empty() {
            format!("{base_name}.{extension}")
        } else {
            format!("{base_name}-{suffix}.{extension}")
        }
    }
}

#[derive(Default)]
pub struct ThumbOptionsBuilder {
    width: Option<u32>,
    height: Option<u32>,
    crop: Option<CropPosition>,
    quality: Option<u8>,
    blur: Option<u32>,
    grayscale: bool,
    sharpen: Option<i32>,
}

impl ThumbOptionsBuilder {
    pub fn width(mut self, width: u32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: u32) -> Self {
        self.height = Some(height);
        self
    }

    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    pub fn crop(mut self, position: CropPosition) -> Self {
        self.crop = Some(position);
        self
    }

    pub fn quality(mut self, quality: u8) -> Self {
        self.quality = Some(quality.min(100));
        self
    }

    pub fn blur(mut self, amount: u32) -> Self {
        self.blur = Some(amount);
        self
    }

    pub fn grayscale(mut self) -> Self {
        self.grayscale = true;
        self
    }

    pub fn sharpen(mut self, amount: i32) -> Self {
        self.sharpen = Some(amount);
        self
    }

    pub fn build(self) -> ThumbOptions {
        ThumbOptions {
            width: self.width,
            height: self.height,
            crop: self.crop,
            quality: self.quality,
            blur: self.blur,
            grayscale: self.grayscale,
            sharpen: self.sharpen,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_suffix() {
        let opts = ThumbOptions::builder().width(300).height(200).build();
        assert_eq!(opts.to_suffix(), "300x200");

        let opts = ThumbOptions::builder()
            .width(300)
            .height(200)
            .crop(CropPosition::Center)
            .build();
        assert_eq!(opts.to_suffix(), "300x200-crop");

        let opts = ThumbOptions::builder()
            .width(300)
            .height(200)
            .crop(CropPosition::Top)
            .quality(90)
            .build();
        assert_eq!(opts.to_suffix(), "300x200-crop-top-q90");

        let opts = ThumbOptions::builder()
            .width(150)
            .height(150)
            .blur(5)
            .grayscale()
            .build();
        assert_eq!(opts.to_suffix(), "150x150-blur5-bw");
    }

    #[test]
    fn test_to_filename() {
        let opts = ThumbOptions::builder().width(300).height(200).build();
        assert_eq!(opts.to_filename("valley", "webp"), "valley-300x200.webp");

        let opts = ThumbOptions::default();
        assert_eq!(opts.to_filename("valley", "webp"), "valley.webp");
    }
}
