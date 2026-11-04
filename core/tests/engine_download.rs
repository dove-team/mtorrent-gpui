use mtorrent_gpui_core::state::progress::DownloadProgress;
use mtorrent_gpui_core::{RuntimeConfig, Runtimes, TaskManager, ui_channel};
use std::env;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const TRACKERS_JSON: &str = r#"{
  "trackers": [
    "udp://tracker.torrent.eu.org:451/announce",
    "udp://open.stealth.si:80/announce",
    "udp://tracker.dler.org:6969/announce"
  ]
}"#;

fn remove_dir_all(path: &Path) {
    if let Err(e) = std::fs::remove_dir_all(path) {
        if e.kind() != ErrorKind::NotFound {
            panic!("could not clean {}: {e}", path.display());
        }
    }
}

fn snapshot(manager: &TaskManager) -> DownloadProgress {
    let progress = manager.tasks()[0].progress().lock();
    (*progress).clone()
}

#[test]
#[ignore = "需要真实种子网络，且会在 target/test/engine 预分配整个种子的大小"]
fn downloads_a_real_torrent() {
    let Some(input) = env::var_os("MTORRENT_TEST_INPUT") else {
        eprintln!("skipped: set MTORRENT_TEST_INPUT to a .torrent path or magnet link");
        return;
    };

    let target_bytes: u64 = env::var("MTORRENT_TEST_TARGET")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8 * 1024 * 1024);
    let timeout = Duration::from_secs(
        env::var("MTORRENT_TEST_TIMEOUT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(300u64),
    );

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../target/test/engine");
    remove_dir_all(&root);
    let data_dir = root.join("data");
    let out_dir = root.join("out");
    std::fs::create_dir_all(&data_dir).expect("data dir");
    std::fs::create_dir_all(&out_dir).expect("out dir");
    std::fs::write(data_dir.join(".mtorrent_cfg"), TRACKERS_JSON).expect("trackers file");

    let (signal, _watcher) = ui_channel();
    let runtimes = Runtimes::start(RuntimeConfig {
        data_dir: data_dir.clone(),
        use_upnp: false,
        dht_local_port: 6901,
        bind_interface: None,
        ..Default::default()
    })
    .expect("runtimes");

    let mut manager = TaskManager::new(runtimes, signal, None);
    manager.add_task();
    manager.set_input(0, input.to_string_lossy().into_owned());
    manager
        .start_download(0, out_dir.clone())
        .expect("start download");

    let started = Instant::now();
    let mut last = snapshot(&manager);
    loop {
        std::thread::sleep(Duration::from_secs(2));
        let current = snapshot(&manager);
        if current.downloaded_bytes != last.downloaded_bytes
            || current.peers.len() != last.peers.len()
            || current.status != last.status
        {
            eprintln!(
                "t={:0>3}s status={:?} peers={} downloaded={} total={} pieces={}/{}",
                started.elapsed().as_secs(),
                current.status,
                current.peers.len(),
                current.downloaded_bytes,
                current.total_bytes,
                current.metadata_downloaded,
                current.metadata_total,
            );
            last = current;
        }
        if last.downloaded_bytes >= target_bytes || started.elapsed() > timeout {
            break;
        }
    }

    let finished = last.downloaded_bytes;
    let peers = last.peers.len();
    let status = last.status;

    manager.shutdown();
    eprintln!("after shutdown: status={status:?} downloaded={finished} peers={peers}");

    let mut files = Vec::new();
    for entry in Walk::new(&out_dir) {
        files.push(entry);
    }
    let largest = files
        .iter()
        .map(|(_, len)| *len)
        .max()
        .unwrap_or(0);

    assert!(
        finished > 0,
        "no bytes arrived: status={status:?}, peers={peers}, trackers were {TRACKERS_JSON}"
    );
    assert!(
        largest >= finished,
        "the file on disk ({largest}) should hold everything reported as downloaded ({finished})"
    );
}

struct Walk {
    stack: Vec<PathBuf>,
}

impl Walk {
    fn new(root: &Path) -> Self {
        Self {
            stack: vec![root.to_path_buf()],
        }
    }
}

impl Iterator for Walk {
    type Item = (PathBuf, u64);

    fn next(&mut self) -> Option<Self::Item> {
        let dir = self.stack.pop()?;
        let entries = std::fs::read_dir(&dir).ok()?;
        let mut deferred = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                deferred.push(path);
            } else if let Ok(meta) = entry.metadata() {
                return Some((path, meta.len()));
            }
        }
        self.stack.extend(deferred);
        self.next()
    }
}
