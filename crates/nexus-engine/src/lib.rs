//! Runtime session + smoothness budgets for mobile.

use std::path::PathBuf;

use nexus_versions::BackendKind;

#[derive(Debug)]
pub enum EngineError {
    AlreadyRunning,
    NotRunning,
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyRunning => write!(f, "session already running"),
            Self::NotRunning => write!(f, "session not running"),
        }
    }
}

impl std::error::Error for EngineError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PerformanceProfile {
    Performance,
    #[default]
    Balanced,
    Quality,
}

#[derive(Debug, Clone)]
pub struct SmoothConfig {
    pub profile: PerformanceProfile,
    pub fps_cap: u32,
    pub render_distance: u8,
    pub simulation_distance: u8,
    pub entity_distance_percent: u8,
    pub resolution_scale_percent: u8,
    pub vsync: bool,
    pub max_chunk_mesh_jobs: u8,
    pub particle_multiplier_percent: u8,
    pub prefer_vulkan: bool,
    pub adaptive_quality: bool,
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
    pub instance_dir: PathBuf,
    pub game_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct SessionHandle {
    pub version_id: String,
    pub backend: BackendKind,
    pub smooth: SmoothConfig,
    pub running: bool,
}

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
                Ok(())
            }
            _ => Err(EngineError::NotRunning),
        }
    }

    pub fn on_frame_over_budget(&mut self) {
        if let Some(s) = self.active.as_mut() {
            if s.smooth.adaptive_quality {
                s.smooth.degrade_one_step();
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
    fn session_starts_and_stops() {
        let mut eng = EngineSupervisor::default();
        let h = eng
            .start(SessionRequest {
                version_id: "26.3".into(),
                backend: BackendKind::HorizonNative,
                smooth: SmoothConfig::from_profile(PerformanceProfile::Balanced),
                instance_dir: PathBuf::from("/tmp/nexus-i"),
                game_dir: PathBuf::from("/tmp/nexus-g"),
            })
            .unwrap();
        assert!(h.running);
        eng.stop().unwrap();
    }
}
