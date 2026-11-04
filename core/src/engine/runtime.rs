use mtorrent::app;
use mtorrent::utils::re_exports::mtorrent_dht as dht;
use mtorrent::utils::re_exports::mtorrent_utils::worker;
use std::{io, path::PathBuf};
use tokio::runtime::Handle;

pub struct RuntimeConfig {
    pub data_dir: PathBuf,
    pub use_upnp: bool,
    pub dht_local_port: u16,
    pub bind_interface: Option<String>,
    pub stack_size: usize,
}

/// `mtorrent_utils::worker` defaults to a 128 KiB stack, which the `pwp` thread overflows
/// in debug builds; a stack overflow aborts the whole process without a panic message.
const WORKER_STACK_SIZE: usize = 2 * 1024 * 1024;

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::new(),
            use_upnp: true,
            dht_local_port: 6881,
            bind_interface: None,
            stack_size: WORKER_STACK_SIZE,
        }
    }
}

pub struct Runtimes {
    app_worker: worker::rt::Handle,
    pwp_worker: worker::rt::Handle,
    storage_worker: worker::rt::Handle,
    _dht_worker: app::dht::Handle,
    dht_cmds: dht::CommandSink,
    config: RuntimeConfig,
}

impl Runtimes {
    pub fn start(config: RuntimeConfig) -> io::Result<Self> {
        std::fs::create_dir_all(&config.data_dir)?;

        let app_worker = worker::with_local_runtime(worker::rt::Config {
            name: "app".to_owned(),
            io_enabled: true,
            time_enabled: true,
            stack_size: config.stack_size,
            ..Default::default()
        })?;

        let storage_worker = worker::with_runtime(worker::rt::Config {
            name: "storage".to_owned(),
            io_enabled: false,
            time_enabled: false,
            stack_size: config.stack_size,
            ..Default::default()
        })?;

        let pwp_worker = worker::with_local_runtime(worker::rt::Config {
            name: "pwp".to_owned(),
            io_enabled: true,
            time_enabled: true,
            stack_size: config.stack_size,
            ..Default::default()
        })?;

        let (dht_worker, dht_cmds) = app::dht::launch_dht_node_runtime(app::dht::Config {
            local_port: config.dht_local_port,
            max_concurrent_queries: None,
            config_dir: config.data_dir.clone(),
            use_upnp: config.use_upnp,
            bootstrap_nodes_override: None,
            bind_interface: config.bind_interface.clone(),
            query_timeout: None,
        })?;

        Ok(Self {
            app_worker,
            pwp_worker,
            storage_worker,
            _dht_worker: dht_worker,
            dht_cmds,
            config,
        })
    }

    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    pub fn app_runtime(&self) -> &Handle {
        self.app_worker.runtime_handle()
    }

    pub fn pwp_runtime(&self) -> &Handle {
        self.pwp_worker.runtime_handle()
    }

    pub fn storage_runtime(&self) -> &Handle {
        self.storage_worker.runtime_handle()
    }

    pub fn dht_cmds(&self) -> &dht::CommandSink {
        &self.dht_cmds
    }
}
