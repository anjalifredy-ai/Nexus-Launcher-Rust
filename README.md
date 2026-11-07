# Nexus Launcher (Rust)

Premium, ultra-optimized Android launcher + Horizon-style **Rust** engine path for Minecraft: Java Edition.

> Not an official Minecraft product. Not affiliated with Mojang, Microsoft, or upstream Horizon.

## Phase 1 product lock

| Pillar | Spec |
|--------|------|
| **Versions** | `1.21.1`–`1.21.11`, `26.1`, `26.2`, `26.3` |
| **Language** | Rust (workspace) |
| **Launch goal** | **&lt; 5s** into game when version is already prepared (no network on Play) |
| **UI** | Premium dark theme — Home, Versions, Skins, Resource Packs, Shaders, Settings |
| **Content** | Skins · Resource packs · Shader browsing |
| **Mods browser** | **No** (by design) |
| **Optimization** | Performance / Balanced / Quality profiles, Vulkan-first path |

## Backend split

| Versions | Backend |
|----------|---------|
| `26.x` | `HorizonNative` — Rust client path (no JVM), max FPS |
| `1.21.x` | `MultiVersionRuntime` — official jars + mobile runtime bridge |

## Workspace

```
crates/
  nexus-versions   # matrix + Mojang manifest types
  nexus-download   # parallel install, SHA1, ready markers
  nexus-content    # skins / resource packs / shaders (no mods)
  nexus-launch     # <5s fast path contract + backend handoff
  nexus-ui         # premium UI state + theme + nav
  nexus-core       # app facade
  nexus-android    # Android cdylib entry (stub → NativeActivity)
```

## Fast launch contract

1. **Install once** — `ensure_version` downloads + verifies, writes `.nexus_ready`
2. **Play** — if ready, **zero network**; hand off to engine entry
3. Target: mid-range device, warm storage, prepared version → **handoff &lt; 5s**

Cold install of assets is separate and can take longer; the 5s goal is **Play after prepare**.

## Build (host checks)

```bash
cargo test -p nexus-versions
cargo check --workspace
```

Android NDK / `cargo-apk` wiring lands with the UI shell milestone.

## Status

- [x] Repo + architecture
- [x] Rust workspace scaffold
- [x] Version matrix API
- [x] Download / ready-marker pipeline (client jar + version json)
- [x] Content: skins, resource packs, shaders (no mods)
- [x] Launch fast-path API
- [x] Premium UI state model
- [ ] Full libraries + assets parallel download
- [ ] Horizon-native engine binary link (26.3)
- [ ] Multi-version runtime link (1.21.x)
- [ ] Android NativeActivity + premium rendered UI
- [ ] Measured &lt;5s launch on device

## License

MIT for Nexus code we author. Game assets/data: Minecraft EULA — own Java Edition.
