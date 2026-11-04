# mtorrent-gpui 鸿蒙端

客户端的开源鸿蒙实现，分成 Rust 原生模块与加载它的 DevEco ArkTS 工程两部分。

[English](README.md) | [简体中文](README.zh-CN.md) · 仓库总览见 [../README.zh-CN.md](../README.zh-CN.md)

| 路径 | 职责 |
| --- | --- |
| `native/` | `mtorrent-gpui-ohos` crate，`crate-type = ["cdylib"]`，产出 `libmtorrent_gpui_ohos.so` |
| `app/` | DevEco 工程（`AppScope/`、`entry/`、hvigor 配置）；`entry/libs/arm64-v8a/` 是 `.so` 的落点 |

## native/

入口在 `src/lib.rs`：`#[openharmony_ability_derive::ability]` 拿到 `OpenHarmonyApp`，启动引擎 runtime，
调 `gpui_ohos::try_current_platform`，再 `Application::run_embedded` 开一个尺寸取自 `content_rect()` 的窗口。
日志经 `ohos_hilog_binding` 走 hilog。

`src/host.rs` 是给 `core::Host` 的鸿蒙实现：

| 方法 | 行为 |
| --- | --- |
| `data_dir` / `pref_dir` | `base_path()` / `pref_path()`，两者都落到 ability 的 `filesDir` |
| `system_locale` | `preferred_locales()` |
| `pick` | `openharmony-ability-plugin-files` 的 `show_file_dialog`，再用 `get_path_from_uri` 换成沙箱路径 |
| `window_chrome` | 只交拖拽区——手机窗口没有最小化/最大化/关闭 |
| `background_activity_file` | `<filesDir>/bg_active`，见下面长时任务一节 |

磁力链接不是从 `initial_uri` 进来的。`module.init` 会先跑 `#[ability]` 函数，ArkTS 之后才把 `want.uri`
存进 Rust，那次读取必然为空。所以 `src/lib.rs` 注册的是 `Application::on_open_urls`，URI 和进度更新共用
`UiSignal` 通道：冷启动在首个 surface 就绪后投递，热启动以 new-want 事件投递。`core` 把每个 URI 变成一张
输入框已填好的新卡片，仍需选目录再点开始下载。

## 编译原生模块

需要 OHOS NDK 和 `*-unknown-linux-ohos` target。`gpui-ohos` 用的是带 `source-fontconfig-dlopen` 的
`zed-font-kit`，这个 target 下会编 `freetype-sys`，所以 OpenHarmony sysroot 里必须有 libfreetype
（`yeslogic-fontconfig-sys` 在 `target_env = "ohos"` 下被排除，不需要）。

```bash
export OHOS_NDK_HOME=/usr/ohos/sdk/default/openharmony
cd ohos/native
ohrs build --arch aarch --dist ../app/entry/libs
ohrs build --release --arch aarch --dist ../app/entry/libs    # 要测停止/续传就用这个
```

`ohrs` 会生成 `entry/libs/index.d.ts`，并把 `libmtorrent_gpui_ohos.so` 和 `libc++_shared.so` 拷到
`entry/libs/arm64-v8a/`。debug 的 `.so` 约 500 MB，release 小得多；而且 debug profile 下停止活跃下载会撞
上游 `debug_assert!` 直接终止进程。

## app/

`entry/src/main/ets/entryability/EntryAbility.ets` 继承 `@ohos-rs/ability` 的 `NativeAbility`，
`moduleName = "mtorrent_gpui_ohos"`（不带 `lib` 前缀也不带 `.so` 后缀，loader 自己拼
`lib${moduleName}.so`，并要求模块导出 `render` 与 `init`），`defaultPage = true` 加载的是 HAR 自带的命名
路由页面，那页挂着 GPUI 渲染的目标 `XComponent`。因此工程里的 `entry/src/main/ets/pages/Index.ets` 不会显示。

`entry/src/main/module.json5` 声明了：

- 权限 `ohos.permission.INTERNET`、`GET_NETWORK_INFO`、`KEEP_BACKGROUND_RUNNING`
- 磁力链接用的第二条 skill：`actions: ["ohos.want.action.viewData"]`、`entities:
  ["entity.system.browsable"]`、`uris: [{ "scheme": "magnet" }]`。这里故意只写 `scheme`：配置项的 `host`
  为空时只要 scheme 相同就算匹配成功，而磁力链接根本没有 authority 段。
- ability 上的 `backgroundModes: ["dataTransfer"]`

后台续传的做法：`onBackground` 读 `<filesDir>/bg_active`（`core` 在活跃下载状态翻转时写入），值为 `1` 才调
`backgroundTaskManager.startBackgroundRunning(context, ["dataTransfer"], wantAgent)`；`onForeground` 调
`stopBackgroundRunning`。之所以要这个文件，是因为 `@ohos-rs/ability`、`openharmony-ability`、`gpui-ohos`
里都没有把应用状态交给 ArkTS 的通道。

## 打包与安装 HAP

```powershell
cd ohos\app
& "<DevEco>\tools\ohpm\bin\ohpm.bat" install --all
$env:DEVECO_SDK_HOME = "<DevEco>\sdk"
& "<DevEco>\tools\hvigor\bin\hvigorw.bat" assembleHap --mode module -p product=default --no-daemon
```

`build-profile.json5` 有两点要注意：

- `compileSdkVersion` 必须是 `"26.0.0"`（字符串）。`@ohos-rs/ability@1.0.0-beta.5` 用了
  `SubWindowOptions.zLevelAboveParentLoosened`，该属性 `@since 26.0.0`；HAR 的源码是和主工程一起编译的，
  `compileSdkVersion` 低于 26 就会报
  `Property 'zLevelAboveParentLoosened' does not exist on type 'SubWindowOptions'`。API 26 及以后版本号要写
  字符串，API 10–25 才写数字。`targetSdkVersion` 和 `compatibleSdkVersion` 保持 23 即可。
- `signingConfigs` 为空，hvigor 会打印 `Will skip sign 'hos_hap'`，只产出
  `entry-default-unsigned.hap`，设备不认。要先给 `com.baigegroup.mtorrent` 生成签名配置。

## 已验证与未验证

已验证：原生模块能按 `aarch64-unknown-linux-ohos` 编出来，HAP 能打出来，包内 `module.json` 确实带着 magnet
skill、`backgroundModes` 和那条权限。未验证：真机上的任何行为——首帧窗口尺寸、点磁力链接是否把输入框填上、
长时任务能不能撑住后台（鸿蒙对数据传输类长时任务要求约 10 分钟内刷新进度，且进度通知预期走实况窗）、以及
2in1 自由窗口下的拖拽表现。

## 许可证

Apache-2.0，见 [../LICENSE](../LICENSE)。
