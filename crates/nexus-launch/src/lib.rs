//! Fast launch path.
//!
//! Contract: if version is `ready`, `launch` must not hit the network and should
//! reach the engine entry within the 5s budget on mid-range devices (prepare-once model).

use std::path::PathBuf;
use std::time::Instant;

use nexus_download::InstallRoot;
use nexus_versions::{BackendKind, VersionInfo};
use thiserror::Error;
use tracing::info;

#[derive(Debug, Error)]
pub enum LaunchError {
    #[error("version not supported: {0}")]
    Unsupported(String),
    #[error("version not installed / not ready: {0} — install first")]
    NotReady(String),
    #[error("engine entry not linked yet for backend {0:?}")]
    EngineNotLinked(BackendKind),
}

#[derive(Debug, Clone)]
pub struct LaunchRequest {
    pub version_id: String,
    pub instance_dir: PathBuf,
    pub performance: PerformanceProfile,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum PerformanceProfile {
    Performance,
    #[default]
    Balanced,
    Quality,
}

#[derive(Debug, Clone)]
pub struct LaunchReport {
    pub version_id: String,
    pub backend: BackendKind,
    pub prepare_ms: u128,
    pub handoff_ms: u128,
}

/// Validate readiness and hand off to the correct backend.
///
//! Full engine binaries are linked in later milestones; this enforces the
//! fast-path contract and returns structured timing.
pub async fn launch(install: &InstallRoot, req: LaunchRequest) -> Result<LaunchReport, LaunchError> {
    let start = Instant::now();
    let info = VersionInfo::for_id(&req.version_id)
        .ok_or_else(|| LaunchError::Unsupported(req.version_id.clone()))?;

    if !install.is_ready(&req.version_id) {
        return Err(LaunchError::NotReady(req.version_id));
    }

    let prepare_ms = start.elapsed().as_millis();

    // Backend handoff points — implement engine entry here as crates land.
    match info.backend {
        BackendKind::HorizonNative => {
            info!(version = %req.version_id, profile = ?req.performance, "handoff → horizon_native");
            // nexus_engine_horizon::start(...)
        }
        BackendKind::MultiVersionRuntime => {
            info!(version = %req.version_id, profile = ?req.performance, "handoff → multi_version");
            // nexus_engine_runtime::start(...)
        }
    }

    let handoff_ms = start.elapsed().as_millis();
    Ok(LaunchReport {
        version_id: req.version_id,
        backend: info.backend,
        prepare_ms,
        handoff_ms,
    })
}
