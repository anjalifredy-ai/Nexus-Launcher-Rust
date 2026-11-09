# Nexus Launcher (Rust)

Real playable core direction: **client + integrated GameServer** (MC-style singleplayer).

```bash
cargo run -p nexus-start -- --version 26.3 --profile performance --seconds 2
```

## Live game core

| System | Status |
|--------|--------|
| GameServer 20 TPS | yes |
| Authoritative world | yes |
| Players join/move | yes |
| Block place / break | yes |
| Culled mesh ~38k tris | yes |
| Client render loop | software |
| Full Mojang protocol | **not yet** |
| Full Java MC parity | **long road** |

Honest scope: this is a **Rust voxel game core** aimed at Horizon-class mobile play, not a finished decompile of Java Edition.

## Crates

`server` `world` `mesh` `render` `input` `engine` `android` `start`

> Not an official Minecraft product.
