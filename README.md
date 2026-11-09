# Nexus Launcher (Rust)

Premium Android-oriented launcher + Horizon-style Rust path. **Session start is live.**

## Quick start

```bash
cargo run -p nexus-start
cargo run -p nexus-start -- --version 26.3 --profile performance
cargo run -p nexus-start -- --version 1.21.1 --profile balanced
```

Example output: session boots with backend (`HorizonNative` / `MultiVersionRuntime`), 60 FPS budget, render distance, resolution scale, Vulkan preferred.

## Versions

`1.21.1`–`1.21.11` · `26.1` · `26.2` · `26.3`

## Smooth profiles

| Profile | RD | Scale |
|---------|----|-------|
| Performance | 6 | 75% |
| Balanced | 8 | 90% |
| Quality | 12 | 100% |

## Status

- [x] `nexus-start` boots session
- [x] Version matrix + dual backend
- [x] SmoothConfig + adaptive quality API
- [x] Download pipeline code (libs/assets)
- [ ] Full Mojang install on device
- [ ] Real world/GPU engine binary
- [ ] Android APK UI

> Not an official Minecraft product.
