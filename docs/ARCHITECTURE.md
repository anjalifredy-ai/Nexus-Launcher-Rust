# Nexus Architecture

## Goals

1. All matrix versions install + launch cleanly
2. Ultra optimization (FPS, RAM, thermals)
3. Premium UI
4. Skins, resource packs, shader browse — **no mod browser**
5. Play ≤ 5s when prepared (Rust fast path)

## Crates

```
UI (nexus-ui state) ──► nexus-core ──► launch / download / content
                              │
              ┌───────────────┴───────────────┐
              ▼                               ▼
     HorizonNative (26.x)          MultiVersionRuntime (1.21.x)
```

## 5 second launch

| Phase | Budget |
|-------|--------|
| Readiness check | &lt; 50 ms |
| Instance paths + profile | &lt; 100 ms |
| Engine process/handoff | rest of budget |
| Network on Play | **forbidden** if ready |

Install/prepare is offline from Play. User installs from Versions screen first.

## Content policy

- Skins: PNG drop-in + selector
- Resource packs: zip/dir browse + enable order (next)
- Shaders: pack browse + enable
- Mods: not exposed in nav or APIs

## Honest scope

Full Horizon-class game logic for every version is multi-milestone. This repo owns launcher, download, content, launch contract, and backend slots. Engine binaries fill those slots incrementally — **26.3 native first**.
