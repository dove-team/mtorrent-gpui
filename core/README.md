# mtorrent-gpui-core

Platform-neutral layer shared by the OpenHarmony and desktop binaries: engine orchestration,
download state, the GPUI view tree, the platform abstraction, i18n and log rotation.

[English](README.md) | [简体中文](README.zh-CN.md) · Repository overview: [../README.md](../README.md)

## Rules this crate follows

- Depends only on `gpui-pre`, `mtorrent`, `rust-i18n`, `parking_lot`, `file-rotate`, `tokio` — never
  on `gpui-ohos` or `gpui-pre-platform`, otherwise a backend would leak into the other target's build.
- No `cfg(target_env = "ohos")` anywhere. Platform differences are `Host` trait methods or values the
  binary passes in.
- Window decoration is not drawn here: `ui::header` only offers two slots (`WindowChrome`), so the
  `WindowControlArea` types stay inside the platform crates.

## Modules

| Path | Responsibility |
| --- | --- |
| `src/host.rs` | `Host` trait (`data_dir`, `pref_dir`, `initial_uri`, `system_locale`, `window_chrome`, `background_activity_file`, `pick`) and `WindowChrome` |
| `src/engine/` | `runtime.rs` (the app, pwp and storage tokio handles plus the DHT handle), `task.rs` (`TaskManager`: add/remove tasks, start/stop, `run_id` generations, in-flight accounting, `open_uri`), `listener.rs` (`StateListener` → progress snapshots), `signal.rs` (`UiSignal`/`UiWatcher` + `spawn_ui_pump`) |
| `src/state/` | `progress.rs` (`DownloadProgress`, `TaskStatus`, `PeerInfo`), `format.rs` (byte and percent formatting) |
| `src/ui/` | `theme.rs`, `header.rs`, `task_card.rs`, `peer_list.rs`, `input.rs` (custom `TextInput` with IME wiring) |
| `src/app_view.rs` | Root GPUI entity: renders task cards, owns the input entities and the peer-list expansion state |
| `src/i18n.rs` | Locale list, current locale, preference file (`lang`), cycle switching |
| `src/logging.rs` | Buffered log sink + rotating file writer; the destination is chosen by the caller |
| `locales/` | `en`, `ru`, `zh-CN`, `zh-Hant` |

## UI wake-up path

Engine callbacks land on the tokio threads, so nothing touches GPUI state directly. Progress updates
send a `UiEvent` over an unbounded channel; `spawn_ui_pump` runs on the GPUI main thread, applies
`UiEvent::OpenUri` to the `TaskManager` and otherwise just refreshes. The same channel carries the
magnet links delivered by the platform through `Application::on_open_urls`.

## Translations

`rust_i18n::i18n!("locales", fallback = "en")` in `src/lib.rs` embeds the YAML files at compile time
and resolves the path against `CARGO_MANIFEST_DIR`. Nothing reads those files at runtime, so neither
the desktop executable nor the OpenHarmony `.so` needs to ship them. Because a proc macro does not
register the YAML files as build dependencies, editing a translation requires touching `src/lib.rs`
(or `cargo clean -p mtorrent-gpui-core`) before the change appears.

## Tests

```bash
cargo test -p mtorrent-gpui-core
```

`tests/ui_layout.rs` renders real widgets through the `test-support` backend (no window, no device);
`tests/engine_download.rs` is `#[ignore]` and performs an actual download against public trackers.

## License

Apache-2.0 — see [../LICENSE](../LICENSE).
