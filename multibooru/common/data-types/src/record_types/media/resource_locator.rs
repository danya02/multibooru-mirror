use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};

/// This represents a location of a media file on an imageboard.
/// From this, a web URL can be built.
/// However, a MediaResourceLocator excludes details that can change in a single imageboard,
/// such as what specific CDN server to download from.
#[derive(Serialize, Deserialize, Clone, Debug, Hash)]
pub enum MediaResourceLocator {
    /// This image is from the Danbooru imageboard.
    Danbooru {
        /// The hash string of the image, like `09bbe06d7f8c4bb3d2e992221693c21e`
        hash: String,
        /// The file extension of the original image, like `png`
        ext: String,
    },
}

impl MediaResourceLocator {
    pub fn true_hash_as_u64(&self) -> u64 {
        let mut hasher = Sha512::new();
        match self {
            MediaResourceLocator::Danbooru { hash, ext } => {
                hasher.update(b"danbooru");
                hasher.update(hash.as_bytes());
                hasher.update(ext.as_bytes());
            }
        }

        let result = hasher.finalize();
        let result = &result[..];
        u64::from_be_bytes(result[0..8].try_into().unwrap())
    }
}
