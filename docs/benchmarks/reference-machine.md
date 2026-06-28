# Reference machine — pre-alpha baseline

This document records the hardware and software configuration used as the reference environment for pre-alpha performance measurements.

## Hardware

| Component | Value |
|-----------|-------|
| CPU | AMD Ryzen 7 9800X3D (8 cores, 16 threads) |
| RAM | 64 GB |
| Storage | NVMe SSD |
| OS | Windows 11 Pro Insider Preview (build 26220) |

## Software

| Component | Version |
|-----------|---------|
| Rust | 1.96.0 (stable, 2026-05-25) |
| Cargo | 1.96.0 |
| Target | x86_64-pc-windows-gnu |
| Linker | GCC via MSYS2 MinGW-w64 |
| Build profile | debug (pre-alpha), release (M0 baseline) |

## Notes

Performance targets from `tech.md` (pre-alpha):

| Metric | Target |
|--------|--------|
| Idle memory | ≤ 150 MB |
| One text document | ≤ 200 MB |
| CPU after load | ~0% |
| Background requests | 0 |
| Scripts executed | 0 |
| Cold start | ≤ 3 s |
| Article parse (excl. network) | ≤ 3 s |

Measured values will be added after the release build is verified.
