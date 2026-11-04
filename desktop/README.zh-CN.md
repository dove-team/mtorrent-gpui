# mtorrent-gpui-desktop

桌面端前端，基于 `mtorrent-gpui-core` 和平台无关的 GPUI 控件，产出单个可执行文件
`mtorrent-gpui-desktop`。

[English](README.md) | [简体中文](README.zh-CN.md) · 仓库总览见 [../README.zh-CN.md](../README.zh-CN.md)

## 目录内容

| 路径 | 职责 |
| --- | --- |
| `src/main.rs` | `127.0.0.1:49731` 单实例守护、窗口参数（`900×700`、最小 `600×400`、`appears_transparent` 标题栏）、日志初始化、把 `argv[1]` 作为初始链接 |
| `src/host.rs` | 桌面 `Host`：`dirs-next` 取数据目录、`rfd` 文件与目录对话框（放独立线程，结果经 `tokio::sync::oneshot` 回主线程）、从环境变量推断系统语言 |
| `src/titlebar.rs` | 通过 `Host::window_chrome` 交给 `core` 的窗口装饰：把标题块标成 `WindowControlArea::Drag` 的闭包，加上最小化/最大化/关闭三个按钮 |
| `build.rs` | 把 `assets/icon.png` 包成 `.res`（`RT_ICON` 存 PNG 原字节 + 一条 `RT_GROUP_ICON` 目录项），经 `cargo:rustc-link-arg-bins` 交给 `link.exe`；只在 `CARGO_CFG_WINDOWS` 且 `target_env = msvc` 下生效，不需要 `rc.exe` |

## 运行

```powershell
cargo run -p mtorrent-gpui-desktop
cargo run -p mtorrent-gpui-desktop -- "magnet:?xt=urn:btih:..."   # 也可以传 .torrent 路径
```

数据和日志放在平台数据目录（Windows 上是 `%LOCALAPPDATA%\mtorrent-gpui`）：`mtorrent.log` 按 10 MiB × 3
个文件滚动，`lang` 存界面语言，`.mtorrent_cfg` 存额外 tracker。

默认路由不对时指定网卡：

```powershell
$env:MTORRENT_NET_IF = "192.168.1.10"
```

## 自绘标题栏的几点

`TitlebarOptions.appears_transparent = true` 会去掉系统 caption，整个窗口都是 GPUI 客户区。拖拽和
最小化/最大化/关闭都是靠 `window_control_area(...)` 标记区域、由窗口管理器执行动作，视图只负责画图形。
因为命中测试是沿命中链取第一个匹配区域，工具条按钮和窗口按钮必须是拖拽区的兄弟节点而不是子节点——
放在拖拽区里的控件收不到点击。缩放边框、双击最大化、贴边分屏都交给 Windows 自身。

## 桌面端已知限制

- HTTPS tracker 与 HTTPS web seed 用不了：引擎没有 TLS 后端。种子只带 https announce 时，往
  `.mtorrent_cfg` 里补 `udp://` tracker。
- debug 构建下停止活跃下载会撞上上游 3 秒 join 超时后的 `debug_assert!`，进程直接终止。测停止/续传请用
  release 构建。
- 冷启动后首轮 announce 可能有一两分钟对等方列表是空的（实测观察，原因未定位）；announce 返回后速度本身不受影响。

## 许可证

Apache-2.0，见 [../LICENSE](../LICENSE)。
