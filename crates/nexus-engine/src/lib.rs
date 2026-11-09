//! Session + frame loop + world + culled mesh + input player + height cache.

use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

use nexus_input::{InputState, Player};
use nexus_mesh::{mesh_chunk_tops, Mesh};
use nexus_render::{draw_scene_from_heights, write_ppm, Camera, Framebuffer};
use nexus_versions::BackendKind;
use nexus_world::World;

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
    camera: Camera,
    world: World,
    mesh: Mesh,
    player: Player,
    input: InputState,
    height_cache: Vec<u8>,
    cache_scroll: i32,
}

impl Default for EngineSupervisor {
    fn default() -> Self {
        Self {
            active: None,
            stats: FrameStats::default(),
            fb: None,
            camera: Camera::default(),
            world: World::default(),
            mesh: Mesh::default(),
            player: Player::default(),
            input: InputState::default(),
            height_cache: Vec::new(),
            cache_scroll: i32::MIN,
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
        self.mesh.clear();
        self.height_cache.clear();
        self.cache_scroll = i32::MIN;
        for cz in -2..=2 {
            for cx in -2..=2 {
                let _ = self.world.ensure_chunk(cx, cz);
                let ch = nexus_world::Chunk::generate(cx, cz);
                mesh_chunk_tops(&ch, &mut self.mesh);
            }
        }
        self.player = Player::default();
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
    pub fn world_chunks(&self) -> usize {
        self.world.chunk_count()
    }
    pub fn mesh_tris(&self) -> usize {
        self.mesh.triangle_count()
    }
    pub fn mesh_verts(&self) -> usize {
        self.mesh.vertices.len()
    }
    pub fn player_pos(&self) -> (f32, f32, f32) {
        (self.player.x, self.player.y, self.player.z)
    }
    pub fn set_input(&mut self, input: InputState) {
        self.input = input;
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

    fn rebuild_height_cache(&mut self, cols: usize, scroll: i32) {
        self.height_cache.clear();
        self.height_cache.reserve(cols);
        for c in 0..cols {
            let wx = c as i32 + scroll;
            let cx = wx.div_euclid(16);
            let lx = wx.rem_euclid(16) as usize;
            let ch = self.world.ensure_chunk(cx, 0);
            self.height_cache.push(ch.column(lx, 8).height);
        }
        self.cache_scroll = scroll;
    }

    pub fn tick(&mut self) -> Result<(), EngineError> {
        {
            let session = self.active.as_ref().ok_or(EngineError::NotRunning)?;
            if !session.running {
                return Err(EngineError::NotRunning);
            }
        }

        let t0 = Instant::now();
        let tick_n = self.stats.frames;

        self.player.integrate(&self.input, 1.0 / 60.0);
        if self.input.move_x == 0.0 && self.input.move_z == 0.0 {
            self.camera.advance(tick_n);
        } else {
            self.camera.x = self.player.x;
            self.camera.y = self.player.y * 0.1;
            self.camera.yaw = self.player.yaw;
        }
        let cam = self.camera;
        let rd = self.active.as_ref().unwrap().smooth.render_distance;
        let fps_cap = self.active.as_ref().unwrap().smooth.fps_cap;
        let vsync = self.active.as_ref().unwrap().smooth.vsync;
        let adaptive = self.active.as_ref().unwrap().smooth.adaptive_quality;
        let budget = self.active.as_ref().unwrap().smooth.frame_duration();

        let cols = ((rd as usize) * 2).clamp(10, 28);
        let scroll = (cam.x * 2.0) as i32 + (tick_n as i32 / 2);
        if self.cache_scroll != scroll || self.height_cache.len() != cols {
            self.rebuild_height_cache(cols, scroll);
        }
        let heights = self.height_cache.clone();
        if let Some(fb) = self.fb.as_mut() {
            draw_scene_from_heights(fb, tick_n, &heights, &cam);
        }
        self.input.clear_edges();

        let elapsed = t0.elapsed();
        self.stats.frames += 1;
        self.stats.last_frame_us = elapsed.as_micros();
        let n = self.stats.frames;
        self.stats.avg_frame_us = ((self.stats.avg_frame_us * ((n - 1) as u128))
            + self.stats.last_frame_us)
            / (n as u128);

        if fps_cap > 0 {
            if elapsed > budget {
                self.stats.over_budget += 1;
                if adaptive {
                    if let Some(s) = self.active.as_mut() {
                        s.smooth.degrade_one_step();
                    }
                    self.stats.degrade_steps += 1;
                }
            } else if let Some(rest) = budget.checked_sub(elapsed) {
                if vsync {
                    thread::sleep(rest);
                }
            }
        } else if elapsed > Duration::from_millis(50) && adaptive {
            self.stats.over_budget += 1;
            if let Some(s) = self.active.as_mut() {
                s.smooth.degrade_one_step();
            }
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
    }
}
