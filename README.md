# Nexus Launcher (Rust)

## Run

```bash
cargo run -p nexus-start -- --version 26.3 --profile performance --seconds 1
```

## Stack now

- **Unlimited FPS** frame loop
- **Software renderer** (row-fill sky/ground, heightmap columns, camera orbit)
- **`nexus-world`** — 16×16 chunks, grass/dirt/stone columns (9 chunks prefetched)
- Preview: `preview.ppm`

## Versions

`1.21.1`–`1.21.11` · `26.1` · `26.2` · `26.3`

## Next

- Mesh from world chunks
- wgpu/Vulkan
- Android shell

> Not an official Minecraft product.
