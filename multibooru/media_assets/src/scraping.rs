use std::{env, path::PathBuf};

use crate::{database::Database, media_asset::MediaAsset, media_type::MediaType};
use futures_util::StreamExt;
use sha2::Digest;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::Receiver;
use tokio::sync::oneshot;

pub type MediaDownloadResult = Result<MediaAsset, MediaDownloadError>;
pub type MediaDownloadSender =
    tokio::sync::mpsc::Sender<(String, tokio::sync::oneshot::Sender<MediaDownloadResult>)>;

pub async fn enqueue_download(
    url: &str,
    sender: &MediaDownloadSender,
) -> oneshot::Receiver<MediaDownloadResult> {
    let (tx, rx) = oneshot::channel();
    tracing::debug!("Enqueuing download of media asset: {}", url);
    sender
        .send((url.to_string(), tx))
        .await
        .expect("Failed to send media asset download request to downloader.");
    rx
}

pub async fn download_files(mut recv: Receiver<(String, oneshot::Sender<MediaDownloadResult>)>) {
    // Ensure that the directory $MEDIA_ROOT exists.
    tokio::fs::create_dir_all(&env::var("MEDIA_ROOT").expect("MEDIA_ROOT must be set"))
        .await
        .expect("Failed to create media root directory.");
    // Future calls to env::var("MEDIA_ROOT").unwrap() will not panic because we just checked that it exists.

    let conn = Database::new().await;
    let client = reqwest::Client::builder()
        .user_agent(common::USER_AGENT);
    let client = common::with_media_proxy(client);
    let client = client.build().unwrap();

    
    // Loop and wait for messages
    while let Some((url, sender)) = recv.recv().await {
        tracing::debug!("Received download request for media asset: {}", url);
        // Before downloading, check if the file already exists in the database
        if let Some(media_asset) = conn.get_media_asset_by_url(&url).await {
            tracing::debug!("Media asset already exists in database: {} as {:?}", url, media_asset);
            // If it does, send the result and continue
            sender.send(Ok(media_asset)).unwrap_or_else(|_| {
                tracing::warn!("Failed to send already existing media asset to receiver.");
            });
            continue;
        };
        // If it doesn't, first download the file.
        // As we do, we calculate its hash and size.
        tracing::debug!("Media asset does not exist in database, downloading: {}", url);
        let media_asset = match download_file(&client, &url).await {
            Ok(media_asset) => media_asset,
            Err(err) => {
                tracing::error!("Failed to download file: {err:?}");
                sender.send(Err(err)).unwrap_or_else(|_| {
                    tracing::warn!("Failed to send media asset download error to receiver.");
                });
                continue;
            }
        };

        // Then, add it to the database.
        tracing::debug!("Media asset downloaded, adding to database: {}", url);
        let result = try_adding_media_asset_to_db(&url, media_asset, &conn).await;
        if let Err(err) = result {
            tracing::error!("Failed to add media asset to db: {err:?}");
            sender.send(Err(err)).unwrap_or_else(|_| {
                tracing::warn!("Failed to send media asset download error to receiver.");
            });
        } else {
            sender.send(result).unwrap_or_else(|_| {
                tracing::warn!("Failed to send media asset to receiver.");
            });
        }
    }
}

async fn make_temporary_dl_file() -> Result<(File, String), MediaDownloadError> {
    // Create a temporary file to download the file into.
    let mut media_root: PathBuf = env::var("MEDIA_ROOT").unwrap().into();
    let file_name = format!("{}.tmp", uuid::Uuid::new_v4());
    media_root.push(file_name);
    let file = File::create(&media_root)
        .await
        .map_err(MediaDownloadError::FileManagementError)?;
    Ok((file, media_root.to_str().unwrap().to_string()))
}

/// Download a file from a URL.
///
/// If this function returns successfully, the file will be downloaded and stored,
/// and the hash and size will be calculated.
/// The [`MediaAsset`] corresponds to the new file.
/// This `MediaAsset` was not yet stored in the database, the caller must do that.
async fn download_file(
    client: &reqwest::Client,
    url: &str,
) -> Result<MediaAsset, MediaDownloadError> {
    // Download a file from a URL.

    let response = client
        .get(url)
        .header("User-Agent", common::USER_AGENT)
        .send()
        .await
        .map_err(MediaDownloadError::WebRequestError)?;

    // Stream the response body into a temporary file
    let (mut file, file_path) = make_temporary_dl_file().await?;
    let mut stream = response.bytes_stream();
    let mut hasher = sha2::Sha256::new();
    let mut size = 0;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(MediaDownloadError::DownloadError)?;
        file.write_all(&chunk)
            .await
            .map_err(MediaDownloadError::FileManagementError)?;
        hasher.update(&chunk);
        size += chunk.len();
        if size > u32::MAX as usize {
            return Err(MediaDownloadError::FileTooBig);
        }
    }
    let size = size as u32;
    let hash: [u8; 32] = hasher.finalize().into();
    std::mem::drop(file); // Close the file

    // Figure out the MediaType of the file
    let extension = url.split('.').last().unwrap_or("");
    let media_type = MediaType::from_extension(extension);

    // Construct the MediaAsset
    let media_asset = MediaAsset {
        hash,
        size,
        media_type,
    };


    // Move the file to its final location
    // (and ensure that it exists)
    let mut media_root: PathBuf = env::var("MEDIA_ROOT").unwrap().into();
    let rel_media_path = media_asset.path();
    media_root.push(rel_media_path);
    tokio::fs::create_dir_all(media_root.parent().unwrap())
        .await
        .map_err(MediaDownloadError::FileManagementError)?;
    tokio::fs::rename(file_path, &media_root)
        .await
        .map_err(MediaDownloadError::FileManagementError)?;


    Ok(media_asset)
}

/// Try to add a media asset to the database.
///
/// If the media asset already exists, return the existing media asset.
/// If the media asset does not exist, add it to the database and return it.
/// If the media asset exists, but its type is unknown, update the media asset in the database.
///
/// In any case, also add a corresponding WebAsset to the database.
async fn try_adding_media_asset_to_db(
    url: &str,
    media_asset: MediaAsset,
    conn: &Database,
) -> Result<MediaAsset, MediaDownloadError> {
    // First we need to check: is there already a file with the same hash?
    let existing_media_asset = conn
        .get_media_asset_by_hash(&media_asset.hash)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get media asset by hash: {e:?}");
            MediaDownloadError::DatabaseError(e)
        })?;

    match existing_media_asset {
        None => {
            // If there isn't, we can just insert the new media asset into the database.
            conn.create_media_asset(&media_asset).await.map_err(|e| {
                tracing::error!("Failed to insert media asset: {e:?}");
                MediaDownloadError::DatabaseError(e)
            })?;
            // Then we can add a corresponding WebAsset to the database.
            conn.link_web_asset(&media_asset, url).await.map_err(|e| {
                tracing::error!("Failed to link web asset: {e:?}");
                MediaDownloadError::DatabaseError(e)
            })?;
            // and then send the result.
            Ok(media_asset)
        }
        Some(existing_media_asset) => {
            // If there is, check that the file sizes match.
            // If they do, it's a very good sign that the files are the same.
            let result = if existing_media_asset.size == media_asset.size {
                // If they do, then we might want to update the existing media asset:
                // - If the existing media asset's media type is unknown, we can update it to the new media type.
                if existing_media_asset.media_type == MediaType::Unknown {
                    conn.add_media_asset(&media_asset).await.map_err(|e| {
                        tracing::error!("Failed to update media asset: {e:?}");
                        MediaDownloadError::DatabaseError(e)
                    })?;
                    // Then we can add a corresponding WebAsset to the database.
                    conn.link_web_asset(&media_asset, url).await.map_err(|e| {
                        tracing::error!("Failed to link web asset: {e:?}");
                        MediaDownloadError::DatabaseError(e)
                    })?;
                    Ok(media_asset)
                } else {
                    // - If the existing media asset's media type is known, we can just return the existing media asset.
                    // But we still need to add a corresponding WebAsset to the database.
                    conn.link_web_asset(&existing_media_asset, url)
                        .await
                        .map_err(|e| {
                            tracing::error!("Failed to link web asset: {e:?}");
                            MediaDownloadError::DatabaseError(e)
                        })?;
                    Ok(existing_media_asset)
                }
            } else {
                // If the hashes match, but the file sizes don't, then we have a big problem
                // because the existence of two files with the same hash, but different sizes
                // implies that the hash function is broken.
                tracing::error!("FATAL ERROR!!!! -> tried downloading URL {url}, and downloaded media asset {media_asset:?}, but there is already a different media asset with the same hash in the database, {existing_media_asset:?}");
                panic!("FATAL ERROR!!!! -> tried downloading URL {url}, and downloaded media asset {media_asset:?}, but there is already a different media asset with the same hash in the database, {existing_media_asset:?}");
            };

            result
        }
    }
}

#[derive(Debug)]
pub enum MediaDownloadError {
    /// An error happened while creating, writing to, moving or deleting a file.
    FileManagementError(std::io::Error),
    /// An error happened while making a web request.
    WebRequestError(reqwest::Error),
    /// An error happened during the download process.
    DownloadError(reqwest::Error),
    /// The file download is bigger than 4GB.
    FileTooBig,
    /// An error happened while recording the media asset or associated information in the database.
    DatabaseError(sqlx::Error),
}
