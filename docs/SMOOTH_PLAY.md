# Smooth play design

## Target feel

- Stable **60 FPS** frametimes (not noisy 40–90 swings)
- Vulkan preferred
- Adaptive quality when a frame exceeds budget (~16.6 ms)
- Low render distance defaults on Performance profile for weak GPUs

## Profiles

| Profile | RD | Res scale | Particles | Use |
|---------|----|-----------|-----------|-----|
| Performance | 6 | 75% | 40% | low-end / heat |
| Balanced | 8 | 90% | 70% | default |
| Quality | 12 | 100% | 100% | flagship |

## Pipeline

1. Install version (libs + assets + jar) until `.nexus_ready`
2. Play → `SmoothConfig` → `EngineSupervisor::start`
3. Over-budget frames → `degrade_one_step` (RD / scale / particles)

## Not yet linked

Actual world simulation + GPU submit still need Horizon-native / multi-version binaries. This layer is the **control plane** so those backends run under strict mobile budgets from day one.
