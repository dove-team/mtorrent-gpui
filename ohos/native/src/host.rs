use gpui::{Div, InteractiveElement, WindowControlArea};
use mtorrent_gpui_core::host::{Host, PickFuture, PickKind, WindowChrome};
use mtorrent_gpui_core::i18n;
use openharmony_ability::OpenHarmonyApp;
use openharmony_ability_plugin_files::{FileDialogFilter, FileDialogOptions, FilesExt, dialog_type};
use std::{io, path::PathBuf, rc::Rc};

pub struct OhosHost {
    app: OpenHarmonyApp,
    data_dir: PathBuf,
    pref_dir: PathBuf,
}

impl OhosHost {
    pub fn new(app: OpenHarmonyApp) -> io::Result<Self> {
        let data_dir = app
            .base_path()
            .map(PathBuf::from)
            .ok_or_else(|| io::Error::other("ability base path is unavailable"))?;
        std::fs::create_dir_all(&data_dir)?;

        let pref_dir = app
            .pref_path()
            .map(PathBuf::from)
            .unwrap_or_else(|| data_dir.clone());

        Ok(Self {
            app,
            data_dir,
            pref_dir,
        })
    }
}

impl Host for OhosHost {
    fn data_dir(&self) -> PathBuf {
        self.data_dir.clone()
    }

    fn pref_dir(&self) -> PathBuf {
        self.pref_dir.clone()
    }

    fn background_activity_file(&self) -> Option<PathBuf> {
        Some(self.data_dir.join("bg_active"))
    }

    fn system_locale(&self) -> Option<String> {
        self.app.preferred_locales()
    }

    fn window_chrome(&self) -> WindowChrome {
        WindowChrome {
            drag_region: Some(Rc::new(|block: Div| {
                block.window_control_area(WindowControlArea::Drag)
            })),
            buttons: None,
        }
    }

    fn pick(&self, kind: PickKind) -> PickFuture {
        let options = match kind {
            PickKind::TorrentFile => FileDialogOptions::new(dialog_type::OPEN_FILE).filters(vec![
                FileDialogFilter::new()
                    .name(i18n::torrent_filter_label())
                    .pattern("torrent"),
            ]),
            PickKind::OutputFolder => FileDialogOptions::new(dialog_type::OPEN_FOLDER),
        };

        let app = self.app.clone();
        Box::pin(async move {
            let response = app.show_file_dialog(options).await.map_err(io::Error::other)?;

            let selected = response
                .files
                .first()
                .map(|uri| {
                    ohos_fileuri_binding::get_path_from_uri(uri)
                        .map(PathBuf::from)
                        .map_err(io::Error::other)
                })
                .transpose()?;

            Ok(selected)
        })
    }
}
