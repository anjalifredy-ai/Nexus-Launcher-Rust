//! Android shell contract — JNI/Activity bridge hooks.

use nexus_engine::{EngineSupervisor, PerformanceProfile, SessionRequest, SmoothConfig};
use nexus_versions::{BackendKind, VersionInfo};
use std::path::PathBuf;

pub struct AndroidLauncher {
    pub data_dir: PathBuf,
    engine: EngineSupervisor,
}

impl AndroidLauncher {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            data_dir,
            engine: EngineSupervisor::default(),
        }
    }

    pub fn supported_versions(&self) -> &'static [&'static str] {
        nexus_versions::SUPPORTED_VERSIONS
    }

    pub fn start_version(
        &mut self,
        version: &str,
        profile: PerformanceProfile,
    ) -> Result<String, String> {
        let info = VersionInfo::for_id(version).ok_or_else(|| format!("bad version {version}"))?;
        let game_dir = self.data_dir.join("versions").join(version);
        let instance_dir = self.data_dir.join("instances").join(version);
        let _ = std::fs::create_dir_all(&game_dir);
        let _ = std::fs::create_dir_all(&instance_dir);
        let smooth = SmoothConfig::from_profile(profile);
        self.engine
            .start(SessionRequest {
                version_id: version.to_string(),
                backend: info.backend,
                smooth,
                instance_dir,
                game_dir,
            })
            .map_err(|e| e.to_string())?;
        let backend = match info.backend {
            BackendKind::HorizonNative => "HorizonNative",
            BackendKind::MultiVersionRuntime => "MultiVersionRuntime",
        };
        Ok(format!(
            "started {version} via {backend}; chunks={} tris={}",
            self.engine.world_chunks(),
            self.engine.mesh_tris()
        ))
    }

    pub fn tick_frames(&mut self, n: u32) -> Result<u64, String> {
        for _ in 0..n {
            self.engine.tick().map_err(|e| e.to_string())?;
        }
        Ok(self.engine.stats().frames)
    }

    pub fn stop(&mut self) -> Result<(), String> {
        self.engine.stop().map_err(|e| e.to_string())
    }
}
