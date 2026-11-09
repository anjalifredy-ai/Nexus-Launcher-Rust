# Nexus Launcher (Rust)

Premium Android launcher + Horizon-style Rust path. Focus: **smooth play**, fast prepared launch, clean version matrix.

> Not an official Minecraft product.

## Smooth play (current work)

- `nexus-engine` — session supervisor, **Performance / Balanced / Quality**, adaptive quality step-down
- Frame budget ~16.6 ms (60 FPS), VSync on, Vulkan preferred
- Full install path: version JSON + client jar + **libraries** + **asset index/objects** (parallel, SHA1)
- Launch: ready-only, no network on Play

## Versions

`1.21.1`–`1.21.11` · `26.1` · `26.2` · `26.3`

## Content

Skins · Resource packs · Shaders — **no mod browser**

## Workspace

```
nexus-versions | nexus-download | nexus-content
nexus-engine   | nexus-launch   | nexus-ui
nexus-core     | nexus-android
```

## Status

- [x] Version matrix + dual backend tags
- [x] Parallel lib/asset install + ready marker
- [x] SmoothConfig + adaptive quality
- [x] Launch handoff + engine session API
- [ ] Link real Horizon-native / multi-version binaries
- [ ] Android premium UI shell on device
- [ ] Measured 60 FPS session on hardware

```bash
cargo test -p nexus-engine
cargo test -p nexus-versions
cargo check --workspace
```
