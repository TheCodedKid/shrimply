# AppKit renderer and GTK interaction parity

Goal: shared Slang rendering works on Metal while remaining functional on CUDA; AppKit preview and timeline interactions match GTK.

Hardware-specific effects fail explicitly when their required GPU feature or shader is unavailable; no software fallback is permitted.

Implementation owner: primary agent. Other agents review only; no implementation delegation.

The Shrimply MCP is unavailable in this session. Existing project files are read through application APIs and are not edited directly.
