//! Runtime session + smoothness budgets.
//!
//! Goal: stable frametimes on mobile — not just peak FPS spikes.

use nexus_versions::BackendKind;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::info;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("session already running")]
    AlreadyRunning,
    #[error("session not running")]
    NotRunning,
    #[error("backend not implemented: {0:?}")]
    BackendPending(BackendKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PerformanceProfile {
    Performance,
    #[default]
    Balanced,
    Quality,
}

/// Tunables that keep the game smooth under thermal / memory pressure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmoothConfig {
    pub profile: PerformanceProfile,
    /// Hard FPS cap (0 = uncapped). Mobile default 60 for stability.
    pub fps_cap: u32,
    pub render_distance: u8,
    pub simulation_distance: u8,
    pub entity_distance_percent: u8,
    /// 100 = native res; lower = GPU relief.
    pub resolution_scale_percent: u8,
    pub vsync: bool,
    pub max_chunk_mesh_jobs: u8,
    pub particle_multiplier_percent: u8,
    /// Prefer Vulkan when available.
    pub prefer_vulkan: bool,
    /// Drop quality automatically if frame time exceeds budget.
    pub adaptive_quality: bool,
    /// Target frame budget in microseconds (16666 ≈ 60 FPS).
    pub frame_budget_us: u32,
}

impl SmoothConfig {
    pub fn from_profile(profile: PerformanceProfile) -> Self {
        match profile {
            PerformanceProfile::Performance => Self {
                profile,
                fps_cap: 60,
                render_distance: 6,
                simulation_distance: 4,
                entity_distance_percent: 50,
                resolution_scale_percent: 75,
                vsync: true,
                max_chunk_mesh_jobs: 2,
                particle_multiplier_percent: 40,
                prefer_vulkan: true,
                adaptive_quality: true,
                frame_budget_us: 16_666,
            },
            PerformanceProfile::Balanced => Self {
                profile,
                fps_cap: 60,
                render_distance: 8,
                simulation_distance: 6,
                entity_distance_percent: 75,
                resolution_scale_percent: 90,
                vsync: true,
                max_chunk_mesh_jobs: 3,
                particle_multiplier_percent: 70,
                prefer_vulkan: true,
                adaptive_quality: true,
                frame_budget_us: 16_666,
            },
            PerformanceProfile::Quality => Self {
                profile,
                fps_cap: 60,
                render_distance: 12,
                simulation_distance: 8,
                entity_distance_percent: 100,
                resolution_scale_percent: 100,
                vsync: true,
                max_chunk_mesh_jobs: 4,
                particle_multiplier_percent: 100,
                prefer_vulkan: true,
                adaptive_quality: true,
                frame_budget_us: 16_666,
            },
        }
    }

    /// Step quality down one notch (thermal / hitch recovery).
    pub fn degrade_one_step(&mut self) {
        if self.resolution_scale_percent > 60 {
            self.resolution_scale_percent = self.resolution_scale_percent.saturating_sub(5);
        }
        if self.render_distance > 4 {
            self.render_distance -= 1;
        }
        if self.particle_multiplier_percent > 20 {
            self.particle_multiplier_percent =
                self.particle_multiplier_percent.saturating_sub(10);
        }
    }
}

#[derive(Debug, Clone)]
pub struct SessionRequest {
    pub version_id: String,
    pub backend: BackendKind,
    pub smooth: SmoothConfig,
    pub instance_dir: std::path::PathBuf,
    pub game_dir: std::path::PathBuf,
}

#[derive(Debug, Clone)]
pub struct SessionHandle {
    pub version_id: String,
    pub backend: BackendKind,
    pub smooth: SmoothConfig,
    pub running: bool,
}

/// Engine supervisor — one active session at a time on mobile.
pub struct EngineSupervisor {
    active: Option<SessionHandle>,
}

impl Default for EngineSupervisor {
    fn default() -> Self {
        Self { active: None }
    }
}

impl EngineSupervisor {
    pub fn start(&mut self, req: SessionRequest) -> Result<SessionHandle, EngineError> {
        if self.active.as_ref().map(|s| s.running).unwrap_or(false) {
            return Err(EngineError::AlreadyRunning);
        }

        info!(
            version = %req.version_id,
            backend = ?req.backend,
            fps_cap = req.smooth.fps_cap,
            rd = req.smooth.render_distance,
            scale = req.smooth.resolution_scale_percent,
            "starting smooth session"
        );

        // Backend-specific entry points land here as binaries link in.
        // Until then we still enforce single-session + smooth config so the
        // launcher path is production-shaped.
        let _ = (&req.instance_dir, &req.game_dir);

        let handle = SessionHandle {
            version_id: req.version_id,
            backend: req.backend,
            smooth: req.smooth,
            running: true,
        };
        self.active = Some(handle.clone());
        Ok(handle)
    }

    pub fn stop(&mut self) -> Result<(), EngineError> {
        match self.active.as_mut() {
            Some(s) if s.running => {
                s.running = false;
                info!(version = %s.version_id, "session stopped");
                Ok(())
            }
            _ => Err(EngineError::NotRunning),
        }
    }

    pub fn on_frame_over_budget(&mut self) {
        if let Some(s) = self.active.as_mut() {
            if s.smooth.adaptive_quality {
                s.smooth.degrade_one_step();
                info!(
                    rd = s.smooth.render_distance,
                    scale = s.smooth.resolution_scale_percent,
                    "adaptive quality step-down"
                );
            }
        }
    }

    pub fn active(&self) -> Option<&SessionHandle> {
        self.active.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn performance_is_lighter_than_quality() {
        let p = SmoothConfig::from_profile(PerformanceProfile::Performance);
        let q = SmoothConfig::from_profile(PerformanceProfile::Quality);
        assert!(p.render_distance < q.render_distance);
        assert!(p.resolution_scale_percent < q.resolution_scale_percent);
    }

    #[test]
    fn degrade_reduces_load() {
        let mut c = SmoothConfig::from_profile(PerformanceProfile::Balanced);
        let rd = c.render_distance;
        c.degrade_one_step();
        assert!(c.render_distance <= rd);
    }
}
