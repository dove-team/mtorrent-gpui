use gpui::{AnyElement, Div};
use std::future::Future;
use std::io;
use std::path::PathBuf;
use std::pin::Pin;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PickKind {
    TorrentFile,
    OutputFolder,
}

pub type PickFuture = Pin<Box<dyn Future<Output = io::Result<Option<PathBuf>>> + Send>>;

#[derive(Clone, Default)]
pub struct WindowChrome {
    pub drag_region: Option<Rc<dyn Fn(Div) -> Div>>,
    pub buttons: Option<Rc<dyn Fn() -> AnyElement>>,
}

pub trait Host {
    fn data_dir(&self) -> PathBuf;

    fn pref_dir(&self) -> PathBuf;

    fn initial_uri(&self) -> Option<String> {
        None
    }

    fn system_locale(&self) -> Option<String>;

    fn window_chrome(&self) -> WindowChrome {
        WindowChrome::default()
    }

    fn background_activity_file(&self) -> Option<PathBuf> {
        None
    }

    fn pick(&self, kind: PickKind) -> PickFuture;
}
