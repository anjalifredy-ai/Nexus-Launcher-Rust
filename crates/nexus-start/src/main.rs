//! Nexus Launcher — start entry (session + smooth config).
//!
//!   cargo run -p nexus-start
//!   cargo run -p nexus-start -- --version 26.3 --profile performance

use std::env;
use std::path::PathBuf;
use std::time::Instant;

use nexus_engine::{EngineSupervisor, PerformanceProfile, SessionRequest, SmoothConfig};
use nexus_versions::{BackendKind, VersionInfo, SUPPORTED_VERSIONS};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut version = "26.3".to_string();
    let mut profile = PerformanceProfile::Balanced;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--version" | "-v" => {
                i += 1;
                if i < args.len() {
                    version = args[i].clone();
                }
            }
            "--profile" | "-p" => {
                i += 1;
                if i < args.len() {
                    profile = match args[i].to_lowercase().as_str() {
                        "performance" | "perf" => PerformanceProfile::Performance,
                        "quality" => PerformanceProfile::Quality,
                        _ => PerformanceProfile::Balanced,
                    };
                }
            }
            "--help" | "-h" => {
                println!(
                    "Nexus Launcher\n\n\
                     Versions: {}\n\n\
                     USAGE:\n  nexus-start [--version ID] [--profile performance|balanced|quality]\n",
                    SUPPORTED_VERSIONS.join(", ")
                );
                return;
            }
            other => {
                eprintln!("unknown arg: {other}");
                std::process::exit(2);
            }
        }
        i += 1;
    }

    let info = match VersionInfo::for_id(&version) {
        Some(v) => v,
        None => {
            eprintln!("unsupported version: {version}");
            eprintln!("supported: {}", SUPPORTED_VERSIONS.join(", "));
            std::process::exit(1);
        }
    };

    let data = env::var("NEXUS_DATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| env::temp_dir().join("nexus-launcher"));
    let game_dir = data.join("versions").join(&version);
    let instance_dir = data.join("instances").join(&version);
    let _ = std::fs::create_dir_all(&game_dir);
    let _ = std::fs::create_dir_all(&instance_dir);

    let ready = game_dir.join(".nexus_ready");
    if !ready.exists() {
        let _ = std::fs::write(&ready, b"local-dev");
    }

    let smooth = SmoothConfig::from_profile(profile);
    let start = Instant::now();

    let mut engine = EngineSupervisor::default();
    let handle = match engine.start(SessionRequest {
        version_id: version.clone(),
        backend: info.backend,
        smooth: smooth.clone(),
        instance_dir,
        game_dir,
    }) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("launch failed: {e}");
            std::process::exit(1);
        }
    };

    let ms = start.elapsed().as_millis();
    let backend_name = match handle.backend {
        BackendKind::HorizonNative => "HorizonNative",
        BackendKind::MultiVersionRuntime => "MultiVersionRuntime",
    };

    println!("══════════════════════════════════════");
    println!("  NEXUS SESSION STARTED");
    println!("══════════════════════════════════════");
    println!("  version     : {}", handle.version_id);
    println!("  backend     : {backend_name}");
    println!("  profile     : {:?}", handle.smooth.profile);
    println!("  handoff     : {ms} ms");
    println!("  fps_cap     : {}", handle.smooth.fps_cap);
    println!("  render dist : {}", handle.smooth.render_distance);
    println!("  res scale   : {}%", handle.smooth.resolution_scale_percent);
    println!("  vsync       : {}", handle.smooth.vsync);
    println!("  vulkan      : {}", handle.smooth.prefer_vulkan);
    println!("  adaptive    : {}", handle.smooth.adaptive_quality);
    println!("  data        : {}", data.display());
    println!("══════════════════════════════════════");
    println!("  status: LIVE (engine binary link = next milestone)");
    println!("══════════════════════════════════════");
}
