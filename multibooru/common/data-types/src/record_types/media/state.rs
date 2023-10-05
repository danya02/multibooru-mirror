use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MediaState {
    /// We have not yet tried downloading this media.
    NotDownloadedYet,

    /// We have tried downloading it, but got an error.
    /// The error text is included.
    DownloadError(String),

    /// We have successfully downloaded the media, and the description is provided.
    /// If this record is issued more than once, then the latest state is to be used;
    /// it replaces the original, for example by losslessly compressing it.
    Present {
        /// This is a string reference to whichever method is storing the files.
        /// For example, it could be a file name.
        media_ref: String,
    },
}
