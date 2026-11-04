use crate::engine::TaskManager;
use gpui::{App, Entity, Task};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

pub enum UiEvent {
    Refresh,
    OpenUri(String),
}

#[derive(Clone)]
pub struct UiSignal {
    tx: UnboundedSender<UiEvent>,
}

impl UiSignal {
    pub fn ping(&self) {
        let _ = self.tx.send(UiEvent::Refresh);
    }

    pub fn open_uri(&self, uri: String) {
        let _ = self.tx.send(UiEvent::OpenUri(uri));
    }
}

pub struct UiWatcher {
    rx: UnboundedReceiver<UiEvent>,
}

pub fn ui_channel() -> (UiSignal, UiWatcher) {
    let (tx, rx) = unbounded_channel();
    (UiSignal { tx }, UiWatcher { rx })
}

pub fn spawn_ui_pump(app: &App, watcher: UiWatcher, manager: Entity<TaskManager>) -> Task<()> {
    app.spawn(async move |cx| {
        let mut rx = watcher.rx;
        while let Some(event) = rx.recv().await {
            match event {
                UiEvent::Refresh => cx.refresh(),
                UiEvent::OpenUri(uri) => {
                    cx.update(|app| manager.update(app, |manager, _| manager.open_uri(uri)));
                    cx.refresh();
                }
            }
        }
    })
}
