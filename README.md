# Nexus Launcher (Rust)

Horizon-style Rust path + unlimited FPS + **software scene renderer**.

## Run

```bash
cargo run -p nexus-start -- --version 26.3 --profile performance --seconds 1
```

Writes `preview.ppm` under data dir — sky, sun, terrain blocks, crosshair.

## Now

- Unlimited FPS
- Live frame loop + adaptive quality
- Software RGB framebuffer (scale-aware)
- Dual backend for 1.21.x / 26.x

## Next

- wgpu / Vulkan surface
- Android APK shell
- Mojang install on device

> Not an official Minecraft product.
