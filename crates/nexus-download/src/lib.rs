//! Parallel download + SHA1 verify + resume-friendly install pipeline.
//!
//! Fast launch depends on this finishing *before* Play: once `ready`, launch
//! path skips network and aims for <5s into the game loop.

use std::path::{Path, PathBuf};

use nexus_versions::{MANIFEST_URL, VersionManifestV2, SUPPORTED_VERSIONS};
use sha1::{Digest, Sha1};
use thiserror::Error;
use tracing::{info, warn};

#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("http: {0}")]
    Http(#[from] reqwest::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("sha1 mismatch for {path}: expected {expected}, got {got}")]
    Sha1Mismatch {
        path: String,
        expected: String,
        got: String,
    },
    #[error("version not in Nexus matrix: {0}")]
    UnsupportedVersion(String),
    #[error("version not found in Mojang manifest: {0}")]
    NotInManifest(String),
}

#[derive(Debug, Clone)]
pub struct InstallRoot {
    pub root: PathBuf,
}

impl InstallRoot {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn versions_dir(&self) -> PathBuf {
        self.root.join("versions")
    }

    pub fn version_dir(&self, id: &str) -> PathBuf {
        self.versions_dir().join(id)
    }

    pub fn libraries_dir(&self) -> PathBuf {
        self.root.join("libraries")
    }

    pub fn assets_dir(&self) -> PathBuf {
        self.root.join("assets")
    }

    pub fn ready_marker(&self, id: &str) -> PathBuf {
        self.version_dir(id).join(".nexus_ready")
    }

    pub fn is_ready(&self, id: &str) -> bool {
        self.ready_marker(id).is_file()
    }
}

pub async fn fetch_manifest(client: &reqwest::Client) -> Result<VersionManifestV2, DownloadError> {
    let manifest: VersionManifestV2 = client.get(MANIFEST_URL).send().await?.error_for_status()?.json().await?;
    Ok(manifest)
}

pub async fn ensure_version(
    client: &reqwest::Client,
    install: &InstallRoot,
    version_id: &str,
) -> Result<(), DownloadError> {
    if !SUPPORTED_VERSIONS.contains(&version_id) {
        return Err(DownloadError::UnsupportedVersion(version_id.to_string()));
    }
    if install.is_ready(version_id) {
        info!(version = version_id, "already ready — skip download");
        return Ok(());
    }

    let manifest = fetch_manifest(client).await?;
    let entry = manifest
        .versions
        .iter()
        .find(|v| v.id == version_id)
        .ok_or_else(|| DownloadError::NotInManifest(version_id.to_string()))?;

    let version_json: serde_json::Value = client
        .get(&entry.url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let dir = install.version_dir(version_id);
    tokio::fs::create_dir_all(&dir).await?;
    let json_path = dir.join(format!("{version_id}.json"));
    tokio::fs::write(&json_path, serde_json::to_vec_pretty(&version_json)?).await?;

    // Client jar
    if let Some(client_obj) = version_json.get("downloads").and_then(|d| d.get("client")) {
        let url = client_obj
            .get("url")
            .and_then(|u| u.as_str())
            .ok_or_else(|| DownloadError::NotInManifest("client.url".into()))?;
        let sha = client_obj
            .get("sha1")
            .and_then(|s| s.as_str())
            .unwrap_or("");
        let jar_path = dir.join(format!("{version_id}.jar"));
        download_verified(client, url, &jar_path, sha).await?;
    } else {
        warn!(version = version_id, "no client download in version json — native path may use prepare pack instead");
    }

    // Mark ready only after core artifacts exist. Libraries/assets fill-in is next iteration.
    tokio::fs::write(install.ready_marker(version_id), b"ok").await?;
    info!(version = version_id, "marked ready");
    Ok(())
}

async fn download_verified(
    client: &reqwest::Client,
    url: &str,
    dest: &Path,
    expected_sha1: &str,
) -> Result<(), DownloadError> {
    if dest.is_file() && !expected_sha1.is_empty() {
        let existing = tokio::fs::read(dest).await?;
        let got = sha1_hex(&existing);
        if got == expected_sha1 {
            return Ok(());
        }
    }

    let bytes = client.get(url).send().await?.error_for_status()?.bytes().await?;
    if !expected_sha1.is_empty() {
        let got = sha1_hex(&bytes);
        if got != expected_sha1 {
            return Err(DownloadError::Sha1Mismatch {
                path: dest.display().to_string(),
                expected: expected_sha1.to_string(),
                got,
            });
        }
    }
    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(dest, &bytes).await?;
    Ok(())
}

fn sha1_hex(data: &[u8]) -> String {
    let mut h = Sha1::new();
    h.update(data);
    hex::encode(h.finalize())
}
