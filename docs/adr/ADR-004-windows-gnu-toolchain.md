# ADR-004: Windows GNU toolchain (MinGW-w64)

**Status:** Accepted  
**Date:** 2026-06-27

## Decision

The project targets `x86_64-pc-windows-gnu` and links against MinGW-w64 GCC instead of the MSVC linker.

## Context

The MSVC toolchain requires Visual Studio C++ Build Tools. These are not always present, and their licensing adds friction for contributors. `reqwest` with `rustls` (our HTTP client) avoids the native-tls C++ dependency entirely.

MinGW-w64 via MSYS2 provides a complete, freely installable GCC toolchain that produces native Windows binaries without requiring a Visual Studio installation.

## Consequences

- Developers must install MSYS2 and MinGW-w64 (see README).
- `C:\msys64\mingw64\bin` must be in the system PATH so GCC can locate its own runtime DLLs (`libmpfr-6.dll` etc.). Builds fail silently with `ld returned 53` if this path is missing.
- The `.cargo/config.toml` linker path is machine-specific and not committed to the repository. It must specify `linker`, `ar`, and `dlltool` explicitly.
- `rust-toolchain.toml` pins the channel to `stable-x86_64-pc-windows-gnu` so `rustup` selects the GNU toolchain automatically.
- The produced `.exe` has no dependency on MSVC runtime DLLs.
