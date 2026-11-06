# Nexus Launcher (Rust)

Lightweight Android launcher + client path for **Minecraft: Java Edition**, built around a **Horizon-style Rust engine** — native, no JVM on device, focused on low memory and fast startup.

> Not an official Minecraft product. Not affiliated with Mojang, Microsoft, or the upstream Horizon project.

## Engine direction

| Choice | Decision |
|--------|----------|
| **Engine style** | Horizon Rust (Minecraft Java 26.3 rewritten in Rust) |
| **Runtime** | Native Rust — **no Java / JVM on the phone** |
| **Graphics** | Vulkan primary (wgpu), GLES fallback where needed |
| **Goal** | Ultra-lightweight: small APK, low RAM, quick launch |
| **Mods** | Native / sandboxed engine mods (e.g. `.hmod`-style) — not Fabric/Forge Java jars |
| **Assets** | Official Mojang endpoints + hash verify; user must own Java Edition |

This is **not** a Pojav-style “run the official JAR on Android” launcher.  
The game client itself is a **Rust reimplementation path** inspired by [Horizon](https://playhorizon.dev/) — same worlds/rules target, different binary.

## Why this approach

- No heavy mobile JRE / LWJGL natives stack
- Smaller process, better control over memory and GPU
- Touch, lifecycle, and storage designed for Android from day one
- Launcher UI + download/prepare pipeline + native engine in one product: **Nexus**

## Planned layout

```
Nexus-Launcher-Rust/
├── android/              # Android app (UI, accounts, instances, controls)
├── engine/               # Horizon-style core (world, net, render glue)
├── native/               # Rust crates (wgpu/Vulkan, input, prepare)
├── download/             # Manifests, assets, hash verify, resume
├── docs/                 # Architecture notes
└── README.md
```

Exact crate names lock in as we scaffold.

## Status

- [x] Repo created
- [x] Engine direction: **Horizon Rust, lightweight**
- [ ] Scaffold crates + Android shell
- [ ] Resource download / prepare pipeline
- [ ] First Vulkan surface + touch input
- [ ] Singleplayer bring-up target

## References (upstream inspiration only)

- Horizon: https://playhorizon.dev/
- Horizon docs: https://playhorizon.dev/docs/
- Unofficial Android ports exist in the community; **Nexus is our own launcher + integration**, not a rebrand of those repos.

## License

TBD (engine code we write; respect Minecraft EULA for game assets/data).
