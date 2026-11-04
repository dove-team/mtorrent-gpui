#![cfg_attr(windows, windows_subsystem = "windows")]

mod host;
mod titlebar;

use gpui::{App, AppContext, Bounds, TitlebarOptions, WindowBounds, WindowOptions, px, size};
use gpui_platform::application;
use host::DesktopHost;
use log::LevelFilter;
use mtorrent_gpui_core::{
    AppView, Host, RuntimeConfig, Runtimes, i18n,
    logging::{Config, setup_log_rotation, spawn_writer_thread},
    ui_channel,
};
use std::io;
use std::net::TcpListener;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

const INSTANCE_GUARD_ADDR: &str = "127.0.0.1:49731";
const WINDOW_WIDTH: f32 = 900.;
const WINDOW_HEIGHT: f32 = 700.;
const WINDOW_MIN_WIDTH: f32 = 600.;
const WINDOW_MIN_HEIGHT: f32 = 400.;
const ICON: &[u8] = include_bytes!("../assets/icon.png");

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        rfd::MessageDialog::new()
            .set_level(rfd::MessageLevel::Error)
            .set_title("mtorrent")
            .set_description(e.to_string())
            .show();
        std::process::exit(1);
    }
}

fn instance_guard() -> io::Result<TcpListener> {
    TcpListener::bind(INSTANCE_GUARD_ADDR).map_err(|e| {
        if e.kind() == io::ErrorKind::AddrInUse {
            io::Error::new(io::ErrorKind::AlreadyExists, "mtorrent is already running.")
        } else {
            e
        }
    })
}

fn window_icon() -> io::Result<Arc<image::RgbaImage>> {
    let decoded = image::load_from_memory(ICON)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(Arc::new(decoded.to_rgba8()))
}

fn setup_logging(data_dir: &Path) -> io::Result<()> {
    let (log_sink, log_writer) = setup_log_rotation(Config {
        file_path: data_dir.join("mtorrent.log"),
        max_files: 3,
        max_file_size: 10 * 1024 * 1024,
        buffer_capacity: 32 * 1024,
    });

    spawn_writer_thread(log_writer)?;

    env_logger::Builder::from_default_env()
        .filter(None, LevelFilter::Debug)
        .target(env_logger::Target::Pipe(Box::new(log_sink)))
        .init();

    Ok(())
}

fn run() -> io::Result<()> {
    let _guard = instance_guard()?;

    let host = Rc::new(DesktopHost::new(cli_uri())?);
    setup_logging(&host.data_dir())?;
    i18n::apply(&*host);

    let runtimes = Runtimes::start(RuntimeConfig {
        data_dir: host.data_dir(),
        bind_interface: std::env::var("MTORRENT_NET_IF").ok(),
        ..Default::default()
    })?;

    let icon = window_icon()?;
    let title = format!("mtorrent v{}", env!("CARGO_PKG_VERSION")).into();
    let (signal, watcher) = ui_channel();

    application().run(move |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(WINDOW_WIDTH), px(WINDOW_HEIGHT)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(px(WINDOW_MIN_WIDTH), px(WINDOW_MIN_HEIGHT))),
                titlebar: Some(TitlebarOptions {
                    title: Some(title),
                    appears_transparent: true,
                    ..Default::default()
                }),
                icon: Some(icon),
                ..Default::default()
            },
            move |_, cx| {
                cx.new(|cx| AppView::new(host.clone(), runtimes, signal.clone(), watcher, cx))
            },
        )
        .expect("failed to open the mtorrent window");

        cx.activate(true);
    });

    Ok(())
}

fn cli_uri() -> Option<String> {
    std::env::args_os()
        .nth(1)
        .map(|arg| arg.to_string_lossy().into_owned())
}
