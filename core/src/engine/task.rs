use crate::engine::listener::{Canceller, listener_with_canceller};
use crate::engine::runtime::Runtimes;
use crate::engine::signal::UiSignal;
use crate::state::progress::{DownloadProgress, PeerInfo, TaskStatus, protocol_string};
use mtorrent::app;
use mtorrent::utils::re_exports::mtorrent_utils::peer_id::PeerId;
use mtorrent::utils::startup;
use parking_lot::{Condvar, Mutex};
use rust_i18n::t;
use std::borrow::Cow;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

const EXIT_WAIT: Duration = Duration::from_secs(5);

pub struct DownloadTask {
    id: usize,
    uri: String,
    name: Option<String>,
    output_dir: PathBuf,
    canceller: Option<Canceller>,
    progress: Arc<Mutex<DownloadProgress>>,
    input: String,
}

impl DownloadTask {
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn uri(&self) -> &str {
        &self.uri
    }

    pub fn output_dir(&self) -> &PathBuf {
        &self.output_dir
    }

    pub fn progress(&self) -> &Mutex<DownloadProgress> {
        &self.progress
    }

    pub fn has_canceller(&self) -> bool {
        self.canceller.is_some()
    }

    pub fn is_downloading(&self) -> bool {
        self.canceller.is_some() && self.progress.lock().status.is_active()
    }

    pub fn display_name(&self) -> Cow<'_, str> {
        match &self.name {
            Some(name) => Cow::Borrowed(name.as_str()),
            None if self.uri.is_empty() && self.input.is_empty() => t!("task.default_name", id => self.id),
            None => t!("task.download_name", id => self.id),
        }
    }
}

pub struct TaskManager {
    peer_id: PeerId,
    runtimes: Runtimes,
    signal: UiSignal,
    tasks: Vec<DownloadTask>,
    next_id: usize,
    in_flight: Arc<(Mutex<usize>, Condvar)>,
}

impl TaskManager {
    pub fn new(runtimes: Runtimes, signal: UiSignal, initial_uri: Option<String>) -> Self {
        let mut manager = Self {
            peer_id: PeerId::generate_new(),
            runtimes,
            signal,
            tasks: Vec::new(),
            next_id: 1,
            in_flight: Arc::new((Mutex::new(0), Condvar::new())),
        };

        if let Some(uri) = initial_uri {
            manager.open_uri(uri);
        }

        manager
    }

    pub fn runtimes(&self) -> &Runtimes {
        &self.runtimes
    }

    pub fn tasks(&self) -> &[DownloadTask] {
        &self.tasks
    }

    pub fn add_task(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        self.tasks.push(DownloadTask {
            id,
            uri: String::new(),
            name: None,
            output_dir: PathBuf::new(),
            canceller: None,
            progress: Arc::new(Mutex::new(DownloadProgress::default())),
            input: String::new(),
        });

        id
    }

    pub fn open_uri(&mut self, uri: String) {
        self.add_task();
        if let Some(task) = self.tasks.last_mut() {
            task.input = uri;
        }
    }

    pub fn remove_task(&mut self, index: usize) {
        if index < self.tasks.len() {
            self.stop_download(index);
            self.tasks.remove(index);
        }
    }

    pub fn stop_download(&mut self, index: usize) {
        if let Some(task) = self.tasks.get_mut(index) {
            task.canceller = None;
            let mut progress = task.progress.lock();
            progress.run_id = progress.run_id.wrapping_add(1);
            progress.status = TaskStatus::Stopped;
        }
    }

    pub fn set_input(&mut self, index: usize, value: String) {
        if let Some(task) = self.tasks.get_mut(index) {
            task.input = value;
        }
    }

    pub fn start_download(&mut self, index: usize, output_dir: PathBuf) -> io::Result<()> {
        let Some(task) = self.tasks.get_mut(index) else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("no task at index {index}"),
            ));
        };

        let uri = task.input.clone();
        if uri.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                t!("task.magnet_label").into_owned(),
            ));
        }

        task.uri = uri.clone();
        task.output_dir = output_dir.clone();
        task.name = startup::get_torrent_name(&uri);

        let run_id = {
            let mut progress = task.progress.lock();
            progress.run_id = progress.run_id.wrapping_add(1);
            progress.status = TaskStatus::Loading;
            progress.total_bytes = 0;
            progress.downloaded_bytes = 0;
            progress.metadata_total = 0;
            progress.metadata_downloaded = 0;
            progress.peers.clear();
            progress.run_id
        };

        let progress = Arc::clone(&task.progress);
        let config_dir = self.runtimes.config().data_dir.clone();
        let use_upnp = self.runtimes.config().use_upnp;
        let bind_interface = self.runtimes.config().bind_interface.clone();
        let peer_id = self.peer_id;
        let app_runtime = self.runtimes.app_runtime().clone();
        let pwp_runtime = self.runtimes.pwp_runtime().clone();
        let storage_runtime = self.runtimes.storage_runtime().clone();
        let dht_cmds = self.runtimes.dht_cmds().clone();
        let snapshot_signal = self.signal.clone();

        let (listener, canceller, cancel) = listener_with_canceller(
            move |snapshot| {
                snapshot_signal.ping();

                let mut prog = progress.lock();
                if prog.run_id != run_id {
                    return;
                }

                prog.total_bytes = snapshot.bytes.total as u64;
                prog.downloaded_bytes = snapshot.bytes.downloaded as u64;
                prog.metadata_total = snapshot.metainfo.total_pieces;
                prog.metadata_downloaded = snapshot.metainfo.downloaded_pieces;

                prog.peers.clear();
                for (addr, state) in &snapshot.peers {
                    let peer_info = PeerInfo {
                        client: state
                            .extensions
                            .as_ref()
                            .and_then(|ext| ext.client_type.as_deref())
                            .map(str::to_owned),
                        origin: state.origin,
                        proto: protocol_string(state.transport, state.encryption),
                        downloaded: state.download.bytes_received as u64,
                        uploaded: state.upload.bytes_sent as u64,
                    };
                    prog.peers.insert(addr.to_string(), peer_info);
                }

                prog.status = if snapshot.metainfo.total_pieces > 0
                    && snapshot.metainfo.downloaded_pieces < snapshot.metainfo.total_pieces
                {
                    TaskStatus::Loading
                } else {
                    TaskStatus::Downloading
                };
            },
            log::Level::Debug,
        );

        task.canceller = Some(canceller);

        *self.in_flight.0.lock() += 1;
        let in_flight = Arc::clone(&self.in_flight);
        let result_signal = self.signal.clone();
        let progress = Arc::clone(&task.progress);

        app_runtime.spawn(async move {
            tokio::task::spawn_local(async move {
                let result = app::main::single_torrent(
                    uri.clone(),
                    listener,
                    cancel,
                    app::main::Config {
                        local_peer_id: peer_id,
                        output_dir,
                        config_dir,
                        use_upnp,
                        pwp_port: None,
                        bind_interface,
                        download_strategy: Default::default(),
                        mode: Default::default(),
                        file_selection: Default::default(),
                    },
                    app::main::Context {
                        dht_handle: Some(dht_cmds),
                        pwp_runtime,
                        storage_runtime,
                    },
                )
                .await;

                {
                    let mut progress = progress.lock();
                    if progress.run_id == run_id {
                        progress.peers.clear();
                        progress.metadata_total = 0;
                        progress.metadata_downloaded = 0;
                        match result {
                            Ok(app::main::Outcome::Finished) => {
                                log::info!("Download completed: {uri}");
                                progress.status = TaskStatus::Completed;
                            }
                            Ok(app::main::Outcome::Cancelled) => {
                                progress.status = TaskStatus::Stopped;
                            }
                            Err(e) => {
                                log::error!("Download failed: {uri} - {e}");
                                progress.status = TaskStatus::Failed;
                            }
                        }
                    }
                }
                result_signal.ping();

                let (lock, condvar) = &*in_flight;
                let mut active = lock.lock();
                *active = active.saturating_sub(1);
                condvar.notify_all();
            });
        });

        Ok(())
    }

    pub fn cancel_all(&mut self) {
        for index in 0..self.tasks.len() {
            self.stop_download(index);
        }
    }

    pub fn wait_for_drain(&mut self) {
        let (lock, condvar) = &*self.in_flight;
        let mut active = lock.lock();
        let deadline = Instant::now() + EXIT_WAIT;
        while *active > 0 {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() || condvar.wait_for(&mut active, remaining).timed_out() {
                break;
            }
        }
    }

    pub fn shutdown(&mut self) {
        self.cancel_all();
        self.wait_for_drain();
    }
}
