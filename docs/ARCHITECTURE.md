# Nexus Architecture (Phase 1)

## Product goals

1. Versions work: `1.21.1` … `1.21.11`, `26.1`, `26.2`, `26.3`
2. Extreme mobile optimization (FPS, RAM, startup, thermals)
3. Premium launcher UI

## Dual backend

```
┌─────────────────────────────────────┐
│           Nexus Launcher UI         │
│     (Kotlin · premium · Android)    │
└─────────────────┬───────────────────┘
                  │ version + instance
        ┌─────────┴─────────┐
        ▼                   ▼
┌──────────────────┐ ┌─────────────────────┐
│ horizon_native   │ │ multi_version       │
│ Rust client      │ │ official jars +     │
│ 26.3 → 26.x      │ │ mobile runtime      │
│ no JVM           │ │ 1.21.1–1.21.11      │
└────────┬─────────┘ └──────────┬──────────┘
         │                      │
         └──────────┬───────────┘
                    ▼
         shared: download · SHA1 · storage
         shared: input · lifecycle · perf profiles
```

## Why dual path

- Pure Horizon-style rewrite = best FPS / lightest process, but **one game version per huge effort** (upstream Horizon ≈ 26.3 only).
- Official multi-version support needs the real client jars + a mobile-capable runtime/renderer bridge.
- One UI, two backends, shared download and performance policy.

## Optimization layers

1. **Device class** — detect RAM / GPU; pick default profile
2. **Performance profiles** — Performance / Balanced / Quality
3. **Dynamic scaling** — render distance, entity distance, particles under pressure
4. **Graphics** — Vulkan preferred; GLES fallback
5. **I/O** — parallel downloads, resume, hash verify, aggressive cache

## Out of scope for Phase 1

- Fancy feature packs, mod store polish, social, themes marketplace
- Full parity of every Horizon desktop extra on day one
