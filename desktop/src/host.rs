use crate::titlebar;
use mtorrent_gpui_core::host::{Host, PickFuture, PickKind, WindowChrome};
use std::{io, path::PathBuf};

const APP_DIR_NAME: &str = "mtorrent-gpui";

pub struct DesktopHost {
    data_dir: PathBuf,
    pref_dir: PathBuf,
    initial_uri: Option<String>,
}

impl DesktopHost {
    pub fn new(initial_uri: Option<String>) -> io::Result<Self> {
        let base = dirs_next::data_local_dir()
            .or_else(dirs_next::data_dir)
            .or_else(dirs_next::config_dir)
            .ok_or_else(|| io::Error::other("no writable data directory"))?;
        let data_dir = base.join(APP_DIR_NAME);
        std::fs::create_dir_all(&data_dir)?;

        Ok(Self {
            pref_dir: data_dir.clone(),
            data_dir,
            initial_uri,
        })
    }
}

impl Host for DesktopHost {
    fn data_dir(&self) -> PathBuf {
        self.data_dir.clone()
    }

    fn pref_dir(&self) -> PathBuf {
        self.pref_dir.clone()
    }

    fn initial_uri(&self) -> Option<String> {
        self.initial_uri.clone()
    }

    fn window_chrome(&self) -> WindowChrome {
        titlebar::chrome()
    }

    fn system_locale(&self) -> Option<String> {
        for key in ["LC_ALL", "LC_MESSAGES", "LANG"] {
            if let Ok(value) = std::env::var(key) {
                let normalized = value
                    .split(['.', '@'])
                    .next()
                    .unwrap_or_default()
                    .replace('_', "-");
                if !normalized.is_empty() {
                    return Some(normalized);
                }
            }
        }
        None
    }

    fn pick(&self, kind: PickKind) -> PickFuture {
        let filter_name = mtorrent_gpui_core::i18n::torrent_filter_label();
        let (tx, rx) = tokio::sync::oneshot::channel::<Option<PathBuf>>();

        Box::pin(async move {
            let _worker = std::thread::Builder::new()
                .name("file-dialog".to_owned())
                .spawn(move || {
                    let selected = match kind {
                        PickKind::TorrentFile => rfd::FileDialog::new()
                            .add_filter(filter_name, &["torrent"])
                            .pick_file(),
                        PickKind::OutputFolder => rfd::FileDialog::new().pick_folder(),
                    };
                    let _ = tx.send(selected);
                })?;

            rx.await
                .map_err(|_| io::Error::other("file dialog thread ended without a result"))
        })
    }
}
