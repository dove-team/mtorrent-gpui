# mtorrent-gpui

基于 [GPUI](https://crates.io/crates/gpui-pre) 的 BitTorrent 客户端，开源鸿蒙与桌面端共用同一套界面和
同一套引擎编排层。下载引擎使用 [`mtorrent`](https://github.com/DanglingPointer/mtorrent) crate（0.10.1）。

[English](README.md) | 简体中文

## 目录结构

| 路径 | 内容 |
| --- | --- |
| [`core/`](core/README.zh-CN.md) | 与操作系统无关的统一层：引擎编排、下载状态、共享 GPUI 视图、`Host` trait、多语言、日志滚动 |
| [`desktop/`](desktop/README.zh-CN.md) | 桌面可执行文件：基于 `dirs-next` + `rfd` 的 `Host`、自绘标题栏、内嵌 exe 图标 |
| [`ohos/`](ohos/README.zh-CN.md) | 鸿蒙端：`ohos/native`（Rust cdylib + NAPI）与 `ohos/app`（DevEco ArkTS 工程） |
| `docs/` | 架构图（`architecture-runtime.svg`）与桌面端截图 |

workspace 配置为 `[workspace] members = ["core", "desktop", "ohos/native"]`。

## 两端是怎么隔开的

`core` 只依赖 `gpui-pre`，不依赖任何平台后端，代码里也没有 `cfg(target_env = "ohos")`。所有平台差异都
收敛在 `core::Host` trait 的方法上，由各端的二进制实现：

| `Host` 方法 | 桌面端 | 鸿蒙端 |
| --- | --- | --- |
| `data_dir` / `pref_dir` | `%LOCALAPPDATA%/mtorrent-gpui` | ability 上下文给的 `filesDir` |
| `pick` | `rfd` 对话框（独立线程） | `FilesPlugin` 文件选择 |
| `initial_uri` | `argv[1]`（.torrent 路径或磁力链） | 不使用 |
| `window_chrome` | 拖拽区 + 最小化/最大化/关闭 | 只有拖拽区 |
| `background_activity_file` | 不使用 | 单字节标记文件，ArkTS 退后台前读它决定要不要申请长时任务 |

`core` 也不做任何平台专属注册：URL 投递（`Application::on_open_urls`）是在 `ohos/native` 里接的，所以两端
跑的是同一份视图代码。

## 构建

桌面端（Windows 需要 MSVC 工具链；exe 图标由 `desktop/build.rs` 生成并内嵌）：

```powershell
cargo run -p mtorrent-gpui-desktop            # 可选：argv[1] 传 .torrent 路径或磁力链
```

鸿蒙端分两步，先出原生模块，再打 HAP：

```bash
# WSL / Linux，需要 OHOS_NDK_HOME 和 *-unknown-linux-ohos target
cd ohos/native
ohrs build --arch aarch --dist ../app/entry/libs
```

```powershell
cd ohos\app
& "<DevEco>\tools\ohpm\bin\ohpm.bat" install --all
$env:DEVECO_SDK_HOME = "<DevEco>\sdk"
& "<DevEco>\tools\hvigor\bin\hvigorw.bat" assembleHap --mode module -p product=default --no-daemon
```

具体路径、以及 ArkTS 依赖要求的 `compileSdkVersion` 取值见 [`ohos/README.zh-CN.md`](ohos/README.zh-CN.md)。

## 测试

```powershell
cargo test -p mtorrent-gpui-core          # 无头界面断言 + 状态/格式化/引擎单测
```

`core/tests/engine_download.rs` 标了 `#[ignore]`：它驱动真实引擎连公共 tracker，需要网络和可达的对等方。

## 界面语言

`rust_i18n` 在编译期把四个语言包嵌进二进制：英文、俄文、简体中文、繁体中文（`core/locales/`）。改 `.yml`
不会触发重编，改完要 `touch core/src/lib.rs`（或 `cargo clean -p mtorrent-gpui-core`）。

## 当前限制

- **没有 TLS。** `mtorrent-base` 引入 `reqwest` 时是 `default-features = false` 且没有 TLS 后端，锁文件里
  也解析不出 `rustls`/`native-tls`/`openssl`。因此 https tracker 无法 announce，https web seed 也取不到；
  请用 `udp://` tracker 和 `http://` web seed。引擎会把 `<data_dir>/.mtorrent_cfg` 里的 tracker 与种子自带的
  合并，所以不改种子文件也能补 UDP tracker。
- **debug 构建停止下载会终止进程。** 关闭活跃下载时上游那个 3 秒 join 超时走的是
  `debug_assert!(false, ...)`，debug profile 下直接 abort；要测停止/续传请用 release 构建。
- **鸿蒙端还没上过真机。** 目前验证到的是编译与打包层面：原生模块能按 `aarch64-unknown-linux-ohos` 编出来、
  HAP 能打出来、深链与长时任务声明确实进了包内的 `module.json`。
- **后台传输受系统约束。** 应用退到后台且有活跃下载时申请 `dataTransfer` 长时任务，但鸿蒙对数据传输类长时
  任务要求持续刷新进度（首次超过约 10 分钟未更新即取消），且进度通知预期要走实况窗。这一点尚未真机验证。
- **装机需要签名。** `signingConfigs` 为空时 hvigor 会跳过签名，只产出 `entry-default-unsigned.hap`。

## 许可证

Apache-2.0，见 [LICENSE](LICENSE)；该文件与上游 `mtorrent` 仓库的 LICENSE 逐字节一致。
