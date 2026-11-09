//! Install pipeline: version JSON, client jar, libraries (rule-filtered), asset index + objects.
//! Parallel downloads with SHA1 verify. Ready marker only after core success.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use futures::stream::{self, StreamExt};
use nexus_versions::{MANIFEST_URL, SUPPORTED_VERSIONS, VersionManifestV2};
use serde::Deserialize;
use sha1::{Digest, Sha1};
use thiserror::Error;
use tracing::{info, warn};

const MAX_PARALLEL: usize = 8;
const ASSET_BASE: &str = "https://resources.download.minecraft.net";

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
    #[error("missing field: {0}")]
    MissingField(String),
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

#[derive(Debug, Deserialize)]
struct VersionFile {
    downloads: Option<Downloads>,
    libraries: Option<Vec<Library>>,
    #[serde(rename = "assetIndex")]
    asset_index: Option<AssetIndexRef>,
}

#[derive(Debug, Deserialize)]
struct Downloads {
    client: Option<Artifact>,
}

#[derive(Debug, Deserialize)]
struct Artifact {
    url: String,
    sha1: String,
    path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Library {
    name: String,
    downloads: Option<LibraryDownloads>,
    rules: Option<Vec<Rule>>,
}

#[derive(Debug, Deserialize)]
struct LibraryDownloads {
    artifact: Option<Artifact>,
}

#[derive(Debug, Deserialize)]
struct Rule {
    action: String,
    os: Option<OsRule>,
}

#[derive(Debug, Deserialize)]
struct OsRule {
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AssetIndexRef {
    id: String,
    url: String,
    sha1: String,
}

#[derive(Debug, Deserialize)]
struct AssetIndexFile {
    objects: std::collections::HashMap<String, AssetObject>,
}

#[derive(Debug, Deserialize)]
struct AssetObject {
    hash: String,
    #[allow(dead_code)]
    size: u64,
}

#[derive(Clone)]
struct Job {
    url: String,
    dest: PathBuf,
    sha1: String,
}

pub async fn fetch_manifest(client: &reqwest::Client) -> Result<VersionManifestV2, DownloadError> {
    Ok(client
        .get(MANIFEST_URL)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?)
}

/// Full install for a matrix version: JSON + client + libraries + assets → ready.
pub async fn ensure_version(
    client: &reqwest::Client,
    install: &InstallRoot,
    version_id: &str,
) -> Result<(), DownloadError> {
    if !SUPPORTED_VERSIONS.contains(&version_id) {
        return Err(DownloadError::UnsupportedVersion(version_id.to_string()));
    }
    if install.is_ready(version_id) {
        info!(version = version_id, "already ready");
        return Ok(());
    }

    let manifest = fetch_manifest(client).await?;
    let entry = manifest
        .versions
        .iter()
        .find(|v| v.id == version_id)
        .ok_or_else(|| DownloadError::NotInManifest(version_id.to_string()))?;

    let raw = client
        .get(&entry.url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    let version: VersionFile = serde_json::from_slice(&raw)?;

    let dir = install.version_dir(version_id);
    tokio::fs::create_dir_all(&dir).await?;
    tokio::fs::write(dir.join(format!("{version_id}.json")), &raw).await?;

    let mut jobs: Vec<Job> = Vec::new();

    if let Some(client_art) = version.downloads.as_ref().and_then(|d| d.client.as_ref()) {
        jobs.push(Job {
            url: client_art.url.clone(),
            dest: dir.join(format!("{version_id}.jar")),
            sha1: client_art.sha1.clone(),
        });
    } else {
        warn!(version = version_id, "no client jar in metadata");
    }

    if let Some(libs) = &version.libraries {
        for lib in libs {
            if !library_allowed_android(lib) {
                continue;
            }
            if let Some(art) = lib.downloads.as_ref().and_then(|d| d.artifact.as_ref()) {
                let rel = art
                    .path
                    .clone()
                    .unwrap_or_else(|| maven_path_from_name(&lib.name));
                jobs.push(Job {
                    url: art.url.clone(),
                    dest: install.libraries_dir().join(rel),
                    sha1: art.sha1.clone(),
                });
            }
        }
    }

    // Asset index + objects
    if let Some(ai) = &version.asset_index {
        let index_path = install
            .assets_dir()
            .join("indexes")
            .join(format!("{}.json", ai.id));
        download_verified(client, &ai.url, &index_path, &ai.sha1).await?;
        let index_bytes = tokio::fs::read(&index_path).await?;
        let index: AssetIndexFile = serde_json::from_slice(&index_bytes)?;
        for obj in index.objects.values() {
            let prefix = &obj.hash[..2];
            let dest = install
                .assets_dir()
                .join("objects")
                .join(prefix)
                .join(&obj.hash);
            if dest.is_file() {
                continue;
            }
            jobs.push(Job {
                url: format!("{ASSET_BASE}/{prefix}/{}", obj.hash),
                dest,
                sha1: obj.hash.clone(),
            });
        }
    }

    info!(version = version_id, jobs = jobs.len(), "downloading");
    run_jobs(client, jobs).await?;

    tokio::fs::write(install.ready_marker(version_id), b"ok").await?;
    info!(version = version_id, "ready");
    Ok(())
}

fn library_allowed_android(lib: &Library) -> bool {
    let rules = match &lib.rules {
        None => return true,
        Some(r) if r.is_empty() => return true,
        Some(r) => r,
    };
    // Default disallow if any rule exists, then apply in order (Mojang style).
    let mut allow = false;
    for rule in rules {
        let os_match = match &rule.os {
            None => true,
            Some(os) => match os.name.as_deref() {
                // Treat Android closer to linux for classifier-less artifacts.
                Some("linux") | None => true,
                Some("windows") | Some("osx") => false,
                _ => false,
            },
        };
        if !os_match {
            continue;
        }
        allow = rule.action == "allow";
    }
    allow
}

fn maven_path_from_name(name: &str) -> String {
    // group:artifact:version[:classifier]
    let parts: Vec<&str> = name.split(':').collect();
    if parts.len() < 3 {
        return name.replace(':', "/");
    }
    let group = parts[0].replace('.', "/");
    let artifact = parts[1];
    let version = parts[2];
    let file = if parts.len() >= 4 {
        format!("{artifact}-{version}-{}.jar", parts[3])
    } else {
        format!("{artifact}-{version}.jar")
    };
    format!("{group}/{artifact}/{version}/{file}")
}

async fn run_jobs(client: &reqwest::Client, jobs: Vec<Job>) -> Result<(), DownloadError> {
    let client = Arc::new(client.clone());
    let results: Vec<Result<(), DownloadError>> = stream::iter(jobs)
        .map(|job| {
            let client = Arc::clone(&client);
            async move { download_verified(&client, &job.url, &job.dest, &job.sha1).await }
        })
        .buffer_unordered(MAX_PARALLEL)
        .collect()
        .await;

    for r in results {
        r?;
    }
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
        if sha1_hex(&existing) == expected_sha1 {
            return Ok();
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
