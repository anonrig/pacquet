use crate::{
    kill_verdaccio::kill_all_verdaccio_children, port_to_url::port_to_url, MockInstanceOptions,
};
use pipe_trait::Pipe;
use serde::{Deserialize, Serialize};
use std::{
    env::temp_dir,
    fs,
    io::ErrorKind,
    mem::forget,
    path::{Path, PathBuf},
    sync::OnceLock,
};
use sysinfo::{Pid, PidExt, Signal};

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct RegistryInfo {
    pub port: u16,
    pub pid: u32,
}

impl RegistryInfo {
    pub fn url(&self) -> String {
        port_to_url(self.port)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PreparedRegistryInfo {
    pub info: RegistryInfo,
}

impl PreparedRegistryInfo {
    fn path() -> &'static Path {
        static PATH: OnceLock<PathBuf> = OnceLock::new();
        PATH.get_or_init(|| temp_dir().join("pacquet-registry-mock-prepared-registry-info.json"))
    }

    pub fn try_load() -> Option<Self> {
        match PreparedRegistryInfo::path().pipe(fs::read_to_string) {
            Ok(text) => text
                .pipe_as_ref(serde_json::from_str::<PreparedRegistryInfo>)
                .expect("parse prepared registry info")
                .pipe(Some),
            Err(error) if error.kind() == ErrorKind::NotFound => None,
            Err(error) => panic!("Failed to load prepared registry info: {error}"),
        }
    }

    fn save(&self) {
        let text = serde_json::to_string_pretty(self).expect("convert anchor to JSON");
        fs::write(PreparedRegistryInfo::path(), text).expect("write to anchor");
    }

    fn delete() {
        fs::remove_file(PreparedRegistryInfo::path()).expect("delete prepared registry info")
    }

    pub async fn launch(options: MockInstanceOptions<'_>) -> Self {
        let port = options.port;
        let mock_instance = options.spawn().await;
        let pid = mock_instance.process.id();
        let info = RegistryInfo { port, pid };
        let prepared = PreparedRegistryInfo { info };
        prepared.save();
        forget(mock_instance); // prevent this process from killing itself on drop
        prepared
    }

    pub fn end() -> Option<Self> {
        let prepared = PreparedRegistryInfo::try_load()?;
        let pid = prepared.info.pid;

        eprintln!("info: Terminating all verdaccio instances below {pid}...");
        let kill_count = kill_all_verdaccio_children(Pid::from_u32(pid), Signal::Interrupt);
        eprintln!("info: Terminated {kill_count} verdaccio instances");

        PreparedRegistryInfo::delete();
        Some(prepared)
    }
}
