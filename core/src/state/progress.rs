use mtorrent::utils::re_exports::mtorrent_base::pwp::{PeerOrigin, TransportProto};
use rust_i18n::t;
use std::borrow::Cow;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskStatus {
    Idle,
    Loading,
    Downloading,
    Stopped,
    Completed,
    Failed,
}

impl TaskStatus {
    pub fn label(self) -> Cow<'static, str> {
        match self {
            Self::Idle => t!("status.idle"),
            Self::Loading => t!("status.loading"),
            Self::Downloading => t!("status.downloading"),
            Self::Stopped => t!("status.stopped"),
            Self::Completed => t!("status.completed"),
            Self::Failed => t!("status.failed"),
        }
    }

    pub fn is_active(self) -> bool {
        matches!(self, Self::Loading | Self::Downloading)
    }
}

#[derive(Clone, Debug)]
pub struct PeerInfo {
    pub client: Option<String>,
    pub origin: PeerOrigin,
    pub proto: String,
    pub downloaded: u64,
    pub uploaded: u64,
}

pub fn origin_label(origin: PeerOrigin) -> Cow<'static, str> {
    match origin {
        PeerOrigin::Tracker => t!("peers.origin_tracker"),
        PeerOrigin::Listener => t!("peers.origin_listener"),
        PeerOrigin::Pex => t!("peers.origin_pex"),
        PeerOrigin::Dht => t!("peers.origin_dht"),
        PeerOrigin::Other => t!("peers.origin_other"),
    }
}

pub fn protocol_string(transport: Option<TransportProto>, encrypted: bool) -> String {
    let proto_str = match transport {
        Some(TransportProto::Tcp) => "TCP",
        Some(TransportProto::Utp) => "uTP",
        None => "",
    };
    if encrypted {
        format!("{proto_str} (S)")
    } else {
        proto_str.to_string()
    }
}

#[derive(Clone, Debug)]
pub struct DownloadProgress {
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub metadata_total: usize,
    pub metadata_downloaded: usize,
    pub peers: BTreeMap<String, PeerInfo>,
    pub status: TaskStatus,
    pub run_id: u64,
}

impl Default for DownloadProgress {
    fn default() -> Self {
        Self {
            total_bytes: 0,
            downloaded_bytes: 0,
            metadata_total: 0,
            metadata_downloaded: 0,
            peers: BTreeMap::new(),
            status: TaskStatus::Idle,
            run_id: 0,
        }
    }
}

impl DownloadProgress {
    pub fn ratio(&self) -> f32 {
        if self.total_bytes > 0 {
            self.downloaded_bytes as f32 / self.total_bytes as f32
        } else if self.metadata_total > 0 {
            self.metadata_downloaded as f32 / self.metadata_total as f32
        } else {
            0.0
        }
        .clamp(0.0, 1.0)
    }

    pub fn status_line(&self) -> Cow<'static, str> {
        let downloaded = crate::state::format::format_bytes(self.downloaded_bytes);
        let total = crate::state::format::format_bytes(self.total_bytes);
        let kib = crate::state::format::format_bytes_kib(self.downloaded_bytes);
        t!(
            "status.line",
            status => self.status.label(),
            downloaded => downloaded,
            total => total,
            kib => kib
        )
    }
}
