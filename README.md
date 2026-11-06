# Nexus Launcher (Rust)

Custom **Minecraft Java Edition** launcher for **Android**, built on our own engine — not a PojavLauncher fork.

> Engine name / core stack: *TBD* (will be decided next).

## Goals

- Native Android launcher UI
- Custom mobile-optimized Java runtime path
- Official Mojang download & version system (manifests, libraries, assets, SHA1 verify)
- First-class loader support as engine modules: Fabric, Quilt, Forge, NeoForge, OptiFine
- Graphics translation layer suitable for mobile GPUs (OpenGL ES / ANGLE / fallbacks)
- Rust-side performance-critical components where it makes sense

## Planned layout (high level)

```
Nexus-Launcher-Rust/
├── android/          # Android app (UI + JNI bridge)
├── engine/           # Core engine (download, launch, loaders, runtime)
├── native/           # Rust / C++ native modules
├── docs/             # Architecture & design notes
└── README.md
```

Exact module names will lock in once the engine name is chosen.

## Status

**Scaffold only.** Repo created. Architecture + engine name coming next.

## License

TBD
