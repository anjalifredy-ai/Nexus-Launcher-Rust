//! Premium launcher UI model (Rust).
//!
//! Android shell renders this state. No mod browser by product decision.

use nexus_content::{ResourcePackEntry, ShaderPackEntry, SkinSlot};
use nexus_versions::{SUPPORTED_VERSIONS, VersionInfo};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Screen {
    #[default]
    Home,
    Versions,
    Skins,
    ResourcePacks,
    Shaders,
    Settings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PremiumTheme {
    pub name: &'static str,
    /// Deep charcoal background.
    pub bg: u32,
    pub surface: u32,
    pub accent: u32,
    pub text: u32,
    pub muted: u32,
}

impl Default for PremiumTheme {
    fn default() -> Self {
        Self {
            name: "Nexus Midnight",
            bg: 0x0B0D12,
            surface: 0x141821,
            accent: 0x6C5CE7,
            text: 0xF4F6FB,
            muted: 0x8B93A7,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherState {
    pub screen: Screen,
    pub theme: PremiumTheme,
    pub selected_version: String,
    pub versions: Vec<VersionInfo>,
    pub skins: Vec<SkinSlot>,
    pub resource_packs: Vec<ResourcePackEntry>,
    pub shader_packs: Vec<ShaderPackEntry>,
    /// Last launch handoff time (ms) for HUD / debug.
    pub last_launch_ms: Option<u128>,
    pub busy: bool,
    pub status_line: String,
}

impl Default for LauncherState {
    fn default() -> Self {
        let versions: Vec<_> = SUPPORTED_VERSIONS
            .iter()
            .filter_map(|id| VersionInfo::for_id(id))
            .collect();
        Self {
            screen: Screen::Home,
            theme: PremiumTheme::default(),
            selected_version: "26.3".into(),
            versions,
            skins: vec![],
            resource_packs: vec![],
            shader_packs: vec![],
            last_launch_ms: None,
            busy: false,
            status_line: "Ready".into(),
        }
    }
}

/// Navigation destinations exposed in the premium chrome.
/// Explicitly excludes Mods.
pub fn primary_nav() -> &'static [(Screen, &'static str)] {
    &[
        (Screen::Home, "Home"),
        (Screen::Versions, "Versions"),
        (Screen::Skins, "Skins"),
        (Screen::ResourcePacks, "Resource Packs"),
        (Screen::Shaders, "Shaders"),
        (Screen::Settings, "Settings"),
    ]
}
