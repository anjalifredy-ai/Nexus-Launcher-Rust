//! Official version matrix for Nexus.

pub const SUPPORTED_VERSIONS: &[&str] = &[
    "1.21.1", "1.21.2", "1.21.3", "1.21.4", "1.21.5", "1.21.6", "1.21.7", "1.21.8",
    "1.21.9", "1.21.10", "1.21.11", "26.1", "26.2", "26.3",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKind {
    HorizonNative,
    MultiVersionRuntime,
}

#[derive(Debug, Clone)]
pub struct VersionInfo {
    pub id: String,
    pub backend: BackendKind,
    pub ready: bool,
}

impl VersionInfo {
    pub fn for_id(id: &str) -> Option<Self> {
        if !SUPPORTED_VERSIONS.contains(&id) {
            return None;
        }
        let backend = if id.starts_with("26.") {
            BackendKind::HorizonNative
        } else {
            BackendKind::MultiVersionRuntime
        };
        Some(Self {
            id: id.to_string(),
            backend,
            ready: false,
        })
    }
}

pub const MANIFEST_URL: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_covers_requested_ids() {
        assert!(SUPPORTED_VERSIONS.contains(&"1.21.1"));
        assert!(SUPPORTED_VERSIONS.contains(&"1.21.11"));
        assert!(SUPPORTED_VERSIONS.contains(&"26.3"));
        assert_eq!(SUPPORTED_VERSIONS.len(), 14);
    }

    #[test]
    fn backend_split() {
        assert_eq!(
            VersionInfo::for_id("26.3").unwrap().backend,
            BackendKind::HorizonNative
        );
        assert_eq!(
            VersionInfo::for_id("1.21.4").unwrap().backend,
            BackendKind::MultiVersionRuntime
        );
    }
}
