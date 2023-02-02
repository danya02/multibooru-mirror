use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::media_type::MediaType;

/// This struct represents a media asset: an image, video, etc.
///
/// Media assets in boorus are perennial: once uploaded and stored, they never change.
/// They are also typically very large (on the order of megabytes).
/// For this reason, they are treated separately from other types of data.
///
/// Media assets are identified by their SHA-256 hash.
/// This hash is computed from the asset's content.
/// This ensures that images are deduplicated: if two images have the same content,
/// they will have the same hash, and thus be stored only once.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MediaAsset {
    /// The SHA-256 hash of this asset.
    pub hash: [u8; 32],
    /// The size of this asset, in bytes.
    pub size: u32, // Files larger than 4GB are not to be expected.
    /// The media type of this asset.
    pub media_type: MediaType,
}

impl MediaAsset {
    /// Get the relative path where this asset should be stored.
    ///
    /// This depends on the hash of the asset, and its media type.
    /// For example, a JPEG image with a hash that starts with `0xabcdef...`
    /// will be stored at `ab/cd/abcdef...jpg`.
    pub fn path(&self) -> PathBuf {
        let mut path = PathBuf::new();
        path.push(format!("{:02x}", self.hash[0]));
        path.push(format!("{:02x}", self.hash[1]));
        path.push(hex::encode(self.hash));
        path.set_extension(self.media_type.extension());
        path
    }
}

#[cfg(test)]
mod test {
    use crate::media_type::ImageType;

    use super::*;

    #[test]
    fn test_path() {
        let asset = MediaAsset {
            hash: [
                0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45,
                0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01,
                0x23, 0x45, 0x67, 0x89,
            ],
            size: 0,
            media_type: MediaType::Image(ImageType::Jpeg),
        };
        assert_eq!(
            asset.path(),
            PathBuf::from(
                "ab/cd/abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789.jpg"
            )
        );
    }
}
