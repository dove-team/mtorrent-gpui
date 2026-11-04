mod host;

use gpui::{App, AppContext, Application, ApplicationHandle, Bounds, WindowBounds, WindowOptions, px, size};
use host::OhosHost;
use log::LevelFilter;
use mtorrent_gpui_core::{AppView, Host, RuntimeConfig, Runtimes, i18n, ui_channel};
use ohos_hilog_binding::log::Config as HilogConfig;
use openharmony_ability::OpenHarmonyApp;
use std::cell::RefCell;
use std::rc::Rc;

thread_local! {
    #[allow(clippy::missing_const_for_thread_local)]
    static APPLICATION: RefCell<Option<ApplicationHandle>> = const { RefCell::new(None) };
}

#[openharmony_ability_derive::ability]
pub fn openharmony_app(app: OpenHarmonyApp) {
    ohos_hilog_binding::log::init_once(HilogConfig::default().with_max_level(LevelFilter::Info));

    let host = match OhosHost::new(app.clone()) {
        Ok(host) => Rc::new(host),
        Err(e) => {
            log::error!("Failed to prepare the OHOS host: {e}");
            return;
        }
    };

    i18n::apply(&*host);

    let runtimes = match Runtimes::start(RuntimeConfig {
        data_dir: host.data_dir(),
        ..Default::default()
    }) {
        Ok(runtimes) => runtimes,
        Err(e) => {
            log::error!("Failed to start download runtimes: {e}");
            return;
        }
    };

    let platform = match gpui_ohos::try_current_platform(
        app.clone(),
        gpui_ohos::PlatformOptions::default(),
    ) {
        Ok(platform) => platform,
        Err(e) => {
            log::error!("Failed to initialize the OHOS GPUI platform: {e}");
            return;
        }
    };

    let inner_app = app.clone();
    let (signal, watcher) = ui_channel();
    let application = Application::with_platform(platform);

    let urls_signal = signal.clone();
    application.on_open_urls(move |urls| {
        for uri in urls {
            urls_signal.open_uri(uri);
        }
    });

    let application_handle = application.run_embedded(move |cx: &mut App| {
        let rect = inner_app.content_rect();
        let default_size = size(px(rect.width as f32), px(rect.height as f32));
        let bounds = Bounds::centered(None, default_size, cx);

        if let Err(e) = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            move |_, cx| {
                cx.new(|cx| AppView::new(host.clone(), runtimes, signal.clone(), watcher, cx))
            },
        ) {
            log::error!("Failed to open the mtorrent window: {e}");
            return;
        }

        cx.activate(true);
    });

    APPLICATION.with(|application| {
        application.replace(Some(application_handle));
    });
}
