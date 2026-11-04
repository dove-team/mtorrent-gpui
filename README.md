# mtorrent-gpui

A BitTorrent client built on [GPUI](https://crates.io/crates/gpui-pre), sharing one UI and one
engine layer between OpenHarmony and desktop. The download engine is the
[`mtorrent`](https://github.com/DanglingPointer/mtorrent) crate (0.10.1).

English | [简体中文](README.zh-CN.md)

## Layout

| Path | What it holds |
| --- | --- |
| [`core/`](core/README.md) | Platform-neutral layer: engine orchestration, download state, shared GPUI views, `Host` trait, i18n, log rotation |
| [`desktop/`](desktop/README.md) | Desktop binary: `Host` on `dirs-next` + `rfd`, self-drawn title bar, embedded exe icon |
| [`ohos/`](ohos/README.md) | OpenHarmony side: `ohos/native` (Rust cdylib, NAPI) and `ohos/app` (DevEco ArkTS project) |
| `docs/` | Architecture diagram (`architecture-runtime.svg`) and a desktop screenshot |

The workspace is `[workspace] members = ["core", "desktop", "ohos/native"]`.

## How the platforms are separated

`core` depends only on `gpui-pre` and never on a platform backend, and contains no
`cfg(target_env = "ohos")`. Every platform difference is a method on the `core::Host` trait,
implemented by each binary:

| `Host` method | Desktop | OpenHarmony |
| --- | --- | --- |
| `data_dir` / `pref_dir` | `%LOCALAPPDATA%/mtorrent-gpui` | `filesDir` from the ability context |
| `pick` | `rfd` dialog on a worker thread | `FilesPlugin` file dialog |
| `initial_uri` | `argv[1]` (`.torrent` path or magnet link) | not used |
| `window_chrome` | drag region + minimize/maximize/close | drag region only |
| `background_activity_file` | not used | one-byte marker read by ArkTS before requesting a continuous task |

`core` also registers nothing platform specific: URL handoff (`Application::on_open_urls`) is wired
in `ohos/native`, so the same UI code runs on both ends.

## Build

Desktop (Windows needs the MSVC toolchain; the exe icon is embedded by `desktop/build.rs`):

```powershell
cargo run -p mtorrent-gpui-desktop            # optional: pass a .torrent path or a magnet link as argv[1]
```

OpenHarmony has two stages — the native module, then the HAP:

```bash
# WSL / Linux, requires OHOS_NDK_HOME and the *-unknown-linux-ohos targets
cd ohos/native
ohrs build --arch aarch --dist ../app/entry/libs
```

```powershell
cd ohos\app
& "<DevEco>\tools\ohpm\bin\ohpm.bat" install --all
$env:DEVECO_SDK_HOME = "<DevEco>\sdk"
& "<DevEco>\tools\hvigor\bin\hvigorw.bat" assembleHap --mode module -p product=default --no-daemon
```

See [`ohos/README.md`](ohos/README.md) for the exact paths and for the `compileSdkVersion` value the
ArkTS dependency requires.

## Tests

```powershell
cargo test -p mtorrent-gpui-core          # headless UI assertions + state/format/engine units
```

`core/tests/engine_download.rs` is marked `#[ignore]`: it drives the real engine against public
trackers, so it needs network and reachable peers.

## UI languages

Four locales are compiled into the binary at build time by `rust_i18n`: English, Russian,
Simplified Chinese, Traditional Chinese (`core/locales/`). Editing a `.yml` does not trigger a
rebuild — touch `core/src/lib.rs` after changing translations.

## Current limitations

- **No TLS.** `mtorrent-base` pulls `reqwest` with `default-features = false` and no TLS backend, and
  the lockfile resolves no `rustls`/`native-tls`/`openssl`. HTTPS trackers and HTTPS web seeds
  therefore cannot announce or fetch; use `udp://` trackers and plain `http://` web seeds. The engine
  merges `<data_dir>/.mtorrent_cfg` with the trackers inside a torrent, so you can add UDP trackers
  there without touching the torrent file.
- **Debug builds abort when stopping a download.** Shutting down an active download hits the
  upstream `debug_assert!(false, ...)` after a 3-second join timeout, which aborts the process in a
  debug profile. Use a release build to exercise stop/resume.
- **OpenHarmony has not been run on a device yet.** Everything verified so far is compile and
  package level: the native module builds for `aarch64-unknown-linux-ohos`, the HAP packages, and the
  deep-link and continuous-task declarations land in the packaged `module.json`.
- **Background transfers are bounded by the system.** The app requests a `dataTransfer` continuous
  task when it goes to background with an active download, but HarmonyOS cancels a data-transfer task
  whose progress is not refreshed within about 10 minutes, and progress refresh is expected to go
  through a Live View. That part is unverified.
- **Signing is required to install.** With an empty `signingConfigs`, hvigor skips signing and
  produces `entry-default-unsigned.hap`.

## License

Apache-2.0. See [LICENSE](LICENSE), which is byte-identical to the license file of the upstream
`mtorrent` repository.
