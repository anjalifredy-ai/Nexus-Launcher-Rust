# Nexus Launcher (Rust)

Premium Android launcher + **lightweight Rust engine path** for Minecraft: Java Edition.

> Not an official Minecraft product. Not affiliated with Mojang, Microsoft, or upstream Horizon.

## Phase 1 priorities (now)

1. **Versions that must work properly** (matrix below)
2. **Aggressive optimization** — FPS / RAM / startup at a different level than typical mobile Java launchers
3. **Premium launcher UI** — polished, modern, fast

Features (accounts extras, mod browser polish, etc.) come **after** these three.

## Target version matrix

| Line | Versions |
|------|----------|
| **1.21.x** | 1.21.1 · 1.21.2 · 1.21.3 · 1.21.4 · 1.21.5 · 1.21.6 · 1.21.7 · 1.21.8 · 1.21.9 · 1.21.10 · 1.21.11 |
| **26.x** | 26.1 · 26.2 · 26.3 |

Every listed version must install, verify, and launch cleanly on supported devices.

### Architecture note (honest)

Upstream **Horizon** is a full Rust rewrite aimed at **26.3 only** (no JVM).  
Supporting **all** of 1.21.1–1.21.11 + 26.1–26.3 as pure separate Rust rewrites is multi-year work per version.

**Nexus approach:**

| Path | Versions | What it is |
|------|----------|------------|
| **Horizon-native (Rust client)** | 26.3 first, then 26.2 / 26.1 as feasible | No JVM — max FPS / lowest overhead |
| **Multi-version runtime path** | Full 1.21.1–1.21.11 (+ 26.x fallback) | Official jars + mobile-tuned runtime / renderer bridge, Rust-owned download & launch |

One launcher UI. Version picker chooses the right backend. Optimization work applies to both paths (render scale, chunk/meshing policy, memory caps, thermal).

## Optimization bar

- **FPS:** high stable frametimes; dynamic quality (render distance, entities, particles) under thermal / memory pressure
- **GPU:** Vulkan first (wgpu where native); GLES fallback only when required
- **RAM:** hard budgets by device class; aggressive asset / chunk residency limits
- **Startup:** cached manifests, parallel download + SHA1 verify, resume, minimal main-thread work
- **Binary:** keep APK and native libs lean; strip unused paths per ABI

## Premium UI (launcher)

- Dark premium theme, clear hierarchy, smooth motion (no jank)
- Home: play / last version / quick status
- Versions: matrix install state, progress, repair
- Instances: isolated game dirs
- Controls: editable touch schemes (later polish OK; base must feel premium)
- Settings: renderer, performance profiles (Performance / Balanced / Quality)

## Layout

```
Nexus-Launcher-Rust/
├── android/                 # Premium UI (Kotlin)
├── engine/
│   ├── horizon_native/      # 26.x Rust client path
│   └── multi_version/       # Official-jar runtime path (1.21.x …)
├── native/                  # Shared Rust: download, verify, launch, wgpu glue
├── download/                # Mojang manifests, libraries, assets
├── docs/
│   └── ARCHITECTURE.md
└── README.md
```

## Status

- [x] Repo + engine direction
- [x] Version matrix + optimization + UI locked as Phase 1
- [ ] Scaffold Android shell + Rust workspace
- [ ] Download / verify pipeline (all matrix versions)
- [ ] 26.3 Horizon-native bring-up track
- [ ] 1.21.x multi-version launch track
- [ ] Performance profiles + premium UI screens

## License

TBD. Game assets/data: Minecraft EULA — user must own Java Edition.
