# AppKit renderer and GTK interaction parity

Goal: shared Slang rendering works on Metal while remaining functional on CUDA; AppKit preview and timeline interactions match GTK.

Hardware-specific effects fail explicitly when their required GPU feature or shader is unavailable; no software fallback is permitted.

Implementation owner: primary agent. Other agents review only; no implementation delegation.

## Current priorities

- [ ] Move reusable SAM2 and transparent-fill planning, cache ownership, and lifecycle orchestration into `video-core`; keep AppKit/Metal and CUDA as thin upload/dispatch bridges over the shared Slang kernels.
- [ ] Refactor the shared OBJ Slang ray pipeline into a compute-raytracing form usable by CUDA and Metal, move scene preparation into `-core`, and connect the Metal BLAS/TLAS bridge; AppKit must fail explicitly when the device lacks ray tracing or the shared Metal shader is unavailable.
- [ ] Move remaining reusable source/effect orchestration into `-core`; keep Metal and CUDA crates as resource/execution bridges.
- [ ] Connect tracked-camera sampling to native rendering and preview geometry.

## Preview

- [ ] Fix any remaining AppKit interaction gaps reported during use through the shared preview interaction core.

## AppKit

- [ ] Rebuild Settings as a compact native macOS preferences window following Apple HIG spacing and grouping; label the Apple-silicon memory limit as a unified renderer memory budget and keep persistence in preferences-core.

## Timeline

- [ ] Complete screen recording, transcription, silence-removal and speech-generation workflows through shared timeline operations; microphone recording is owned by timeline-core and connected to AppKit.

The Shrimply MCP is unavailable in this session. Existing project files are read through application APIs and are not edited directly.
