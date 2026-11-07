//! Android JNI / NativeActivity entry (stub).
//!
//! Wire `android-activity` + egui/wgpu shell in the next milestone.
//! Premium UI state comes from `nexus_ui::LauncherState`.

use nexus_core::NexusApp;

/// Placeholder so the cdylib builds once Android NDK targets are configured.
#[no_mangle]
pub extern "C" fn nexus_android_bootstrap(data_dir: *const i8) {
    let _ = data_dir;
    // Real path: parse data_dir, construct NexusApp, run UI event loop.
    let _app = NexusApp::new("/data/data/dev.nexus.launcher/files");
}
