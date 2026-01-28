<div align="center">
  <a href="https://github.com/immichFrame/ImmichFrame_Desktop">
    <img src="src-tauri/icons/icon.png" alt="Logo" width="200" height="200">
  </a>

  <h3 align="center">ImmichFrame Desktop</h3>

  <p align="center">
    Desktop application that runs <a href="https://github.com/immichFrame/ImmichFrame">ImmichFrame</a> fullscreen.
    <br />
    <br />
    <a href="https://immichframe.dev">Documentation</a>
  <p>
</div>


## Autostart (Linux)

If you want ImmichFrame to start automatically after login, create an autostart desktop entry:

- Create `~/.config/autostart/immich.desktop`
- Example:

```ini
[Desktop Entry]
Type=Application
Name=ImmichFrame
Terminal=false

# Notes:
# - On some ARM/Mali Wayland setups, forcing GLES helps GTK/WebKit enable acceleration.
# - Optional watchdog/diagnostics env vars (off by default):
#   - IMMICHFRAME_DEBUG_OVERLAY=1            Shows a small on-screen memory overlay (click to hide)
#   - IMMICHFRAME_AUTO_RELOAD_MINUTES=60     Periodically hard-reloads the webview
#   - IMMICHFRAME_AUTO_RESTART_MINUTES=180   Restarts the app after N minutes (most effective on low-RAM boards)
Exec=sh -lc 'sleep 5; cd "$HOME/ImmichFrame"; export GDK_BACKEND=wayland; export GDK_GL=gles; export IMMICHFRAME_DEBUG_OVERLAY=1; export IMMICHFRAME_AUTO_RELOAD_MINUTES=60; exec immichframe >>"$HOME/.cache/immichframe.log" 2>&1'

# Alternative (stronger workaround): replace AUTO_RELOAD with AUTO_RESTART, e.g.
# Exec=sh -lc 'sleep 5; cd "$HOME/ImmichFrame"; export GDK_BACKEND=wayland; export GDK_GL=gles; export IMMICHFRAME_DEBUG_OVERLAY=1; export IMMICHFRAME_AUTO_RESTART_MINUTES=180; exec immichframe >>"$HOME/.cache/immichframe.log" 2>&1'
```

ImmichFrame stores the configured URL in `~/.config/immichFrame/Settings.txt` on Linux.
If ImmichFrame Docker is running on the same machine with port 8080, the URL should be `http://localhost:8080`.

## ARM64 (aarch64) builds via self-hosted runner

GitHub-hosted ARM64 runners are not available, so this repo supports building the Linux ARM64 `.deb` on a self-hosted runner.

High level steps:

1. Provision an Ubuntu aarch64 machine (e.g. Khadas VIM3) and install the GitHub Actions runner.
2. Register the runner for your fork/repo with labels: `self-hosted`, `linux`, `arm64`.
3. Run the workflow `.github/workflows/build-and-push.yml` manually via `workflow_dispatch` and pick `build=arm64`.
  - Provide an existing tag (e.g. `v1.0.2`) so the workflow can upload the `.deb` to that release.

Runner notes:

- On low-memory boards, consider configuring swap/zram locally on the runner to avoid OOM during Rust compilation.

## 📄 Documentation
You can find the documentation [here](https://immichframe.dev).

# Development

This application is built with [Tauri](https://github.com/tauri-apps/tauri).

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
