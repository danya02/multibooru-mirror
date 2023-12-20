pub mod resource_locator;
pub mod state;
use serde::{Deserialize, Serialize};

use self::{resource_locator::MediaResourceLocator, state::MediaState};

#[derive(Serialize, Deserialize, Clone, Debug, Hash)]
pub struct MediaRecord {
    pub locator: MediaResourceLocator,
    pub state: MediaState,
}

impl MediaRecord {
    pub fn get_state(&self) -> MediaStateSimple {
        match self.state {
            MediaState::NotDownloadedYet => MediaStateSimple::NotDownloadedYet,
            MediaState::DownloadError(_) => MediaStateSimple::DownloadError,
            MediaState::Present { .. } => MediaStateSimple::Present,
        }
    }
}

pub enum MediaStateSimple {
    NotDownloadedYet = 0,
    DownloadError = 1,
    Present = 2,
}
