//! Runtime session + smooth frame loop + software scene.

use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

use nexus_render::{draw_scene, write_ppm, Framebuffer};
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
                fps_cap: 0,
                render_distance: 6,
                simulation_distance: 4,
                entity_distance_percent: 50,
                resolution_scale_percent: 75,
                vsync: false,
                max_chunk_mesh_jobs: 2,
                particle_multiplier_percent: 40,
                prefer_vulkan: true,
                adaptive_quality: true,
                frame_budget_us: 16_666,
            },
            PerformanceProfile::Balanced => Self {
                profile,
                fps_cap: 0,
                render_distance: 8,
                simulation_distance: 6,
                entity_distance_percent: 75,
                resolution_scale_percent: 90,
                vsync: false,
                max_chunk_mesh_jobs: 3,
                particle_multiplier_percent: 70,
                prefer_vulkan: true,
                adaptive_quality: true,
                frame_budget_us: 16_666,
            },
            PerformanceProfile::Quality => Self {
                profile,
                fps_cap: 0,
                render_distance: 12,
                simulation_distance: 8,
                entity_distance_percent: 100,
                resolution_scale_percent: 100,
                vsync: false,
                max_chunk_mesh_jobs: 4,
                particle_multiplier_percent: 100,
                prefer_vulkan: true,
                adaptive_quality: true,
                frame_budget_us: 16_666,
            },
        }
    }

    pub fn frame_duration(&self) -> Duration {
        if self.fps_cap == 0 {
            return Duration::from_micros(1);
        }
        Duration::from_micros(1_000_000 / u64::from(self.fps_cap.max(1)))
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

#[derive(Debug, Clone, Default)]
pub struct FrameStats {
    pub frames: u64,
    pub over_budget: u64,
    pub degrade_steps: u32,
    pub last_frame_us: u128,
    pub avg_frame_us: u128,
}

pub struct EngineSupervisor {
    active: Option<SessionHandle>,
    stats: FrameStats,
    fb: Option<Framebuffer>,
}

impl Default for EngineSupervisor {
    fn default() -> Self {
        Self {
            active: None,
            stats: FrameStats::default(),
            fb: None,
        }
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
        self.stats = FrameStats::default();
        self.fb = Some(Framebuffer::from_scale(
            1280,
            720,
            handle.smooth.resolution_scale_percent,
        ));
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

    pub fn stats(&self) -> &FrameStats {
        &self.stats
    }

    pub fn active(&self) -> Option<&SessionHandle> {
        self.active.as_ref()
    }

    pub fn save_preview(&self, path: &std::path::Path) -> Result<(), EngineError> {
        let Some(fb) = self.fb.as_ref() else {
            return Err(EngineError::NotRunning);
        };
        write_ppm(fb, path).map_err(|_| EngineError::NotRunning)
    }

    pub fn framebuffer_size(&self) -> Option<(u32, u32)> {
        self.fb.as_ref().map(|f| (f.width, f.height))
    }

    pub fn tick(&mut self) -> Result<(), EngineError> {
        let Some(session) = self.active.as_mut() else {
            return Err(EngineError::NotRunning);
        };
        if !session.running {
            return Err(EngineError::NotRunning);
        }

        let budget = session.smooth.frame_duration();
        let t0 = Instant::now();

        let tick_n = self.stats.frames;
        let rd = session.smooth.render_distance;
        if let Some(fb) = self.fb.as_mut() {
            draw_scene(fb, tick_n, rd);
        }

        let elapsed = t0.elapsed();
        self.stats.frames += 1;
        self.stats.last_frame_us = elapsed.as_micros();
        let n = self.stats.frames;
        self.stats.avg_frame_us = ((self.stats.avg_frame_us * ((n - 1) as u128))
            + self.stats.last_frame_us)
            / (n as u128);

        let capped = session.smooth.fps_cap > 0;
        if capped {
            if elapsed > budget {
                self.stats.over_budget += 1;
                if session.smooth.adaptive_quality {
                    session.smooth.degrade_one_step();
                    self.stats.degrade_steps += 1;
                }
            } else if let Some(rest) = budget.checked_sub(elapsed) {
                if session.smooth.vsync {
                    thread::sleep(rest);
                }
            }
        } else if elapsed > Duration::from_millis(50) && session.smooth.adaptive_quality {
            self.stats.over_budget += 1;
            session.smooth.degrade_one_step();
            self.stats.degrade_steps += 1;
        }

        Ok(())
    }

    pub fn run_for(&mut self, duration: Duration) -> Result<FrameStats, EngineError> {
        let end = Instant::now() + duration;
        while Instant::now() < end {
            if !self.active.as_ref().map(|s| s.running).unwrap_or(false) {
                break;
            }
            self.tick()?;
        }
        Ok(self.stats.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unlimited_by_default() {
        let p = SmoothConfig::from_profile(PerformanceProfile::Balanced);
        assert_eq!(p.fps_cap, 0);
        assert!(!p.vsync);
    }
}
