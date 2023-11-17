use crate::{
    kill_verdaccio::kill_all_verdaccio_children, node_registry_mock, port_to_url::port_to_url,
    PreparedRegistryInfo, RegistryAnchor, RegistryInfo,
};
use assert_cmd::prelude::*;
use pipe_trait::Pipe;
use portpicker::pick_unused_port;
use reqwest::Client;
use std::{
    fs::File,
    path::Path,
    process::{Child, Command, Stdio},
};
use sysinfo::{Pid, PidExt, Signal};
use tokio::time::{sleep, Duration};

#[derive(Debug)]
pub struct MockInstance {
    pub(crate) process: Child,
}

impl Drop for MockInstance {
    fn drop(&mut self) {
        let MockInstance { process, .. } = self;
        let pid = process.id();
        eprintln!("info: Terminating all verdaccio instances below {pid}...");
        let kill_count = kill_all_verdaccio_children(Pid::from_u32(pid), Signal::Interrupt);
        eprintln!("info: Terminated {kill_count} verdaccio instances");
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MockInstanceOptions<'a> {
    pub client: &'a Client,
    pub port: u16,
    pub stdout: Option<&'a Path>,
    pub stderr: Option<&'a Path>,
    pub max_retries: usize,
    pub retry_delay: Duration,
}

impl<'a> MockInstanceOptions<'a> {
    async fn is_registry_ready(self) -> bool {
        let MockInstanceOptions { client, port, .. } = self;
        let url = port_to_url(port);

        let Err(error) = client.head(url).send().await else {
            return true;
        };

        if error.is_connect() {
            eprintln!("info: {error}");
            return false;
        }

        panic!("{error}");
    }

    async fn wait_for_registry(self) {
        let MockInstanceOptions { max_retries, retry_delay, .. } = self;
        let mut retries = max_retries;

        while !self.is_registry_ready().await {
            retries = retries.checked_sub(1).unwrap_or_else(|| {
                panic!("Failed to check for the registry for {max_retries} times")
            });

            sleep(retry_delay).await;
        }
    }

    pub(crate) async fn spawn(self) -> MockInstance {
        let MockInstanceOptions { port, stdout, stderr, .. } = self;
        let port = port.to_string();

        eprintln!("Preparing...");
        node_registry_mock()
            .pipe(Command::new)
            .arg("prepare")
            .env("PNPM_REGISTRY_MOCK_PORT", &port)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .assert()
            .success();

        let stdout = stdout.map_or_else(Stdio::null, |stdout| {
            File::create(stdout).expect("create file for stdout").into()
        });
        let stderr = stderr.map_or_else(Stdio::null, |stderr| {
            File::create(stderr).expect("create file for stderr").into()
        });
        let process = node_registry_mock()
            .pipe(Command::new)
            .env("PNPM_REGISTRY_MOCK_PORT", &port)
            .stdin(Stdio::null())
            .stdout(stdout)
            .stderr(stderr)
            .spawn()
            .expect("spawn mocked registry");

        self.wait_for_registry().await;

        MockInstance { process }
    }

    pub async fn spawn_if_necessary(self) -> Option<MockInstance> {
        let MockInstanceOptions { port, .. } = self;
        if self.is_registry_ready().await {
            eprintln!("info: {port} is already available");
            None
        } else {
            eprintln!("info: spawning mocked registry...");
            self.spawn().await.pipe(Some)
        }
    }
}

#[derive(Debug)]
#[must_use]
pub enum AutoMockInstance {
    Prepared(PreparedRegistryInfo),
    RefCount(RegistryAnchor),
}

impl AutoMockInstance {
    pub fn load_or_init() -> Self {
        if let Some(prepared) = PreparedRegistryInfo::try_load() {
            return AutoMockInstance::Prepared(prepared);
        }

        let anchor = RegistryAnchor::load_or_init(|| {
            let port = pick_unused_port().expect("pick an unused port");

            let mock_instance = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("build tokio runtime")
                .block_on({
                    MockInstanceOptions {
                        client: &Client::new(),
                        port,
                        stdout: None,
                        stderr: None,
                        max_retries: 5,
                        retry_delay: Duration::from_millis(500),
                    }
                    .spawn()
                })
                .pipe(Box::new)
                .pipe(Box::leak);

            let pid = mock_instance.process.id();

            RegistryInfo { port, pid }
        });

        AutoMockInstance::RefCount(anchor)
    }

    fn info(&self) -> &'_ RegistryInfo {
        match self {
            AutoMockInstance::Prepared(prepared) => &prepared.info,
            AutoMockInstance::RefCount(anchor) => &anchor.info,
        }
    }

    pub fn url(&self) -> String {
        self.info().url()
    }
}
