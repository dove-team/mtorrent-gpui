# mtorrent-gpui-core

开源鸿蒙端与桌面端共用的统一层：引擎编排、下载状态、GPUI 视图树、平台抽象、多语言与日志滚动。

[English](README.md) | [简体中文](README.zh-CN.md) · 仓库总览见 [../README.zh-CN.md](../README.zh-CN.md)

## 这个 crate 遵守的约定

- 只依赖 `gpui-pre`、`mtorrent`、`rust-i18n`、`parking_lot`、`file-rotate`、`tokio`，绝不依赖
  `gpui-ohos` 或 `gpui-pre-platform`，否则某个平台的后端会被拖进另一端的构建。
- 代码里不出现 `cfg(target_env = "ohos")`。平台差异一律走 `Host` trait 的方法，或由二进制传进来的值。
- 这里不画窗口装饰：`ui::header` 只提供两个插槽（`WindowChrome`），`WindowControlArea` 这些类型留在各平台 crate 里。

## 模块划分

| 路径 | 职责 |
| --- | --- |
| `src/host.rs` | `Host` trait（`data_dir`、`pref_dir`、`initial_uri`、`system_locale`、`window_chrome`、`background_activity_file`、`pick`）与 `WindowChrome` |
| `src/engine/` | `runtime.rs`（app、pwp、storage 三个 tokio handle 加 DHT handle）、`task.rs`（`TaskManager`：增删任务、启停、`run_id` 世代、在途计数、`open_uri`）、`listener.rs`（`StateListener` → 进度快照）、`signal.rs`（`UiSignal`/`UiWatcher` 与 `spawn_ui_pump`） |
| `src/state/` | `progress.rs`（`DownloadProgress`、`TaskStatus`、`PeerInfo`）、`format.rs`（字节与百分比格式化） |
| `src/ui/` | `theme.rs`、`header.rs`、`task_card.rs`、`peer_list.rs`、`input.rs`（自绘 `TextInput`，接好了 IME） |
| `src/app_view.rs` | 根视图实体：渲染任务卡片，持有输入框实体与对等方列表的展开状态 |
| `src/i18n.rs` | 语言列表、当前语言、偏好文件 `lang`、循环切换 |
| `src/logging.rs` | 缓冲日志 sink + 滚动文件写入线程；输出目标由调用方决定 |
| `locales/` | `en`、`ru`、`zh-CN`、`zh-Hant` |

## 界面是怎么被唤醒的

引擎回调落在 tokio 线程上，所以没有任何地方直接改 GPUI 状态。进度更新通过一个无界通道发出
`UiEvent`；`spawn_ui_pump` 跑在 GPUI 主线程，遇到 `UiEvent::OpenUri` 就交给 `TaskManager`，其余情况只做
刷新。平台经 `Application::on_open_urls` 送进来的磁力链接走的也是这条通道。

## 文案

`src/lib.rs` 里的 `rust_i18n::i18n!("locales", fallback = "en")` 在编译期把 YAML 嵌进二进制，路径按
`CARGO_MANIFEST_DIR` 解析。运行期不读这些文件，所以桌面 exe 和鸿蒙 `.so` 都不用带文案目录。由于 proc
macro 不会把 YAML 登记成构建依赖，改完翻译要 `touch src/lib.rs`（或 `cargo clean -p mtorrent-gpui-core`）
才会生效。

## 测试

```bash
cargo test -p mtorrent-gpui-core
```

`tests/ui_layout.rs` 用 `test-support` 后端渲染真实控件（不开窗口、不需要设备）；`tests/engine_download.rs`
标了 `#[ignore]`，会真的对着公共 tracker 跑一次下载。

## 许可证

Apache-2.0，见 [../LICENSE](../LICENSE)。
