//! Nexus core facade — wires versions, download, content, launch, UI state.

pub use nexus_content as content;
pub use nexus_download as download;
pub use nexus_launch as launch;
pub use nexus_ui as ui;
pub use nexus_versions as versions;

use download::InstallRoot;
use launch::{LaunchRequest, LaunchReport, PerformanceProfile};
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
}

pub struct NexusApp {
    pub install: InstallRoot,
    pub state: LauncherState,
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
            client: reqwest::Client::new(),
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
        self.state.status_line = format!("{} ready", self.state.selected_version);
        Ok(())
    }

    pub async fn play(&mut self, profile: PerformanceProfile) -> Result<LaunchReport, NexusError> {
        self.state.busy = true;
        self.state.status_line = "Launching…".into();
        let report = launch::launch(
            &self.install,
            LaunchRequest {
                version_id: self.state.selected_version.clone(),
                instance_dir: self.install.root.join("instances").join(&self.state.selected_version),
                performance: profile,
            },
        )
        .await?;
        self.state.last_launch_ms = Some(report.handoff_ms);
        self.state.busy = false;
        self.state.status_line = format!("Launched in {} ms", report.handoff_ms);
        Ok(report)
    }
}
