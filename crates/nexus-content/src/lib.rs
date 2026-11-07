//! Skins, resource packs, shaders — **no mod browser**.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContentError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid skin: {0}")]
    InvalidSkin(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkinSlot {
    pub name: String,
    pub path: PathBuf,
    /// Classic 64x64 or 64x64 slim (Alex).
    pub slim: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePackEntry {
    pub name: String,
    pub path: PathBuf,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderPackEntry {
    pub name: String,
    pub path: PathBuf,
    pub enabled: bool,
}

/// Content roots under the instance / global data dir.
#[derive(Debug, Clone)]
pub struct ContentPaths {
    pub skins: PathBuf,
    pub resourcepacks: PathBuf,
    pub shaderpacks: PathBuf,
}

impl ContentPaths {
    pub fn under(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        Self {
            skins: root.join("skins"),
            resourcepacks: root.join("resourcepacks"),
            shaderpacks: root.join("shaderpacks"),
        }
    }

    pub async fn ensure_dirs(&self) -> Result<(), ContentError> {
        tokio::fs::create_dir_all(&self.skins).await?;
        tokio::fs::create_dir_all(&self.resourcepacks).await?;
        tokio::fs::create_dir_all(&self.shaderpacks).await?;
        Ok(())
    }
}

/// Local browse helpers (filesystem only — remote catalog can plug in later).
pub async fn list_resource_packs(dir: &PathBuf) -> Result<Vec<ResourcePackEntry>, ContentError> {
    list_zip_or_dir(dir, |name, path| ResourcePackEntry {
        name,
        path,
        enabled: false,
    })
    .await
}

pub async fn list_shader_packs(dir: &PathBuf) -> Result<Vec<ShaderPackEntry>, ContentError> {
    list_zip_or_dir(dir, |name, path| ShaderPackEntry {
        name,
        path,
        enabled: false,
    })
    .await
}

pub async fn list_skins(dir: &PathBuf) -> Result<Vec<SkinSlot>, ContentError> {
    let mut out = Vec::new();
    let mut rd = match tokio::fs::read_dir(dir).await {
        Ok(r) => r,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(out),
        Err(e) => return Err(e.into()),
    };
    while let Some(entry) = rd.next_entry().await? {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("png") {
            let name = path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "skin".into());
            out.push(SkinSlot {
                name,
                path,
                slim: false,
            });
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

async fn list_zip_or_dir<T, F>(dir: &PathBuf, map: F) -> Result<Vec<T>, ContentError>
where
    F: Fn(String, PathBuf) -> T,
{
    let mut out = Vec::new();
    let mut rd = match tokio::fs::read_dir(dir).await {
        Ok(r) => r,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(out),
        Err(e) => return Err(e.into()),
    };
    while let Some(entry) = rd.next_entry().await? {
        let path = entry.path();
        let is_pack = path.is_dir()
            || path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("zip"))
                .unwrap_or(false);
        if !is_pack {
            continue;
        }
        let name = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "pack".into());
        out.push(map(name, path));
    }
    Ok(out)
}
