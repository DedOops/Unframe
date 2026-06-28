# Unframe

> Information beyond the interface.

A document internet client for Windows x64. Fetches HTML without executing JavaScript, converts it to an internal Information Document Model, and renders it with a native GUI.

No Chromium. No WebView. No third-party scripts.

---

## Building from source

### Requirements

- Windows x64
- [Rust](https://rustup.rs/) (stable, GNU target)
- [MSYS2](https://www.msys2.org/) with MinGW-w64 GCC

### 1. Install Rust

```powershell
winget install Rustlang.Rustup
```

### 2. Install the GNU toolchain target

```powershell
rustup target add x86_64-pc-windows-gnu
rustup toolchain install stable-x86_64-pc-windows-gnu
```

### 3. Install MSYS2 and MinGW-w64

```powershell
winget install MSYS2.MSYS2
```

Then open the MSYS2 terminal and run:

```bash
pacman -S mingw-w64-x86_64-gcc
```

### 4. Add MinGW to your PATH

GCC requires its own DLLs at runtime. Add the MinGW bin directory to your user PATH:

```powershell
$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
[Environment]::SetEnvironmentVariable("PATH", "C:\msys64\mingw64\bin;C:\msys64\usr\bin;" + $userPath, "User")
```

Restart your terminal after this step.

### 5. Configure the linker

Create `.cargo/config.toml` in the project root with your MSYS2 path:

```toml
[build]
target = "x86_64-pc-windows-gnu"

[target.x86_64-pc-windows-gnu]
linker = "C:\\msys64\\mingw64\\bin\\gcc.exe"
ar     = "C:\\msys64\\mingw64\\bin\\ar.exe"
rustflags = ["-C", "dlltool=C:\\msys64\\mingw64\\bin\\dlltool.exe"]
```

Adjust the paths if MSYS2 is installed elsewhere.

### 6. Build and run

```powershell
cargo test --workspace
cargo build --release
./target/x86_64-pc-windows-gnu/release/unframe.exe
```

---

## Architecture

```
URL → Network Gateway → HTML Parser → Generic Adapter → Document Model → Native Renderer
```

Each layer is a separate crate:

| Crate | Purpose |
|-------|---------|
| `unframe-model` | Information Document Model + validation |
| `unframe-network` | HTTP gateway (reqwest + rustls) |
| `unframe-parser` | HTML parser (scraper / html5ever) |
| `unframe-adapter-html` | Generic HTML adapter |
| `unframe-renderer` | egui block widgets |
| `app` | Application shell, state machine, UI layout |

---

## Design principles

- No JavaScript execution
- No WebView or browser engine
- All network requests go through a single gateway
- The renderer only accepts the internal document model
- Every document has a provenance audit trail

See [docs/adr/](docs/adr/) for architecture decisions.

---

## Status

Pre-alpha. The core pipeline works. Expect rough edges.

## License

MIT — see [LICENSE](LICENSE).
