//! Fast + smooth launch path.
//!
//! Ready version → no network → apply SmoothConfig → engine session.

use std::path::PathBuf;
use std::time::Instant;

use nexus_download::InstallRoot;
use nexus_engine::{
    EngineSupervisor, PerformanceProfile, SessionRequest, SmoothConfig,
};
use nexus_versions::VersionInfo;
use thiserror::Error;
use tracing::info;

#[derive(Debug, Error)]
pub enum LaunchError {
    #[error("version not supported: {0}")]
    Unsupported(String),
    #[error("version not installed / not ready: {0}")]
    NotReady(String),
    #[error(transparent)]
    Engine(#[from] nexus_engine::EngineError),
}

#[derive(Debug, Clone)]
pub struct LaunchRequest {
    pub version_id: String,
    pub instance_dir: PathBuf,
    pub performance: PerformanceProfile,
}

#[derive(Debug, Clone)]
pub struct LaunchReport {
    pub version_id: String,
    pub backend: nexus_versions::BackendKind,
    pub prepare_ms: u128,
    pub handoff_ms: u128,
    pub smooth: SmoothConfig,
}

pub async fn launch(
    install: &InstallRoot,
    engine: &mut EngineSupervisor,
    req: LaunchRequest,
) -> Result<LaunchReport, LaunchError> {
    let start = Instant::now();
    let info = VersionInfo::for_id(&req.version_id)
        .ok_or_else(|| LaunchError::Unsupported(req.version_id.clone()))?;

    if !install.is_ready(&req.version_id) {
        return Err(LaunchError::NotReady(req.version_id));
    }

    let smooth = SmoothConfig::from_profile(req.performance);
    let prepare_ms = start.elapsed().as_millis();

    let game_dir = install.version_dir(&req.version_id);
    tokio::fs::create_dir_all(&req.instance_dir)
        .await
        .map_err(|e| LaunchError::NotReady(format!("instance dir: {e}")))?;

    let handle = engine.start(SessionRequest {
        version_id: req.version_id.clone(),
        backend: info.backend,
        smooth: smooth.clone(),
        instance_dir: req.instance_dir,
        game_dir,
    })?;

    let handoff_ms = start.elapsed().as_millis();
    info!(
        version = %handle.version_id,
        handoff_ms,
        rd = handle.smooth.render_distance,
        "launch handoff complete"
    );

    Ok(LaunchReport {
        version_id: handle.version_id,
        backend: handle.backend,
        prepare_ms,
        handoff_ms,
        smooth: handle.smooth,
    })
}
