# mtorrent-gpui OpenHarmony

The OpenHarmony side of the client, split into a Rust native module and a DevEco ArkTS project that
loads it.

[English](README.md) | [简体中文](README.zh-CN.md) · Repository overview: [../README.md](../README.md)

| Path | Responsibility |
| --- | --- |
| `native/` | `mtorrent-gpui-ohos` crate, `crate-type = ["cdylib"]`, produces `libmtorrent_gpui_ohos.so` |
| `app/` | DevEco project (`AppScope/`, `entry/`, hvigor config); `entry/libs/arm64-v8a/` receives the `.so` |

## native/

`src/lib.rs` is the entry point: `#[openharmony_ability_derive::ability]` receives the
`OpenHarmonyApp`, starts the engine runtimes, calls `gpui_ohos::try_current_platform` and then
`Application::run_embedded`, which opens one window sized from `content_rect()`. Logging goes to
hilog through `ohos_hilog_binding`.

`src/host.rs` implements `core::Host` for the platform:

| Method | Behaviour |
| --- | --- |
| `data_dir` / `pref_dir` | `base_path()` / `pref_path()`; both resolve to the ability's `filesDir` |
| `system_locale` | `preferred_locales()` |
| `pick` | `show_file_dialog` from `openharmony-ability-plugin-files`, then `get_path_from_uri` |
| `window_chrome` | drag region only — a phone window has no minimize/maximize/close buttons |
| `background_activity_file` | `<filesDir>/bg_active`, see the continuous task below |

Magnet links do not come in through `initial_uri`. `module.init` runs the `#[ability]` function
before ArkTS stores `want.uri`, so that read would always be empty. Instead `src/lib.rs` registers
`Application::on_open_urls`, and the URIs reach the UI over the same `UiSignal` channel used for
progress updates: cold start is delivered after the first surface is ready, warm start arrives as a
new-want event. `core` turns each URI into a new task card with the link already filled in; starting
still requires choosing a folder and pressing the download button.

## Building the native module

Needs the OHOS NDK and the `*-unknown-linux-ohos` targets. `gpui-ohos` uses `zed-font-kit` with the
`source-fontconfig-dlopen` feature, which compiles `freetype-sys` for this target, so the OpenHarmony
sysroot has to provide libfreetype (`yeslogic-fontconfig-sys` is excluded for
`target_env = "ohos"`).

```bash
export OHOS_NDK_HOME=/usr/ohos/sdk/default/openharmony
cd ohos/native
ohrs build --arch aarch --dist ../app/entry/libs
ohrs build --release --arch aarch --dist ../app/entry/libs    # stop/resume testing needs this
```

`ohrs` writes `entry/libs/index.d.ts` and copies both `libmtorrent_gpui_ohos.so` and
`libc++_shared.so` into `entry/libs/arm64-v8a/`. A debug `.so` is around 500 MB; release is far
smaller, and the debug profile also aborts when stopping an active download (upstream
`debug_assert!`).

## app/

`entry/src/main/ets/entryability/EntryAbility.ets` extends `NativeAbility` from `@ohos-rs/ability`
with `moduleName = "mtorrent_gpui_ohos"` (no `lib` prefix, no `.so` suffix — the loader builds
`lib${moduleName}.so` and requires the exports `render` and `init`) and `defaultPage = true`, which
loads the HAR's own named route hosting the `XComponent` that GPUI renders into. The project's
`entry/src/main/ets/pages/Index.ets` is therefore never shown.

`entry/src/main/module.json5` declares:

- permissions `ohos.permission.INTERNET`, `GET_NETWORK_INFO`, `KEEP_BACKGROUND_RUNNING`
- a second skill for magnet links: `actions: ["ohos.want.action.viewData"]`,
  `entities: ["entity.system.browsable"]`, `uris: [{ "scheme": "magnet" }]`. Only `scheme` is set on
  purpose: when the declared `host` is empty the match succeeds on scheme alone, and magnet URIs have
  no authority section to match.
- `backgroundModes: ["dataTransfer"]` on the ability

Background continuity: `onBackground` reads `<filesDir>/bg_active` (written by `core` whenever the
active-download state flips) and, if it says `1`, calls
`backgroundTaskManager.startBackgroundRunning(context, ["dataTransfer"], wantAgent)`; `onForeground`
calls `stopBackgroundRunning`. This is why the file exists: nothing in `@ohos-rs/ability`,
`openharmony-ability` or `gpui-ohos` exposes the app state to ArkTS.

## Building and installing the HAP

```powershell
cd ohos\app
& "<DevEco>\tools\ohpm\bin\ohpm.bat" install --all
$env:DEVECO_SDK_HOME = "<DevEco>\sdk"
& "<DevEco>\tools\hvigor\bin\hvigorw.bat" assembleHap --mode module -p product=default --no-daemon
```

Two things to know about `build-profile.json5`:

- `compileSdkVersion` must be `"26.0.0"` (a string). `@ohos-rs/ability@1.0.0-beta.5` uses
  `SubWindowOptions.zLevelAboveParentLoosened`, which is `@since 26.0.0`; HAR sources are compiled
  with the rest of the app, so a lower `compileSdkVersion` fails the build with
  `Property 'zLevelAboveParentLoosened' does not exist on type 'SubWindowOptions'`. For API 26 and
  later the version has to be a string; for API 10–25 it has to be numeric.
  `targetSdkVersion` and `compatibleSdkVersion` can stay at 23.
- `signingConfigs` is empty, so hvigor prints `Will skip sign 'hos_hap'` and produces
  `entry-default-unsigned.hap`, which a device will not install. Generate a signing profile for
  `com.baigegroup.mtorrent` first.

## What is verified and what is not

Verified: the native module cross-compiles for `aarch64-unknown-linux-ohos`, the HAP packages, and
the packaged `module.json` really contains the magnet skill, `backgroundModes` and the permission.
Not verified: anything on a physical device — window size on first frame, whether the magnet link
fills the input box, whether the continuous task survives background (HarmonyOS cancels a
data-transfer task whose progress is not refreshed within roughly 10 minutes and expects those
updates to go through a Live View), and the 2in1 freeform drag behaviour.

## License

Apache-2.0 — see [../LICENSE](../LICENSE).
