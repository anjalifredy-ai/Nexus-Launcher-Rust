//! Nexus core — install, smooth launch, content, UI state.

pub use nexus_content as content;
pub use nexus_download as download;
pub use nexus_engine as engine;
pub use nexus_launch as launch;
pub use nexus_ui as ui;
pub use nexus_versions as versions;

use download::InstallRoot;
use engine::{EngineSupervisor, PerformanceProfile};
use launch::{LaunchRequest, LaunchReport};
use thiserror::Error;
use ui::LauncherState;

#[derive(Debug, Error)]
pub enum NexusError {
    #[error(transparent)]
    Download(#[from] download::DownloadError),
    #[error(transparent)]
    Launch(#[from] launch::LaunchError),
    #[error(transparent)]
    Content(#[from] content::ContentError),
    #[error(transparent)]
    Engine(#[from] engine::EngineError),
}

pub struct NexusApp {
    pub install: InstallRoot,
    pub state: LauncherState,
    pub engine: EngineSupervisor,
    client: reqwest::Client,
}

impl NexusApp {
    pub fn new(install_root: impl Into<std::path::PathBuf>) -> Self {
        let install = InstallRoot::new(install_root);
        let mut state = LauncherState::default();
        for v in state.versions.iter_mut() {
            v.ready = install.is_ready(&v.id);
        }
        Self {
            install,
            state,
            engine: EngineSupervisor::default(),
            client: reqwest::Client::builder()
                .user_agent("NexusLauncher/0.1")
                .build()
                .expect("reqwest client"),
        }
    }

    pub async fn install_selected(&mut self) -> Result<(), NexusError> {
        self.state.busy = true;
        self.state.status_line = format!("Installing {}…", self.state.selected_version);
        download::ensure_version(&self.client, &self.install, &self.state.selected_version).await?;
        if let Some(v) = self
            .state
            .versions
            .iter_mut()
            .find(|v| v.id == self.state.selected_version)
        {
            v.ready = true;
        }
        self.state.busy = false;
        self.state.status_line = format!("{} ready — Play for smooth session", self.state.selected_version);
        Ok(())
    }

    pub async fn play(&mut self, profile: PerformanceProfile) -> Result<LaunchReport, NexusError> {
        self.state.busy = true;
        self.state.status_line = "Launching smooth session…".into();
        let report = launch::launch(
            &self.install,
            &mut self.engine,
            LaunchRequest {
                version_id: self.state.selected_version.clone(),
                instance_dir: self
                    .install
                    .root
                    .join("instances")
                    .join(&self.state.selected_version),
                performance: profile,
            },
        )
        .await?;
        self.state.last_launch_ms = Some(report.handoff_ms);
        self.state.busy = false;
        self.state.status_line = format!(
            "Live · {} ms · RD {} · scale {}%",
            report.handoff_ms,
            report.smooth.render_distance,
            report.smooth.resolution_scale_percent
        );
        Ok(report)
    }

    pub fn stop_game(&mut self) -> Result<(), NexusError> {
        self.engine.stop()?;
        self.state.status_line = "Stopped".into();
        Ok(())
    }
}
