# AppKit renderer and GTK interaction parity

Goal: shared Slang rendering works on Metal while remaining functional on CUDA; AppKit preview and timeline interactions match GTK.

Hardware-specific effects fail explicitly when their required GPU feature or shader is unavailable; no software fallback is permitted.

Implementation owner: primary agent. Other agents review only; no implementation delegation.

## Current priorities

- [ ] Complete shared OBJ Slang rendering parity for environments, grounds, outlines and transmission on Metal while preserving CUDA; unsupported modes fail explicitly.
- [ ] Move remaining reusable source/effect orchestration into `-core`; keep Metal and CUDA crates as resource/execution bridges.
- [ ] Replace render-3d-metal's ad hoc reflected-binding source formatting with reusable Slang/Metal build support; the crate build script should only declare its module, entry point and required resources.
- [ ] Connect tracked-camera sampling to native rendering and preview geometry.

## Preview

- [ ] Fix any remaining AppKit interaction gaps reported during use through the shared preview interaction core.

## Settings

- [ ] Match GTK's Integrations preferences on AppKit: manage multiple compute servers, show each server's version in the server list, show the selected server's full feature list and device selector, and report whether its URL resolves to a compatible server; keep the server model and validation in shared core logic.

## Project opening

- [ ] Match GTK project-lock takeover on AppKit: when another editor process owns a project's lock, let the user terminate that process and retry opening the project through shared lock-management logic.

## Timeline

- [ ] Complete screen recording, transcription, silence-removal and speech-generation workflows through shared timeline operations; microphone recording is owned by timeline-core and connected to AppKit.

The Shrimply MCP is unavailable in this session. Existing project files are read through application APIs and are not edited directly.
