# mtorrent-gpui-desktop

Desktop front end. It builds a single binary, `mtorrent-gpui-desktop`, on top of
`mtorrent-gpui-core` and the platform-neutral GPUI widgets.

[English](README.md) | [简体中文](README.zh-CN.md) · Repository overview: [../README.md](../README.md)

## Contents

| Path | Responsibility |
| --- | --- |
| `src/main.rs` | Single-instance guard on `127.0.0.1:49731`, window options (`900×700`, minimum `600×400`, `appears_transparent` title bar), log setup, `argv[1]` as the initial link |
| `src/host.rs` | `Host` for the desktop: `dirs-next` data dir, `rfd` file and folder dialogs on their own thread with the result returned over a `tokio::sync::oneshot`, environment locale detection |
| `src/titlebar.rs` | The window chrome handed to `core` through `Host::window_chrome`: a closure that marks the title block as `WindowControlArea::Drag`, plus the minimize/maximize/close buttons |
| `build.rs` | Wraps `assets/icon.png` into a `.res` (raw PNG as `RT_ICON` plus one `RT_GROUP_ICON` entry) and passes it to `link.exe` via `cargo:rustc-link-arg-bins`; runs only under `CARGO_CFG_WINDOWS` with `target_env = msvc` and needs no `rc.exe` |

## Run

```powershell
cargo run -p mtorrent-gpui-desktop
cargo run -p mtorrent-gpui-desktop -- "magnet:?xt=urn:btih:..."   # or a .torrent path
```

Data and logs live in the platform data directory (`%LOCALAPPDATA%\mtorrent-gpui` on Windows):
`mtorrent.log` rotated at 10 MiB × 3 files, `lang` for the UI language, `.mtorrent_cfg` for extra
trackers.

Bind a specific interface when the default route is wrong:

```powershell
$env:MTORRENT_NET_IF = "192.168.1.10"
```

## Notes on the self-drawn title bar

`TitlebarOptions.appears_transparent = true` removes the native caption, so the whole window is the
GPUI client area. Dragging and the minimize/maximize/close buttons are marked with
`window_control_area(...)` and the window manager performs the action; the view only draws the glyphs.
Because hit testing walks the hitbox chain and takes the first matching area, the toolbar buttons and
the window buttons are siblings of the drag region, not its children — a control inside a drag region
never receives clicks. Resize borders, double-click maximize and edge snapping are all left to Windows.

## Limitations seen on desktop

- HTTPS trackers and HTTPS web seeds do not work: the engine has no TLS backend. Add `udp://`
  trackers to `.mtorrent_cfg` if a torrent only announces over HTTPS.
- In a debug build, stopping an active download can hit an upstream `debug_assert!` after a 3-second
  join timeout and abort the process. Use a release build for stop/resume testing.
- The first tracker announce can leave the peer list empty for one to two minutes on a cold start
  (observed, cause not located). Download speed itself is unaffected once the announce returns.

## License

Apache-2.0 — see [../LICENSE](../LICENSE).
