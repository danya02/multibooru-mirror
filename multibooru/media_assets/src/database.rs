use std::env;

use crate::media_asset::MediaAsset;
use sqlx::SqlitePool;

pub(crate) struct Database {
    pub connection: SqlitePool,
}

impl Database {
    pub async fn new() -> Self {
        let connection =
            SqlitePool::connect(&env::var("DATABASE_URL").expect("DATABASE_URL must be set"))
                .await
                .expect("Failed to connect to media storage database.");
        Self { connection }
    }

    /// Insert a media asset into the database.
    ///
    /// This should be called after the media asset has been downloaded,
    /// meaning that its file exists on disk,
    /// and its hash and size are known.
    pub async fn create_media_asset(&self, media_asset: &MediaAsset) -> Result<(), sqlx::Error> {
        let hash_slice = &media_asset.hash[..];
        let media_type = media_asset.media_type;
        let media_type_json = serde_json::to_string(&media_type).unwrap();
        sqlx::query!(
            "INSERT INTO media_assets (
                sha256, media_type_json, size
            ) VALUES ($1, $2, $3)",
            hash_slice,
            media_type_json,
            media_asset.size
        )
        .execute(&self.connection)
        .await?;
        Ok(())
    }

    /// Try to get a media asset by its corresponding URL.
    ///
    /// If there is a media asset associated with the URL,
    /// it will be returned.
    pub async fn get_media_asset_by_url(&self, url: &str) -> Option<MediaAsset> {
        let row = sqlx::query!(
            "SELECT media_assets.sha256, media_type_json, size
                FROM media_assets
                INNER JOIN web_assets ON media_assets.sha256 = web_assets.sha256
                WHERE web_assets.url = $1",
            url
        )
        .fetch_optional(&self.connection)
        .await
        .unwrap();
        if let Some(row) = row {
            let media_type = serde_json::from_str(&row.media_type_json)
                .expect("Unparseable media type in database?!");
            let sha256: [u8; 32] = row
                .sha256
                .try_into()
                .expect("SHA256 hash in database is not 32 bytes long?!");
            Some(MediaAsset {
                hash: sha256,
                media_type,
                size: row.size as u32,
            })
        } else {
            None
        }
    }

    /// Try to get a media asset by its hash.
    pub async fn get_media_asset_by_hash(
        &self,
        hash: &[u8; 32],
    ) -> Result<Option<MediaAsset>, sqlx::Error> {
        let hash_slice = &hash[..];
        let row = sqlx::query!(
            "SELECT media_type_json, size
                FROM media_assets
                WHERE sha256 = $1",
            hash_slice
        )
        .fetch_optional(&self.connection)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get media asset by hash: {}", e);
            e
        })?;
        Ok(if let Some(row) = row {
            let media_type = serde_json::from_str(&row.media_type_json)
                .expect("Unparseable media type in database?!");
            Some(MediaAsset {
                hash: *hash,
                media_type,
                size: row.size as u32,
            })
        } else {
            None
        })
    }

    /// Add a media asset to the database.
    /// If the media asset already exists, update it.
    ///
    /// Uses the SQLite ON CONFLICT clause to do an upsert.
    pub async fn add_media_asset(&self, media_asset: &MediaAsset) -> Result<(), sqlx::Error> {
        let hash_slice = &media_asset.hash[..];
        let media_type = media_asset.media_type;
        let media_type_json = serde_json::to_string(&media_type).unwrap();
        sqlx::query!(
            "INSERT INTO media_assets (
                sha256, media_type_json, size
            ) VALUES ($1, $2, $3)
            ON CONFLICT (sha256) DO UPDATE SET
                media_type_json = excluded.media_type_json,
                size = excluded.size",
            hash_slice,
            media_type_json,
            media_asset.size
        )
        .execute(&self.connection)
        .await?;
        Ok(())
    }

    /// Add a web asset reference to the database.
    /// If the web asset already exists, update it.
    pub async fn link_web_asset(
        &self,
        media_asset: &MediaAsset,
        url: &str,
    ) -> Result<(), sqlx::Error> {
        let hash_slice = &media_asset.hash[..];
        sqlx::query!(
            "INSERT INTO web_assets (
                url, sha256
            ) VALUES ($1, $2)
            ON CONFLICT (url) DO UPDATE SET
                sha256 = excluded.sha256",
            url,
            hash_slice
        )
        .execute(&self.connection)
        .await?;
        Ok(())
    }
}
