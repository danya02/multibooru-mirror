use serde::{Deserialize, Serialize};

/// This represents a location of a media file on an imageboard.
/// From this, a web URL can be built.
/// However, a MediaResourceLocator excludes details that can change in a single imageboard,
/// such as what specific CDN server to download from.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MediaResourceLocator {
    /// This image is from the Danbooru imageboard.
    Danbooru {
        /// The hash string of the image, like `09bbe06d7f8c4bb3d2e992221693c21e`
        hash: String,
        /// The file extension of the original image, like `png`
        ext: String,
    },
}
