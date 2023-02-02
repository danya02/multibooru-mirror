use serde::{Deserialize, Serialize};

/// This enum represents the type of a media asset.
///
/// It is non-exhaustive: new variants may be added in the future,
/// as new media types are discovered.
/// Nevertheless, the `Unknown` variant must be corrected as soon as possible;
/// if it is set, it means that a new media type has been discovered,
/// and code must be added to handle it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaType {
    /// The asset represents an image file type.
    Image(ImageType),
    /// The asset represents an unknown media type.
    /// If this is encountered in production, it needs to be fixed ASAP.
    Unknown,
}

impl MediaType {
    /// Get the file extension corresponding to this media type.
    ///
    /// This is used to determine the file extension of a media asset.
    /// For example, a JPEG image will have the extension `jpg`.
    pub fn extension(&self) -> &'static str {
        match self {
            MediaType::Image(image_type) => image_type.extension(),
            MediaType::Unknown => "bin",
        }
    }

    /// Derive the media type from a file extension.
    pub fn from_extension(extension: &str) -> Self {
        let extension = extension.to_lowercase();
        // Try to parse this extension as each of the known media types.
        let mt = ImageType::try_from_extension(&extension).map(MediaType::Image);
        if let Some(mt) = mt {
            return mt;
        }

        MediaType::Unknown
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageType {
    /// The asset is a JPEG image.
    Jpeg,
    /// The asset is a PNG image.
    Png,
    /// The asset is a GIF image.
    Gif,
    /// The asset is a WebP image.
    Webp,
}

impl ImageType {
    /// Get the file extension corresponding to this image type.
    ///
    /// This is used to determine the file extension of a media asset.
    /// For example, a JPEG image will have the extension `jpg`.
    pub fn extension(&self) -> &'static str {
        match self {
            ImageType::Jpeg => "jpg",
            ImageType::Png => "png",
            ImageType::Gif => "gif",
            ImageType::Webp => "webp",
        }
    }

    /// Derive the image type from a file extension.
    pub fn try_from_extension(extension: &str) -> Option<Self> {
        match extension {
            "jpg" => Some(ImageType::Jpeg),
            "jpeg" => Some(ImageType::Jpeg),
            "png" => Some(ImageType::Png),
            "gif" => Some(ImageType::Gif),
            "webp" => Some(ImageType::Webp),
            _ => None,
        }
    }
}
