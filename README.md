# Nexus Launcher (Rust)

Premium Android-oriented launcher + Horizon-style Rust path.

## Quick start (live frame loop)

```bash
cargo run -p nexus-start
cargo run -p nexus-start -- --version 26.3 --profile performance --seconds 3
cargo run -p nexus-start -- --version 1.21.1 --profile balanced --seconds 2
```

Verified: **120 frames / 2s = 60.0 avg FPS** with VSync pacing, zero over-budget on Performance.

## What runs now

- Session start + dual backend (`HorizonNative` / `MultiVersionRuntime`)
- Live frame loop with FPS cap + adaptive quality
- Smooth profiles: Performance / Balanced / Quality

## Versions

`1.21.1`–`1.21.11` · `26.1` · `26.2` · `26.3`

## Next

- GPU world submit (wgpu / Vulkan)
- Android APK shell
- Full Mojang install on device

> Not an official Minecraft product.
