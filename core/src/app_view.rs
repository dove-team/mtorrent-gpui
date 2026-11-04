use crate::engine::{Runtimes, TaskManager, UiSignal, UiWatcher, spawn_ui_pump};
use crate::host::{Host, PickKind};
use crate::i18n;
use crate::state::format::format_percent;
use crate::state::progress::PeerInfo;
use crate::ui::{self, TextInput};
use gpui::{Context, Entity, IntoElement, Render, Task, Window, div, prelude::*, px, rgb};
use rust_i18n::t;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::PathBuf;
use std::rc::Rc;

pub(crate) struct CardData {
    index: usize,
    id: usize,
    title: String,
    input: String,
    ratio: f32,
    percent: String,
    status_line: String,
    is_downloading: bool,
    peers: BTreeMap<String, PeerInfo>,
}

pub struct AppView {
    host: Rc<dyn Host>,
    manager: Entity<TaskManager>,
    inputs: HashMap<usize, Entity<TextInput>>,
    peers_expanded: HashSet<usize>,
    activity_file: Option<PathBuf>,
    activity_written: Option<bool>,
    pending: Vec<Task<()>>,
    _pump: Task<()>,
}

impl AppView {
    pub fn new(
        host: Rc<dyn Host>,
        runtimes: Runtimes,
        signal: UiSignal,
        watcher: UiWatcher,
        cx: &mut Context<Self>,
    ) -> Self {
        cx.bind_keys(ui::key_bindings());

        let initial_uri = host.initial_uri();
        let activity_file = host.background_activity_file();
        let manager = cx.new(move |_| TaskManager::new(runtimes, signal, initial_uri));
        let pump = spawn_ui_pump(cx, watcher, manager.clone());
        Self::register_exit_handling(cx, &manager);

        Self {
            host,
            manager,
            inputs: HashMap::new(),
            peers_expanded: HashSet::new(),
            activity_file,
            activity_written: None,
            pending: Vec::new(),
            _pump: pump,
        }
    }

    fn register_exit_handling(cx: &mut Context<Self>, manager: &Entity<TaskManager>) {
        cx.set_quit_mode(gpui::QuitMode::Explicit);

        let weak_manager = manager.downgrade();
        cx.on_window_closed(move |app, _| {
            if !app.windows().is_empty() {
                return;
            }

            if let Some(manager) = weak_manager.upgrade() {
                manager.update(app, |manager, _| manager.shutdown());
            }
            app.quit();
        })
        .detach();

        cx.on_app_quit(|this: &mut AppView, cx| {
            this.manager.update(cx, |manager, _| manager.cancel_all());
            async {}
        })
        .detach();
    }

    fn cards(&self, cx: &Context<Self>) -> Vec<CardData> {
        let manager = self.manager.read(cx);
        manager
            .tasks()
            .iter()
            .enumerate()
            .map(|(index, task)| {
                let progress = task.progress().lock();
                CardData {
                    index,
                    id: task.id(),
                    title: task.display_name().into_owned(),
                    input: task.input().to_owned(),
                    ratio: progress.ratio(),
                    percent: format_percent(progress.ratio()),
                    status_line: progress.status_line().into_owned(),
                    is_downloading: task.has_canceller() && progress.status.is_active(),
                    peers: progress.peers.clone(),
                }
            })
            .collect()
    }

    fn sync_activity_marker(&mut self, active: bool) {
        if self.activity_file.is_none() || self.activity_written == Some(active) {
            return;
        }

        let Some(path) = self.activity_file.as_ref() else {
            return;
        };

        match std::fs::write(path, if active { "1" } else { "0" }) {
            Ok(()) => self.activity_written = Some(active),
            Err(e) => log::warn!("Could not update the background activity marker: {e}"),
        }
    }

    fn input_for(
        &mut self,
        id: usize,
        committed: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<TextInput> {
        let input = self
            .inputs
            .entry(id)
            .or_insert_with(|| {
                let placeholder = t!("task.magnet_label").into_owned();
                let initial = committed.to_owned();
                cx.new(|cx| TextInput::new(placeholder, cx).with_text(initial))
            })
            .clone();

        if input.read(cx).text() != committed && !input.read(cx).is_focused(window) {
            let text = committed.to_owned();
            input.update(cx, |input, cx| input.set_text(&text, window, cx));
        }

        let placeholder = t!("task.magnet_label").into_owned();
        if input.read(cx).placeholder() != placeholder {
            input.update(cx, |input, _| input.set_placeholder(placeholder));
        }

        input
    }

    fn card(
        &mut self,
        data: CardData,
        wide: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let index = data.index;
        let id = data.id;
        let peers = data.peers.clone();
        let input = self.input_for(id, &data.input, window, cx);
        let primary_label = if data.is_downloading {
            t!("task.stop").into_owned()
        } else {
            t!("task.start").into_owned()
        };
        let is_downloading = data.is_downloading;

        let body = div()
            .flex()
            .flex_col()
            .gap_4()
            .w_full()
            .child(input.clone())
            .child(ui::progress_block(
                data.percent.clone(),
                data.ratio,
                data.status_line.clone(),
            ));

        let body = if peers.is_empty() {
            body
        } else {
            let count = peers.len();
            let expanded = self.peers_expanded.contains(&id);
            let toggle_label = if expanded {
                t!("peers.collapse").into_owned()
            } else {
                t!("peers.expand").into_owned()
            };

            let header = div()
                .flex()
                .flex_row()
                .gap_4()
                .items_center()
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(ui::theme::DARK.muted))
                        .child(t!("peers.count", count => count).into_owned()),
                );
            let header = header.child(
                ui::quiet_button(format!("peers-toggle-{id}"), toggle_label).on_click(
                    cx.listener(move |this: &mut AppView, _, _, cx| {
                        if !this.peers_expanded.remove(&id) {
                            this.peers_expanded.insert(id);
                        }
                        cx.notify();
                    }),
                ),
            );

            let body = body.child(header);
            if !expanded {
                body
            } else if wide {
                body.child(ui::peer_table(&peers))
            } else {
                body.child(ui::peer_section(&peers))
            }
        };

        let actions = div().flex().flex_row().gap_4().child(
            ui::button(format!("primary-{id}"), primary_label)
                .on_click(cx.listener(move |this: &mut AppView, _, _, cx| {
                    if is_downloading {
                        this.manager.update(cx, |manager, _| manager.stop_download(index));
                        cx.notify();
                        return;
                    }

                    let typed = input.read(cx).text().to_owned();
                    if typed.is_empty() {
                        return;
                    }
                    this.manager
                        .update(cx, |manager, _| manager.set_input(index, typed));

                    let host = this.host.clone();
                    let manager = this.manager.clone();
                    let task = cx.spawn(async move |_view, cx| {
                        let output_dir = match host.pick(PickKind::OutputFolder).await {
                            Ok(Some(dir)) => dir,
                            Ok(None) => return,
                            Err(e) => {
                                log::warn!("Could not pick output directory: {e}");
                                return;
                            }
                        };

                        let result = manager
                            .update(cx, |manager, _| manager.start_download(index, output_dir));

                        if let Err(e) = result {
                            log::error!("Could not start download: {e}");
                        }
                        cx.refresh();
                    });
                    this.pending.push(task);
                })),
        );

        let actions = actions.child(
            ui::quiet_button(format!("select-{id}"), t!("task.select_file").to_string())
                .on_click(cx.listener(move |this: &mut AppView, _, _, cx| {
                    let host = this.host.clone();
                    let manager = this.manager.clone();
                    let task = cx.spawn(async move |_view, cx| {
                        let path = match host.pick(PickKind::TorrentFile).await {
                            Ok(Some(path)) => path,
                            Ok(None) => return,
                            Err(e) => {
                                log::warn!("Could not pick torrent file: {e}");
                                return;
                            }
                        };

                        manager.update(cx, |manager, _| {
                            manager.set_input(index, path.to_string_lossy().into_owned())
                        });
                        cx.refresh();
                    });
                    this.pending.push(task);
                })),
        );

        let actions = actions.child(
            ui::danger_button(format!("remove-{id}"), t!("task.remove").to_string())
                .on_click(cx.listener(move |this: &mut AppView, _, _, cx| {
                    this.manager.update(cx, |manager, _| manager.remove_task(index));
                    cx.notify();
                })),
        );

        ui::task_card(data.title.clone(), body, actions)
    }
}

impl Render for AppView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let locale_label = i18n::label(&i18n::current()).into_owned();
        let wide = window.viewport_size().width >= px(ui::theme::NARROW_WIDTH);
        let cards = self.cards(cx);
        self.sync_activity_marker(cards.iter().any(|card| card.is_downloading));
        let live_ids: Vec<usize> = cards.iter().map(|card| card.id).collect();
        self.inputs.retain(|id, _| live_ids.contains(id));
        self.peers_expanded.retain(|id| live_ids.contains(id));

        let toolbar = div().flex().flex_row().gap_4().child(
            ui::button("add-task".to_owned(), t!("task.add").to_string())
                .on_click(cx.listener(|this: &mut AppView, _, _, cx| {
                    this.manager.update(cx, |manager, _| {
                        manager.add_task();
                    });
                    cx.notify();
                })),
        );

        let toolbar = toolbar.child(
            ui::quiet_button(
                "switch-locale".to_owned(),
                format!("{}: {}", t!("language"), locale_label),
            )
            .on_click(cx.listener(|this: &mut AppView, _, _, cx| {
                if let Err(e) = i18n::cycle_next(&*this.host) {
                    log::warn!("Could not switch language: {e}");
                }
                cx.notify();
            })),
        );

        let toolbar = toolbar.child(
            ui::quiet_button("crates-io".to_owned(), "crates.io")
                .on_click(|_, _, cx| cx.open_url("https://crates.io/crates/mtorrent")),
        );

        let mut list = div()
            .id("task-list")
            .flex()
            .flex_col()
            .gap_4()
            .p_4()
            .w_full()
            .overflow_y_scroll();
        for card in cards {
            list = list.child(self.card(card, wide, window, cx));
        }

        let chrome = self.host.window_chrome();

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(ui::theme::DARK.background))
            .text_color(rgb(ui::theme::DARK.text))
            .child(ui::header(
                "mtorrent",
                format!("v{}", env!("CARGO_PKG_VERSION")),
                toolbar,
                &chrome,
            ))
            .child(list)
    }
}
